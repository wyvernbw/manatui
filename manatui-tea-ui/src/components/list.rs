use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use manatui::prelude::*;
use manatui::ratatui::crossterm::event;
use manatui::ratatui::text::{Line, ToText};
use manatui::tea::focus::{
    CYCLE_KEYMAP, Focus, FocusEvent, KeyMap, VIM_CTRL_KEYMAP, VIM_CTRL_KEYMAP_NO_CYCLE,
};
use manatui::tea::observe::{AreaRef, HitTest};
use manatui::{ratatui::crossterm::event::Event, utils::keyv2};

#[derive(Debug)]
pub struct List {
    index: Option<usize>,
    len: AtomicUsize,
    focused: Arc<AtomicBool>,
    area_ref: AreaRef,
    hit_test: HitTest,
}

impl Default for List {
    fn default() -> Self {
        Self {
            index: Some(0),
            len: AtomicUsize::new(0),
            focused: Arc::default(),
            area_ref: AreaRef::empty(),
            hit_test: HitTest::empty(),
        }
    }
}

impl List {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn select(mut self, idx: Option<usize>) -> Self {
        self.index = idx;
        self
    }
    #[must_use]
    pub fn contains(&self, idx: usize) -> bool {
        (0..self.len.load(Ordering::Relaxed)).contains(&idx)
    }
    #[must_use]
    pub fn select_offset(self, offset: isize) -> Self {
        let new_idx = self.index.map(|idx| {
            idx.saturating_add_signed(offset)
                .clamp(0, self.len.load(Ordering::Relaxed) - 1)
        });
        self.select(new_idx)
    }
    #[must_use]
    pub fn select_next(self) -> Self {
        self.select_offset(1)
    }
    #[must_use]
    pub fn select_prev(self) -> Self {
        self.select_offset(-1)
    }
    #[must_use]
    pub fn select_first(mut self) -> Self {
        self.index = Some(0);
        self
    }
    #[must_use]
    pub fn select_last(self) -> Self {
        self.select_first().select_offset(isize::MAX)
    }
    #[must_use]
    pub fn update(mut self, event: &Event) -> (Self, ListEvent) {
        if !self.focus() {
            return self.no_event();
        }

        if self.index.is_none() {
            self = self.select_first();
        }

        match event {
            keyv2!('j') | keyv2!(down) | keyv2!(tab)
                if self.index == Some(self.len.load(Ordering::Relaxed) - 1) =>
            {
                (self, ListEvent::FocusNext)
            }
            keyv2!('k')
            | keyv2!(up)
            | event::Event::Key(event::KeyEvent {
                code: event::KeyCode::BackTab,
                kind: event::KeyEventKind::Press,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            }) if self.index == Some(0) => (self, ListEvent::FocusPrev),
            keyv2!('j') | keyv2!(down) | keyv2!(tab) => self.select_next().no_event(),
            keyv2!('k')
            | keyv2!(up)
            | event::Event::Key(event::KeyEvent {
                code: event::KeyCode::BackTab,
                kind: event::KeyEventKind::Press,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            }) => self.select_prev().no_event(),
            keyv2!('g') => self.select_first().no_event(),
            keyv2!(shift + 'g') => self.select_last().no_event(),
            _ => self.no_event(),
        }
    }
    fn event(self, event: ListEvent) -> (Self, ListEvent) {
        (self, event)
    }
    fn no_event(self) -> (Self, ListEvent) {
        self.event(ListEvent::None)
    }
}

pub enum ListEvent {
    None,
    FocusNext,
    FocusPrev,
}

impl From<ListEvent> for FocusEvent {
    fn from(value: ListEvent) -> Self {
        match value {
            ListEvent::FocusNext => FocusEvent::Next,
            ListEvent::FocusPrev => FocusEvent::Prev,
            ListEvent::None => FocusEvent::None,
        }
    }
}

impl Focus for List {
    fn set_focus(&self, value: bool) {
        self.focused.store(value, Ordering::Relaxed);
    }

    fn focus(&self) -> bool {
        self.focused.load(Ordering::Relaxed)
    }

    fn rect(&self) -> Option<ratatui::prelude::Rect> {
        self.area_ref.get()
    }

    fn keymaps(&self) -> &'static [manatui::tea::focus::KeyMap] {
        &[VIM_CTRL_KEYMAP_NO_CYCLE]
    }

    fn hit_test(&self) -> manatui::tea::observe::HitEvent {
        self.hit_test.get()
    }
}

#[subview]
pub fn list_view_compact<I: Into<ListItem<'static>>>(
    state: &List,
    items: impl Iterator<Item = I>,
) -> View {
    let len = items.size_hint().1.unwrap_or(usize::MAX);
    state.len.store(len, Ordering::Relaxed);
    let selected = state.focus().then_some(state.index).flatten();
    let items = items
        .enumerate()
        .map(|(idx, line)| {
            let line = line.into();
            let style = if Some(idx) == selected {
                Style::new().white().bg(Color::from_u32(0x843fec))
            } else {
                Style::default()
            };
            line.style(style)
        })
        .collect::<Vec<_>>();
    let height: usize = items.iter().map(manatui::prelude::ListItem::height).sum();
    let width = items
        .iter()
        .map(manatui::prelude::ListItem::width)
        .max()
        .unwrap_or(0);
    ui(ratatui::widgets::List::new(items))
        .with((
            Direction::Vertical,
            Height::fixed(height as u16),
            Width::fixed(width as u16),
            state.area_ref.clone(),
        ))
        .done()
}
