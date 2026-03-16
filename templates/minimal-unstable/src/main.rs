use color_eyre::Result;
use manatui::prelude::*;
use manatui::ratatui::crossterm::event::Event;
use manatui::tea;
use manatui::tea::Effect;
use manatui::utils::keyv2;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tea::run()
        .init(Model::init)
        .view(view)
        .update(Model::update)
        .event_msg(Msg::Event)
        .quit_signal(|_, msg| matches!(msg, Msg::Quit))
        .run()
        .await?;

    Ok(())
}

struct Model {}

#[derive(Debug, Clone)]
enum Msg {
    Quit,
    Event(Event),
}

impl tea::Message for Msg {
    type Model = Model;
}

impl tea::Model for Model {}

impl Model {
    async fn init() -> (Self, Effect<Msg>) {
        (Model {}, Effect::none())
    }

    async fn update(self, msg: Msg) -> (Self, Effect<Msg>) {
        match msg {
            Msg::Quit => unreachable!(),
            Msg::Event(event) => self.handle_event(event).await,
        }
    }

    async fn handle_event(self, event: Event) -> (Self, Effect<Msg>) {
        match event {
            keyv2!(ctrl + 'c') => (self, Effect::msg(Msg::Quit)),
            _ => (self, Effect::none()),
        }
    }
}

async fn view(model: &Model) -> View {
    ui! {
        "i am the magic rat 🐭🪄 (press ^C to exit.)"
    }
}
