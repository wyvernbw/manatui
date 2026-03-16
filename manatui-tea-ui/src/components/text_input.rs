use manatui::ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use manatui::ratatui::layout::Offset;
use manatui::ratatui::text::Span;
use manatui::tea::focus::{DEFAULT_KEYMAP, Focus, VIM_CTRL_KEYMAP};
use manatui::tea::observe::{AreaRef, HitTest};
use manatui::tea::term::CursorAt;
use manatui::utils::keyv2;
use manatui::{prelude::*, tea};

#[derive(Default, Debug, Clone)]
pub struct TextInput {
    text: String,
    focused: bool,
    cursor: u16,
    area_ref: AreaRef,
    hit_test: HitTest,
}

#[subview]
pub fn text_input_view(state: &TextInput, #[builder(default = "")] placeholder: &str) -> View {
    let text = &state.text;
    let hit = &state.hit_test;
    let cursor = state.area_ref.get().map(|rect| {
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

    #[must_use]
    pub fn update(mut self, event: &Event) -> Self {
        if !self.focused {
            return self;
        }
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::CONTROL | KeyModifiers::META,
                ..
            }) => {
                self.text.clear();
                self.cursor = 0;
                self
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers,
                ..
            }) => {
                self.text.pop();
                self.cursor = self.cursor.saturating_sub(1);
                self
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers,
                ..
            }) => {
                if !modifiers.difference(KeyModifiers::SHIFT).is_empty() {
                    return self;
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

    fn keymaps(&self) -> &'static [tea::focus::KeyMap] {
        &[DEFAULT_KEYMAP, VIM_CTRL_KEYMAP]
    }

    fn hit_test(&self) -> manatui::tea::observe::HitEvent {
        self.hit_test.get()
    }
}
