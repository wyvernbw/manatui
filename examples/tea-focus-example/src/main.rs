use std::io::stdout;

use color_eyre::Result;
use manatui::ratatui::crossterm::event::{EnableMouseCapture, Event};
use manatui::tea::Effect;
use manatui::tea::{self, focus};
use manatui::utils::keyv2;
use manatui::{prelude::*, tea::focus::FocusGroup};

use crate::input::{TextInput, TextInputView};

pub mod input;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tea::run()
        .init(Model::init)
        .view(view)
        .update(Model::update)
        .event_msg(Msg::Event)
        .quit_signal(|_, msg| matches!(msg, Msg::Quit))
        .enable_mouse(true)
        .run()
        .await?;

    Ok(())
}

struct Model {
    focus_group: FocusGroup,
    input_1: TextInput,
    input_2: TextInput,
    input_3: TextInput,
}

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
        let mut model = Model {
            input_1: TextInput::new(),
            input_2: TextInput::new(),
            input_3: TextInput::new(),
            focus_group: FocusGroup::new(),
        };
        model.handle_focus();

        (model, Effect::none())
    }

    async fn update(self, msg: Msg) -> (Self, Effect<Msg>) {
        match msg {
            Msg::Quit => unreachable!(),
            Msg::Event(event) => self.handle_event(event).await,
        }
    }

    async fn handle_event(mut self, event: Event) -> (Self, Effect<Msg>) {
        match event {
            keyv2!(ctrl + 'c') => (self, Effect::msg(Msg::Quit)),
            event => {
                match self.focus_group.update(&event) {
                    focus::EventOutcome::Consumed(f) => {
                        self.focus_group = f;
                        self.handle_focus();
                        return (self, Effect::none());
                    }
                    focus::EventOutcome::Unhandled(f) => {
                        self.focus_group = f;
                        self.handle_focus();
                    }
                }

                self.input_1 = self.input_1.update(&event);
                self.input_2 = self.input_2.update(&event);
                self.input_3 = self.input_3.update(&event);

                (self, Effect::none())
            }
        }
    }

    fn handle_focus(&mut self) {
        self.focus_group
            .items(&mut self.input_1)
            .next(&mut self.input_2)
            .next(&mut self.input_3)
            .commit();
    }
}

async fn view(model: &Model) -> View {
    ui! {
        <Block>
            "i am the magic rat 🐭🪄 (press ^C to exit.)"
            <TextInputView .state={&model.input_1} .label="username"/>
            <TextInputView .state={&model.input_2} .label="favorite food"/>
            <TextInputView .state={&model.input_3} .label="level"/>
        </Block>
    }
}
