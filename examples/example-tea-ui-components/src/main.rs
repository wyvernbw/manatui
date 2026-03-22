use std::io::Read;
use std::sync::Arc;

use color_eyre::Result;
use color_eyre::eyre::Context;
use manatui::layout::layout::SharedText;
use manatui::prelude::*;
use manatui::ratatui::crossterm::event::Event;
use manatui::tea;
use manatui::tea::Effect;
use manatui::tea::focus::FocusGroup;
use manatui::utils::keyv2;
use manatui_tea_ui::components::button::{Button, ButtonView};
use manatui_tea_ui::components::list::{List, ListViewCompact};
use manatui_tea_ui::components::pager::{Pager, PagerView};
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
    button: Button,
    text_input_1: TextInput,
    text_input_2: TextInput,
    list: List,
    pager: Pager,
    readme: Arc<str>,
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
        let mut readme = std::fs::File::open("readme.md")
            .wrap_err("failed to open readme")
            .unwrap();
        let mut readme_buf = Vec::new();
        readme.read_to_end(&mut readme_buf).unwrap();
        let readme = String::from_utf8_lossy(&readme_buf).into_owned();

        (
            Model {
                button: Button::new(),
                focus: FocusGroup::new().set_wrap_around(true),
                text_input_1: TextInput::new(),
                text_input_2: TextInput::new(),
                list: List::new(),
                readme: readme.into(),
                pager: Pager::new(),
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

                (self.button, _) = self.button.update(&event);
                (self.text_input_1, self.focus) = self.focus.pipe(self.text_input_1.update(&event));
                (self.text_input_2, self.focus) = self.focus.pipe(self.text_input_2.update(&event));
                (self.list, self.focus) = self.focus.pipe(self.list.update(&event));
                self.pager = self.pager.update(&event);
                self = self.build_focus();

                (self, Effect::none())
            }
        }
    }

    fn build_focus(self) -> Self {
        self.focus
            .items()
            .next_untagged(&self.button)
            .next_untagged(&self.text_input_1)
            .next_untagged(&self.text_input_2)
            .next_untagged(&self.list)
            .next_untagged(&self.pager)
            .commit();
        self
    }
}

async fn view(model: &Model) -> View {
    let items = ["agi", "agilao", "agidyne", "media"];
    let readme = SharedText::<Arc<str>, Paragraph>::new(model.readme.clone());
    ui! {
        <Block Width::grow() Height::grow() Center>
            <Block Direction::Horizontal>
                <Block>
                    <ButtonView .state={&model.button}>
                        "Click me!"
                    </ButtonView>
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
                    <ListViewCompact .state={&model.list} .items={items.into_iter()}/>
                </Block>
                <Block>
                    <PagerView .content={readme.into_view()} .state={&model.pager}
                        Width::fixed(54) Height::fixed(24)
                    />
                </Block>
            </Block>
        </Block>
    }
}
