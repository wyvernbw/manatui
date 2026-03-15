#![feature(trait_alias)]
#![allow(clippy::collapsible_if)]

pub mod backends;
#[path = "./focus/focus.rs"]
pub mod focus;
#[cfg(feature = "tachyonfx")]
pub mod fx;
pub mod observe;

use std::{io::stdout, time::Duration};

use crossterm::terminal::enable_raw_mode;
use flume::{Receiver, Sender};
use hecs::Component;
use manatui_layout::{
    layout::{Element, ElementCtx, NodePostRenderSchedule, PostRenderSchedule, Props},
    ui::View,
};
use ratatui::{Terminal, TerminalOptions, layout::Rect, prelude::Backend};
use smallbox::SmallBox;
use tailcall::tailcall;
use tokio::time::Instant;

use crate::{
    backends::{DefaultBackend, DefaultEvent, ManaBackend, MsgStream},
    observe::AreaRef,
};

pub type Chan<Msg> = (Sender<Msg>, Receiver<Msg>);
pub trait UpdateFn<Msg, Model> = AsyncFn(Model, Msg) -> (Model, Effect<Msg>) + Component;
pub trait InitFn<Msg, Model> = AsyncFn() -> (Model, Effect<Msg>) + Component;
pub trait ViewFn<Msg, Model> = AsyncFn(&Model) -> View + Component;
pub trait SignalFn<Msg, Model> = Fn(&Model, &Msg) -> bool;

type PinnedFuture<R> = SmallBox<dyn Future<Output = R> + Send + Sync + 'static, [usize; 4]>;

pub trait EffectFn<Msg>: Send + Sync + 'static {
    fn run_effect(&mut self, tx: Sender<Msg>) -> PinnedFuture<()>;
}

impl<F, Fut, Msg> EffectFn<Msg> for F
where
    F: FnMut(Sender<Msg>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + Sync + 'static,
{
    fn run_effect(&mut self, tx: Sender<Msg>) -> PinnedFuture<()> {
        let future = (self)(tx);
        SmallBox::<Fut, [usize; 4]>::new(future as _)
    }
}
pub struct Effect<Msg>(SmallBox<dyn EffectFn<Msg>, [usize; 4]>);

impl<Msg: Clone + Send + Sync + 'static> Effect<Msg> {
    #[must_use]
    pub fn none() -> Self {
        Self::new(async move |_| {})
    }
    pub fn new<
        Fut: Future<Output = ()> + Send + Sync + 'static,
        F: FnMut(Sender<Msg>) -> Fut + 'static + Send + Sync,
    >(
        f: F,
    ) -> Self {
        Self(SmallBox::new(f) as _)
    }

    pub fn msg(msg: Msg) -> Self {
        Effect::new(move |tx| {
            let msg = msg.clone();
            async move {
                _ = tx.send_async(msg).await;
            }
        })
    }
}

enum RuntimeMsg<Msg> {
    App(Msg, bool),
    Term(<DefaultBackend<std::io::Stdout> as ManaBackend>::Event),
}

#[derive(thiserror::Error, Debug)]
pub enum RuntimeErr {
    #[error("app channel closed")]
    ChannelClosed,
    #[error("error propagating event")]
    PropagateEventError,
    #[error("error running navigation")]
    NaviagtionError(#[from] anyhow::Error),
    #[error("error initializing runtine")]
    InitErr,
    #[error("error restoring terminal")]
    CleanupError,
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Ctx<B: Backend> {
    #[deref]
    #[deref_mut]
    el_ctx: ElementCtx,
    terminal: Terminal<B>,
}

struct RuntimeConfig {
    fps: std::time::Duration,
}

/// # `HotLoop`
///
/// Marker Component.
///
/// While in the tree, the manatui runtime will run in a hot loop instead of
/// sleeping in between events. This should only be used for things like throbbers
/// or animating components that require frequent rerenders.
pub struct HotLoop;

fn is_hot_loop<B: Backend>(ctx: &Ctx<B>) -> bool {
    ctx.el_ctx.query::<&HotLoop>().iter().len() != 0
}

macro_rules! advance_delta {
    ($ctx:expr, $model:expr, $now:expr) => {
        #[cfg_attr(not(feature = "tachyonfx"), expect(unused_variables))]
        let dt = update_delta_time::<Msg>($model, $now);

        #[cfg(feature = "tachyonfx")]
        if let Some(dt) = dt {
            $ctx.advance_fx(dt);
        }
    };
}

#[tailcall]
#[bon::builder]
async fn runtime<Msg: Message, W: std::io::Write>(
    model: Msg::Model,
    view: impl ViewFn<Msg, Msg::Model>,
    update: impl UpdateFn<Msg, Msg::Model>,
    quit_signal: impl SignalFn<Msg, Msg::Model>,
    mut msg_stream: MsgStream<Msg, W>,
    ctx: &mut Ctx<DefaultBackend<W>>,
    prev_root: Option<Element>,
    config: RuntimeConfig,
    event_msg: Option<impl Fn(DefaultEvent) -> Msg>,
) -> Result<(), RuntimeErr> {
    let now = Instant::now();

    let msg = if is_hot_loop(ctx) {
        MsgStream::<Msg, W>::try_next(&mut msg_stream, config.fps).await
    } else {
        Some(MsgStream::<Msg, W>::next(&mut msg_stream).await)
    };

    match msg {
        None => {
            let mut model = model.on_render();
            advance_delta!(ctx, &mut model, &now);
            let prev_root = Some(rerender(ctx, &model, &view, prev_root).await);
            return runtime(
                model,
                view,
                update,
                quit_signal,
                msg_stream,
                ctx,
                prev_root,
                config,
                event_msg,
            );
        }
        Some(RuntimeMsg::App(msg, _)) if quit_signal(&model, &msg) => Ok(()),

        Some(RuntimeMsg::App(msg, false)) => {
            let mut model = model.on_render();
            advance_delta!(ctx, &mut model, &now);
            let (model, mut effect) = update(model, msg).await;
            tokio::spawn(effect.0.run_effect(msg_stream.dispatch.0.clone()));

            let prev_root = Some(rerender(ctx, &model, &view, prev_root).await);

            runtime(
                model,
                view,
                update,
                quit_signal,
                msg_stream,
                ctx,
                prev_root,
                config,
                event_msg,
            )
        }

        Some(RuntimeMsg::App(msg, true)) => {
            let (model, mut effect) = update(model, msg).await;
            tokio::spawn(effect.0.run_effect(msg_stream.dispatch.0.clone()));

            runtime(
                model,
                view,
                update,
                quit_signal,
                msg_stream,
                ctx,
                prev_root,
                config,
                event_msg,
            )
        }

        Some(RuntimeMsg::Term(event)) => {
            let (model, mut effect) = match event_msg {
                Some(ref event_msg) => update(model, event_msg(event.clone())).await,
                None => (model, Effect::none()),
            };
            tokio::spawn(effect.0.run_effect(msg_stream.dispatch.0.clone()));

            let mut model = model.on_render();
            advance_delta!(ctx, &mut model, &now);
            let result = focus::propagate_event::<Msg>(&ctx.el_ctx, &model, &event)
                .map_err(|_| RuntimeErr::PropagateEventError)?;

            if let Some((msg, mut effect)) = result {
                tokio::spawn(effect.0.run_effect(msg_stream.dispatch.0.clone()));
                msg_stream
                    .dispatch
                    .0
                    .send_async(msg)
                    .await
                    .map_err(|_| RuntimeErr::ChannelClosed)?;
            } else {
                _ = focus::navigation_system::<DefaultBackend<W>>(&mut ctx.el_ctx, &event);
                _ = focus::set_focus_style(&mut ctx.el_ctx);
                render::<Msg, W>(ctx, view(&model).await);
            }

            runtime(
                model,
                view,
                update,
                quit_signal,
                msg_stream,
                ctx,
                prev_root,
                config,
                event_msg,
            )
        }
    }
}

fn update_delta_time<Msg: Message>(model: &mut Msg::Model, now: &Instant) -> Option<Duration> {
    if let Some(delta) = model.delta_time_mut() {
        *delta = now.elapsed();
        Some(*delta)
    } else {
        None
    }
}

async fn rerender<Msg: Message, W: std::io::Write>(
    ctx: &mut Ctx<DefaultBackend<W>>,
    model: &Msg::Model,
    view: &impl ViewFn<Msg, Msg::Model>,
    prev_root: Option<Element>,
) -> Element {
    if let Some(prev) = prev_root {
        ctx.despawn_ui(prev);
    }
    render::<Msg, W>(ctx, view(model).await)
}

fn render<Msg: Message, W: std::io::Write>(
    ctx: &mut Ctx<DefaultBackend<W>>,
    view: View,
) -> Element {
    let root = ctx.spawn_ui(view);
    draw::<Msg, W>(ctx, root)
}

fn draw<Msg: Message, W: std::io::Write>(
    ctx: &mut Ctx<DefaultBackend<W>>,
    root: Element,
) -> Element {
    let result = ctx.terminal.draw(|frame| {
        let result = ctx.el_ctx.calculate_layout(root, frame.area());
        focus::generate_ui_stack(&mut ctx.el_ctx, root);
        focus::init_focus_system(&mut ctx.el_ctx);
        focus::handlers::specialize_on_click_or_key_handlers::<Msg>(&mut ctx.el_ctx);
        _ = focus::set_focus_style(&mut ctx.el_ctx);

        if let Err(err) = result {
            tracing::error!("failed to calculate layout: {err}");
            return;
        }

        ctx.el_ctx.render(root, frame.area(), frame.buffer_mut());
    });

    if let Err(err) = result {
        tracing::error!("failed to draw: {err}");
    }

    root
}

/// # Errors
///
/// errors here should be treated as fatal. this function errros:
///
/// - if the app channel is closed somehow
/// - if an error happens while propagating an event
/// - if there is an error initializing the runtime
#[bon::builder]
#[builder(finish_fn = run)]
pub async fn run_in<W, Msg>(
    #[builder(start_fn)] writer: W,
    init: impl InitFn<Msg, Msg::Model>,
    view: impl ViewFn<Msg, Msg::Model>,
    update: impl UpdateFn<Msg, Msg::Model>,
    quit_signal: impl SignalFn<Msg, Msg::Model>,
    #[builder(default, name = "with_options")] options: TerminalOptions,
    #[builder(default = std::time::Duration::from_millis(50))] fps: std::time::Duration,
    event_msg: Option<impl Fn(DefaultEvent) -> Msg>,
) -> Result<(), RuntimeErr>
where
    Msg: Clone + Message + Component,
    W: std::io::Write + 'static,
{
    fn set_panic_hook() {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            ratatui::restore();
            hook(info);
        }));
    }

    let is_inline = matches!(options.viewport, ratatui::Viewport::Inline(_));
    let dispatch = flume::unbounded::<Msg>();
    let mut backend = DefaultBackend::new(writer);
    let msg_stream = MsgStream {
        event_stream: backend.create_events().await,
        dispatch: dispatch.clone(),
    };
    let terminal =
        ratatui::Terminal::with_options(backend, options).map_err(|_| RuntimeErr::InitErr)?;

    set_panic_hook();
    enable_raw_mode().map_err(|_| RuntimeErr::InitErr)?;

    let mut ctx = Ctx {
        el_ctx: manatui_layout::prelude::ElementCtx::new(),
        terminal,
    };

    #[cfg(feature = "tachyonfx")]
    {
        ctx.setup_fx();
    }
    ctx.add_system::<PostRenderSchedule>(|world, _, _| {
        for (props, area_ref) in world.query_mut::<(&Props, &AreaRef)>() {
            area_ref.set(ratatui::layout::Rect {
                x: props.position.x,
                y: props.position.y,
                width: props.size.x,
                height: props.size.y,
            });
        }
    });

    let mut cursor_pos = ctx
        .terminal
        .get_cursor_position()
        .map_err(|_| RuntimeErr::InitErr)?;

    let (model, mut effect) = init().await;
    tokio::spawn(effect.0.run_effect(dispatch.0.clone()));
    let tree = view(&model).await;
    let root = render::<Msg, W>(&mut ctx, tree);

    runtime()
        .model(model)
        .view(view)
        .update(update)
        .quit_signal(quit_signal)
        .msg_stream(msg_stream)
        .ctx(&mut ctx)
        .prev_root(root)
        .config(RuntimeConfig { fps })
        .maybe_event_msg(event_msg)
        .call()
        .await?;

    ratatui::restore();

    if is_inline {
        let area = ctx.terminal.get_frame().area();
        cursor_pos.y = cursor_pos.y.saturating_sub(area.height) + 1;
        _ = ctx.terminal.set_cursor_position(cursor_pos);
        ctx.terminal
            .backend_mut()
            .clear_region(ratatui::backend::ClearType::AfterCursor)
            .map_err(|_| RuntimeErr::CleanupError)?;
    }

    Ok(())
}

/// # Errors
///
/// errors here should be treated as fatal. this function errros:
///
/// - if the app channel is closed somehow
/// - if an error happens while propagating an event
/// - if there is an error initializing the runtime
#[bon::builder]
#[builder(finish_fn = run)]
pub async fn run<Msg>(
    init: impl InitFn<Msg, Msg::Model>,
    view: impl ViewFn<Msg, Msg::Model>,
    update: impl UpdateFn<Msg, Msg::Model>,
    quit_signal: impl SignalFn<Msg, Msg::Model>,
    #[builder(default, name = "with_options")] options: TerminalOptions,

    #[cfg(feature = "crossterm")]
    #[builder(default)]
    enable_mouse: bool,

    #[builder(default = std::time::Duration::from_millis(50))] fps: std::time::Duration,

    event_msg: Option<impl Fn(DefaultEvent) -> Msg>,
) -> Result<(), RuntimeErr>
where
    Msg: Clone + Message + Component,
{
    let is_inline = matches!(options.viewport, ratatui::Viewport::Inline(_));

    #[cfg(feature = "crossterm")]
    {
        if !is_inline {
            _ = crossterm::execute!(stdout(), crossterm::terminal::EnterAlternateScreen);
        }
        if enable_mouse {
            let hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                _ = crossterm::execute!(stdout(), crossterm::event::DisableMouseCapture);
                hook(info);
            }));
            _ = crossterm::execute!(stdout(), crossterm::event::EnableMouseCapture);
        }
    }

    run_in(stdout())
        .with_options(options)
        .init(init)
        .view(view)
        .update(update)
        .quit_signal(quit_signal)
        .fps(fps)
        .maybe_event_msg(event_msg)
        .run()
        .await?;

    #[cfg(feature = "crossterm")]
    {
        if !is_inline {
            _ = crossterm::execute!(stdout(), crossterm::terminal::LeaveAlternateScreen);
        }
        if enable_mouse {
            _ = crossterm::execute!(stdout(), crossterm::event::DisableMouseCapture);
        }
    }

    Ok(())
}

/// Marker trait that must be implemented by Messages in order to tie them to
/// their model type.
pub trait Message: Clone + Component {
    type Model: Model;
}

/// Marker trait that must be implemented by Models in order to use certain
/// extension methods
pub trait Model: Sized {
    /// The field return will be kept up to date with the delta time,
    /// i.e. the time between this frame and the last one. Useful for
    /// hot render loops or animations.
    fn delta_time_mut(&mut self) -> Option<&mut std::time::Duration> {
        None
    }
    /// Function that runs before every render and allows updating the model.
    /// This is mainly useful for updating animations of [`HotLoop`] components.
    #[must_use]
    fn on_render(self) -> Self {
        self
    }
}

impl Model for () {}
