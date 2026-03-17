use std::ops::RangeInclusive;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use manatui::ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use manatui::ratatui::text::{Line, Span};
use manatui::tea::focus::{CTRL_KEYMAP, DEFAULT_KEYMAP, Focus, FocusEvent, VIM_CTRL_KEYMAP};
use manatui::tea::observe::{AreaRef, HitTest};
use manatui::utils::keyv2;
use manatui::{prelude::*, tea};

use crate::CLIPBOARD;

#[derive(Default, Debug, Clone)]
pub struct TextInput {
    text: String,
    focused: Arc<AtomicBool>,
    cursor: u16,
    area_ref: AreaRef,
    hit_test: HitTest,
    selection: Option<Selection>,
}

#[derive(Debug, Clone)]
struct Selection {
    range: RangeInclusive<u16>,
}

impl Selection {
    #[inline(always)]
    fn range_as<T: From<u16>>(&self) -> RangeInclusive<T> {
        let start = (*self.range.start()).into();
        let end = (*self.range.end()).into();
        start..=end
    }
}

fn split_three_split_at(s: &str, range: RangeInclusive<u16>) -> (&str, &str, &str) {
    let len = s.len();
    let start = *range.start() as usize;
    let end = *range.end() as usize; // inclusive end

    // Clamp the indices so we never panic
    let start = start.min(len);
    let end = end.min(len).max(start); // ensure end >= start

    let (a, rest) = s.split_at(start);
    let (mid, b) = rest.split_at(end - start);

    (a, mid, b)
}

#[subview]
pub fn text_input_view(state: &TextInput, #[builder(default = "")] placeholder: &str) -> View {
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

    let text = match &state.selection {
        None => Line::raw(state.text.clone()),
        Some(sel) => {
            let (start, mid, end) = split_three_split_at(&state.text, sel.range.clone());
            Line::from_iter([
                Span::raw(start.to_owned()),
                Span::raw(mid.to_owned()).style(Style::new().reversed()),
                Span::raw(end.to_owned()),
            ])
        }
    };

    ui(Block::new())
        .with((
            Direction::Horizontal,
            state.area_ref.clone(),
            state.hit_test.clone(),
        ))
        .children((
            ui(Text::raw("> ")),
            ui(if state.text.is_empty() {
                Line::raw(placeholder.to_string()).style(Style::new().dim())
            } else {
                text
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
    pub fn no_selection(self) -> Self {
        Self {
            selection: None,
            ..self
        }
    }

    fn selection_to_clipboard(&self) {
        if let Some(sel) = &self.selection {
            let str = self.text.get(sel.range_as::<usize>()).unwrap_or("");
            _ = CLIPBOARD.lock().unwrap().set_text(str);
        }
    }

    #[inline(always)]
    fn delete_selection(mut self) -> Self {
        match self.selection {
            None => self,
            Some(sel) => {
                let range = *sel.range.start() as usize..*sel.range.end() as usize;
                self.cursor = *sel.range.start();
                self.text.replace_range(range, "");
                self.selection = None;
                self
            }
        }
    }

    fn move_cursor_at(mut self, cursor: u16) -> Self {
        self.cursor = cursor.clamp(0, self.text.len() as u16);
        self
    }

    fn move_cursor(mut self, offset: i16) -> Self {
        let cursor = self.cursor.saturating_add_signed(offset);
        self.move_cursor_at(cursor)
    }

    fn ensure_selection(mut self) -> Self {
        if self.selection.is_none() {
            self.selection = Some(Selection {
                range: self.cursor..=self.cursor,
            });
        }
        self
    }

    fn move_selection(mut self, offset: i16) -> Self {
        let Some(ref mut sel) = self.selection else {
            return self;
        };
        let cursor = self.cursor;
        self.update_selection(cursor.saturating_add_signed(offset))
    }

    fn update_selection(mut self, x: u16) -> Self {
        match self.rect() {
            None => self,
            Some(rect) => {
                if let Some(ref mut sel) = self.selection {
                    let x = x.saturating_sub(rect.x + 2);
                    if &x < sel.range.start() {
                        sel.range = x..=(*sel.range.end()).clamp(0, self.text.len() as u16);
                        self
                    } else {
                        sel.range = *sel.range.start()..=x.clamp(0, self.text.len() as u16);
                        self
                    }
                } else {
                    let x = x.saturating_sub(rect.x + 2);
                    self.selection = Some(Selection { range: x..=x });
                    self
                }
            }
        }
    }

    #[must_use]
    pub fn update(mut self, event: &Event) -> (Self, TextInputEvent) {
        if !self.focused.load(Ordering::Relaxed) {
            return self.no_event();
        }

        match (self.hit_test.get(), self.rect()) {
            (tea::observe::HitEvent::Drag(x, _), Some(rect)) => {
                self = self
                    .move_cursor_at(x.saturating_sub(rect.x + 2))
                    .update_selection(x);
            }
            (tea::observe::HitEvent::Clicked(x, _), Some(rect)) => {
                self.cursor = x.saturating_sub(rect.x + 2).min(self.text.len() as u16);
                self.selection = Some(Selection {
                    range: self.cursor..=self.cursor,
                });
            }
            (tea::observe::HitEvent::Clicked(x, _), None) => {
                self.selection = None;
            }
            _ => {}
        }

        match event {
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Left,
                kind: event::KeyEventKind::Press,
                modifiers,
                ..
            }) => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    self = self.ensure_selection().move_selection(-1);
                }
                self = self.move_cursor(-1);
                self.no_event()
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Right,
                kind: event::KeyEventKind::Press,
                modifiers,
                ..
            }) => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    self = self.ensure_selection().move_selection(1);
                }
                self = self.move_cursor(1);
                self.no_event()
            }

            keyv2!(enter) => self.no_selection().event(TextInputEvent::Confirm),

            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('c' | 'C'),
                kind: event::KeyEventKind::Press,
                modifiers,
                ..
            }) if modifiers.contains(KeyModifiers::SUPER | KeyModifiers::SHIFT)
                | modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.selection_to_clipboard();
                self.no_event()
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('x' | 'X'),
                kind: event::KeyEventKind::Press,
                modifiers,
                ..
            }) if modifiers.contains(KeyModifiers::SUPER | KeyModifiers::SHIFT)
                | modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.selection_to_clipboard();
                self.delete_selection().no_event()
            }
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
            }) => match self.selection {
                None => {
                    self.text.pop();
                    self.cursor = self.cursor.saturating_sub(1);
                    self.no_event()
                }
                Some(_) => self.delete_selection().no_event(),
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers,
                ..
            }) => {
                if self.selection.is_some() {
                    self = self.delete_selection().no_selection();
                }
                if !modifiers.difference(KeyModifiers::SHIFT).is_empty() {
                    return self.no_event();
                }
                match modifiers.contains(KeyModifiers::SHIFT) {
                    true => {
                        let value = ch.to_uppercase().to_string();
                        self.text.insert_str(self.cursor as usize, &value);
                    }
                    false => {
                        self.text.insert(self.cursor as usize, *ch);
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
        &[CTRL_KEYMAP, VIM_CTRL_KEYMAP]
    }

    fn hit_test(&self) -> manatui::tea::observe::HitEvent {
        self.hit_test.get()
    }
}
