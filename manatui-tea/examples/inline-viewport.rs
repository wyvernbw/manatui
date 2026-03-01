use anyhow::Result;
use crossterm::event::{Event, KeyModifiers};
use manatui::prelude::*;
use manatui_macros::ui;
use manatui_tea::{Effect, Message, focus::handlers::On};
use manatui_utils::key;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    manatui_tea::run()
        .quit_signal(|(), msg| matches!(msg, Msg::Quit))
        .update(async |model, _| (model, Effect::none()))
        .init(async || ((), Effect::none()))
        .view(view)
        .with_options(ratatui::TerminalOptions {
            viewport: ratatui::Viewport::Inline(5),
        })
        .run()
        .await?;
    Ok(())
}

#[derive(Debug, Clone)]
enum Msg {
    Quit,
}

impl Message for Msg {
    type Model = ();
}

async fn view((): &()) -> View {
    ui! {
        // "Hello world!"
        <Block .rounded .title_top="magical" On::new(input_handler) Width::grow() Height::grow()>
        </Block>
    }
}

fn input_handler((): &(), event: &Event) -> Option<(Msg, Effect<Msg>)> {
    match event {
        key!(Char('c' | 'C'), KeyModifiers::CONTROL) => Some((Msg::Quit, Effect::none())),
        _ => None,
    }
}
