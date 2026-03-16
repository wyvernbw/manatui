use manatui::{
    prelude::*,
    ratatui::{
        crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
        text::Span,
    },
    tea::observe::{AreaRef, HitTest},
    utils::keyv2,
};

use crate::focus::{DEFAULT_KEYMAP, Focus, VIM_CTRL_KEYMAP};

#[derive(Default, Debug, Clone)]
pub struct TextInput {
    text: String,
    focused: bool,
    cursor: usize,
    area_ref: AreaRef,
    hit_test: HitTest,
}

#[subview]
pub fn text_input_view<'a, 'b>(
    state: &'a TextInput,
    label: Option<&'b str>,
    label_style: Option<Style>,
    label_style_focused: Option<Style>,
) -> View {
    let text = &state.text;
    let label = label.as_ref().map(|label| label.as_ref()).unwrap_or("");
    let label_style = label_style.unwrap_or_default();
    let label_style_focused =
        label_style_focused.unwrap_or_else(|| Style::new().bold().fg(Color::Green));
    let label_style = match state.focused {
        false => label_style,
        true => label_style_focused,
    };
    let hit = &state.hit_test;
    ui! {
        <Block {state.area_ref.clone()} {state.hit_test.clone()}>
            <Span .style={label_style}>"{label}:"</Span>
            "> {text}"
        </Block>
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(mut self, event: &Event) -> Self {
        if !self.focused {
            return self;
        }
        match event {
            keyv2!(backspace) => {
                self.text.pop();
                self
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers,
                ..
            }) => {
                match modifiers.contains(KeyModifiers::SHIFT) {
                    true => {
                        self.text.extend(ch.to_uppercase());
                    }
                    false => {
                        self.text.push(*ch);
                    }
                };
                self.cursor += 1;
                self
            }
            _ => self,
        }
    }
}

impl Focus for TextInput {
    fn set_focus(&mut self, value: bool) {
        self.focused = value;
    }

    fn focus(&self) -> bool {
        self.focused
    }

    fn rect(&self) -> Option<ratatui::prelude::Rect> {
        self.area_ref.get()
    }

    fn keymaps(&self) -> &'static [crate::focus::KeyMap] {
        &[DEFAULT_KEYMAP, VIM_CTRL_KEYMAP]
    }

    fn hit_test(&self) -> manatui::tea::observe::HitEvent {
        self.hit_test.get()
    }
}
