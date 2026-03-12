use std::io::stdout;
use std::time::Duration;

use crossterm::event::{EnableMouseCapture, Event, KeyEvent, KeyModifiers};
use manatui_layout::prelude::*;
use manatui_layout::ui::View;
use manatui_macros::{subview, ui};
use manatui_tea::backends::{DefaultEvent, KeyEventExt};
use manatui_tea::focus::handlers::{ClickOnEnter, On, OnClickOrKey};
use manatui_tea::focus::{FocusStyle, FocusTarget};
use manatui_tea::{Effect, Message, run};
use manatui_utils::key;
use ratatui::style::Style;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    fn should_quit(_: &Model, event: &AppMsg) -> bool {
        matches!(event, AppMsg::Quit)
    }
    run()
        .init(init)
        .view(view)
        .update(update)
        .quit_signal(should_quit)
        .run()
        .await
        .unwrap();
}

#[derive(Debug, Default, Clone)]
struct Model {
    value: i32,
    awake: bool,
}

#[derive(Debug, Clone)]
enum AppMsg {
    Inc,
    Dec,
    Quit,
    Wake,
}

impl Message for AppMsg {
    type Model = Model;
}

impl manatui_tea::Model for Model {}

async fn init() -> (Model, Effect<AppMsg>) {
    _ = crossterm::execute!(stdout(), EnableMouseCapture);
    (
        Model::default(),
        Effect::new(async |tx| {
            tokio::time::sleep(Duration::from_secs(1)).await;
            _ = tx.send_async(AppMsg::Wake).await;
        }),
    )
}

#[subview]
fn my_button<Marker: 'static>(keybind: KeyEvent, msg: AppMsg, tooltip: &'static str) -> View {
    ui! {
        <Block
            .rounded .title_bottom={tooltip}.title_alignment={ratatui::layout::HorizontalAlignment::Center}
            // This is only to illustrate that you can use a dynamic key for focus,
            // since we use Marker types here anyways it would be better to just use `new`
            FocusTarget::new_dyn(std::any::type_name::<Marker>())
            // FocusTarget::new::<Marker>()

            FocusStyle(Style::new().green())
            Width::fixed(5) Center
            ClickOnEnter
            OnClickOrKey::new(keybind, msg)
        >
        </Block>
    }
}

async fn view(model: &Model) -> View {
    struct DecButton;
    struct IncButton;

    let count = model.value;

    ui! {
        <Block
            .rounded
            .title_top="Magical App"
            Center
            Gap(1)
            Height::grow() Width::grow()
            On::new(handle_quit)
        >
            <Block Direction::Horizontal CrossJustify::Center Gap(2)>
                <MyButton(DecButton) .keybind={KeyEvent::char('j')} .tooltip="j" .msg={AppMsg::Dec}>"-"</MyButton>
                <Block Width::fixed(20) Height::fixed(1) Center>
                {
                    if model.awake {
                        format!("I have awoken {count}")
                    } else {
                        "I sleep...".to_string()
                    }
                }
                </Block>
                <MyButton(IncButton) .keybind={KeyEvent::char('k')} .tooltip="k" .msg={AppMsg::Inc}>"+"</MyButton>
            </Block>
            <Paragraph .wrap={Wrap::default()} Width::fixed(40)>
                "Tip: you can click the buttons with the mouse, or switch focus between them with the arrow keys or H and L and press Enter."
            </Paragraph>
        </Block>
    }
}

fn handle_quit(_: &Model, event: &DefaultEvent) -> Option<(AppMsg, Effect<AppMsg>)> {
    match event {
        key!(Char('q')) | key!(Char('c'), KeyModifiers::CONTROL) => {
            Some((AppMsg::Quit, Effect::none()))
        }
        _ => None,
    }
}

async fn update(model: Model, msg: AppMsg) -> (Model, Effect<AppMsg>) {
    match msg {
        AppMsg::Inc => (
            Model {
                value: model.value + 1,
                ..model
            },
            Effect::none(),
        ),
        AppMsg::Dec => (
            Model {
                value: model.value - 1,
                ..model
            },
            Effect::none(),
        ),
        AppMsg::Wake => (
            Model {
                awake: true,
                ..model
            },
            Effect::none(),
        ),
        AppMsg::Quit => (model, Effect::none()),
    }
}
