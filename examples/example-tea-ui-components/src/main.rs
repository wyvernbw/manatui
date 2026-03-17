use color_eyre::Result;
use manatui::prelude::*;
use manatui::ratatui::crossterm::event::Event;
use manatui::tea;
use manatui::tea::Effect;
use manatui::tea::focus::FocusGroup;
use manatui::utils::keyv2;
use manatui_tea_ui::components::text_input::{TextInput, TextInputView};

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
    focus: FocusGroup,
    text_input_1: TextInput,
    text_input_2: TextInput,
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
        (
            Model {
                focus: FocusGroup::new(),
                text_input_1: TextInput::new(),
                text_input_2: TextInput::new(),
            },
            Effect::none(),
        )
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
                match self.focus.update(&event) {
                    tea::focus::EventOutcome::Consumed(focus) => {
                        self.focus = focus;
                        self = self.build_focus();
                        return (self, Effect::none());
                    }
                    tea::focus::EventOutcome::Unhandled(focus) => {
                        self.focus = focus;
                    }
                }

                (self.text_input_1, self.focus) = self.focus.pipe(self.text_input_1.update(&event));
                (self.text_input_2, self.focus) = self.focus.pipe(self.text_input_2.update(&event));
                self = self.build_focus();

                (self, Effect::none())
            }
        }
    }

    fn build_focus(mut self) -> Self {
        self.focus
            .items(&self.text_input_1)
            .next(&self.text_input_2)
            .commit();
        self
    }
}

async fn view(model: &Model) -> View {
    ui! {
        <Block Center Width::grow() Height::grow()>
            <TextInputView
                .state={&model.text_input_1}
                .placeholder="Jack Frost"
                Width::fixed(20)
            />
            <TextInputView
                .state={&model.text_input_2}
                .placeholder="Magician"
                Width::fixed(20)
            />
        </Block>
    }
}
