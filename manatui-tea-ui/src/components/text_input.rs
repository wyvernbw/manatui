use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use manatui::ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use manatui::tea::focus::{DEFAULT_KEYMAP, Focus, FocusEvent, VIM_CTRL_KEYMAP};
use manatui::tea::observe::{AreaRef, HitTest};
use manatui::utils::keyv2;
use manatui::{prelude::*, tea};

#[derive(Default, Debug, Clone)]
pub struct TextInput {
    text: String,
    focused: Arc<AtomicBool>,
    cursor: u16,
    area_ref: AreaRef,
    hit_test: HitTest,
}

#[subview]
pub fn text_input_view(state: &TextInput, #[builder(default = "")] placeholder: &str) -> View {
    let text = &state.text;
    let cursor = state
        .focused
        .load(Ordering::Relaxed)
        .then(|| state.area_ref.get())
        .flatten()
        .map(|rect| {
            ui(Block::new().style(Style::new().remove_modifier(Modifier::DIM).fg(Color::Green)))
                .with((
                    Width::fixed(1),
                    Height::fixed(1),
                    Position::Absolute(
                        Value::Cells(rect.x + state.cursor + 2),
                        Value::Cells(rect.y),
                    ),
                ))
                .child(ui(Text::raw("█")))
                .done()
        });
    ui(Block::new())
        .with((
            Direction::Horizontal,
            state.area_ref.clone(),
            state.hit_test.clone(),
        ))
        .children((
            ui(Text::raw("> ")),
            ui(if state.text.is_empty() {
                Text::raw(placeholder.to_string()).style(Style::new().dim())
            } else {
                Text::raw(text.clone())
            }),
            cursor.unwrap_or_else(|| ui(Text::raw("")).done()),
        ))
        .done()
}

impl TextInput {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn event(self, event: TextInputEvent) -> (Self, TextInputEvent) {
        (self, event)
    }

    fn no_event(self) -> (Self, TextInputEvent) {
        self.event(TextInputEvent::None)
    }

    #[must_use]
    pub fn update(mut self, event: &Event) -> (Self, TextInputEvent) {
        if !self.focused.load(Ordering::Relaxed) {
            return self.no_event();
        }
        match event {
            keyv2!(enter) => self.event(TextInputEvent::Confirm),
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::CONTROL | KeyModifiers::META,
                ..
            }) => {
                self.text.clear();
                self.cursor = 0;
                self.no_event()
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers,
                ..
            }) => {
                self.text.pop();
                self.cursor = self.cursor.saturating_sub(1);
                self.no_event()
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers,
                ..
            }) => {
                if !modifiers.difference(KeyModifiers::SHIFT).is_empty() {
                    return self.no_event();
                }
                match modifiers.contains(KeyModifiers::SHIFT) {
                    true => {
                        self.text.extend(ch.to_uppercase());
                    }
                    false => {
                        self.text.push(*ch);
                    }
                }
                self.cursor += 1;
                self.no_event()
            }
            _ => self.no_event(),
        }
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.text
    }
}

pub enum TextInputEvent {
    None,
    Confirm,
}

impl From<TextInputEvent> for FocusEvent {
    fn from(value: TextInputEvent) -> FocusEvent {
        match value {
            TextInputEvent::None => FocusEvent::None,
            TextInputEvent::Confirm => FocusEvent::Next,
        }
    }
}

impl Focus for TextInput {
    fn set_focus(&self, value: bool) {
        self.focused.store(value, Ordering::Relaxed);
    }

    fn focus(&self) -> bool {
        self.focused.load(Ordering::Relaxed)
    }

    fn rect(&self) -> Option<ratatui::prelude::Rect> {
        self.area_ref.get()
    }

    fn keymaps(&self) -> &'static [tea::focus::KeyMap] {
        &[DEFAULT_KEYMAP, VIM_CTRL_KEYMAP]
    }

    fn hit_test(&self) -> manatui::tea::observe::HitEvent {
        self.hit_test.get()
    }
}
