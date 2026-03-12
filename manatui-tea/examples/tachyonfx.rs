use std::time::Duration;

use crossterm::event::KeyModifiers;
use manatui::prelude::*;
use manatui_macros::ui;
use manatui_tea::{
    Effect, HotLoop, Message,
    focus::handlers::On,
    fx::{self, Fx},
    run,
};
use tachyonfx::EffectManager;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    fn should_quit(_: &Model, msg: &Msg) -> bool {
        matches!(msg, Msg::Quit)
    }
    run()
        .init(init)
        .view(view)
        .update(update)
        .quit_signal(should_quit)
        .fps(Duration::from_millis(16)) // ~60fps hot loop
        .with_options(ratatui::TerminalOptions {
            viewport: ratatui::Viewport::Inline(6),
        })
        .run()
        .await
        .unwrap();
}

#[derive(Clone)]
enum Msg {
    Quit,
}

impl Message for Msg {
    type Model = Model;
}

#[derive(Default)]
struct Model {
    dt: Duration,
    shine: Fx,
}

impl manatui::tea::Model for Model {
    fn delta_time_mut(&mut self) -> Option<&mut std::time::Duration> {
        Some(&mut self.dt)
    }
}

async fn init() -> (Model, Effect<Msg>) {
    let mut model = Model::default();
    let bright = Color::from_u32(0xc4c4c4);
    let dark = Color::from_u32(0x8c8c8c);
    let effect = fx::sequence(&[
        fx::fade_to(bright, bright, 250),
        fx::fade_to(dark, dark, 250),
    ]);
    let shine = fx::repeating(effect);
    model.shine = Fx::new(shine);
    (Model::default(), Effect::none())
}

async fn view(model: &Model) -> View {
    ui! {
        <Block
            .rounded
            On::new(|_, event| {
                match event {
                    key!(Char('c' | 'C'), KeyModifiers::CONTROL) => Some((Msg::Quit, Effect::none())),
                    _ => None,
                }
            })
            Height::fixed(4)
        >
            "Loading content..."
            <Block .style={Style::new().on_white()}
                HotLoop
                {model.shine.clone()}
                Width::grow() Height::grow()
            >
            </Block>
        </Block>
    }
}

async fn update(model: Model, msg: Msg) -> (Model, Effect<Msg>) {
    match msg {
        Msg::Quit => (model, Effect::none()),
    }
}
