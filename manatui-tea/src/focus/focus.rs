use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam::atomic::AtomicCell;
use ratatui::crossterm::event::{Event, KeyModifiers};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;

use crate::observe::HitEvent;

pub trait Focus {
    fn set_focus(&self, value: bool);
    fn focus(&self) -> bool;
    fn rect(&self) -> Option<Rect>;
    fn keymaps(&self) -> &'static [KeyMap];
    fn hit_test(&self) -> HitEvent;
}

#[derive(Debug)]
pub struct FocusGroup<T: Copy = ()> {
    index: AtomicUsize,
    tag: AtomicCell<Option<T>>,
    len: AtomicUsize,
    current_keymaps: AtomicCell<&'static [KeyMap]>,
    transitions: AtomicCell<Transitions>,
}

impl<T: Copy> Default for FocusGroup<T> {
    fn default() -> Self {
        Self {
            index: Default::default(),
            tag: Default::default(),
            len: Default::default(),
            current_keymaps: Default::default(),
            transitions: Default::default(),
        }
    }
}

pub struct FocusGroupItems<'a, T: Copy = ()> {
    group: &'a FocusGroup<T>,
    items: Vec<(&'a dyn Focus, T)>,
    index: usize,
}

macro_rules! match_key {
    ($keymaps:expr,$key:ident,$event:ident) => {
        $keymaps
            .load()
            .iter()
            .flat_map(|map| map.$key)
            .any(|key| key.eq(&$event))
    };
}

pub enum EventOutcome<T> {
    Consumed(T),
    Unhandled(T),
}

impl<T> EventOutcome<T> {
    pub fn into_inner(self) -> T {
        match self {
            EventOutcome::Consumed(value) | EventOutcome::Unhandled(value) => value,
        }
    }
}

impl<T: Copy> FocusGroup<T> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(&self) -> FocusGroupItems<'_, T> {
        FocusGroupItems {
            items: vec![],
            index: self.index.load(Ordering::Relaxed),
            group: self,
        }
    }

    #[must_use]
    pub fn focus_next(self) -> Self {
        let idx = self.index.load(Ordering::Relaxed);
        let idx = idx + 1;
        let idx = idx.clamp(0, self.len.load(Ordering::Relaxed).saturating_sub(1));
        self.index.store(idx, Ordering::Relaxed);
        self
    }

    #[must_use]
    pub fn focus_prev(self) -> Self {
        let index = self.index.load(Ordering::Relaxed);
        let index = index.saturating_sub(1);
        let index = index.clamp(0, self.len.load(Ordering::Relaxed).saturating_sub(1));
        self.index.store(index, Ordering::Relaxed);
        self
    }

    #[must_use]
    pub fn focus_at(self, index: usize) -> Self {
        let index = index.clamp(0, self.len.load(Ordering::Relaxed).saturating_sub(1));
        self.index.store(index, Ordering::Relaxed);
        self
    }

    #[must_use]
    pub fn update(self, event: &Event) -> EventOutcome<Self> {
        if let Some(event) = event.as_key_event() {
            let transitions = self.transitions.load();
            let is_next = match_key!(self.current_keymaps, next, event);
            if is_next {
                return EventOutcome::Consumed(self.focus_next());
            }
            let is_prev = match_key!(self.current_keymaps, prev, event);
            if is_prev {
                return EventOutcome::Consumed(self.focus_prev());
            }
            let is_down = match_key!(self.current_keymaps, down, event);
            if is_down {
                let next = transitions.down;
                return EventOutcome::Consumed(self.focus_at(next));
            }
            let is_right = match_key!(self.current_keymaps, right, event);
            if is_right {
                let next = transitions.right;
                return EventOutcome::Consumed(self.focus_at(next));
            }
            let is_left = match_key!(self.current_keymaps, left, event);
            if is_left {
                let next = transitions.left;
                return EventOutcome::Consumed(self.focus_at(next));
            }
            let is_up = match_key!(self.current_keymaps, up, event);
            if is_up {
                let next = transitions.up;
                return EventOutcome::Consumed(self.focus_at(next));
            }
        }
        EventOutcome::Unhandled(self)
    }

    #[must_use]
    pub fn handle_event(self, event: impl Into<FocusEvent>) -> Self {
        let effect = event.into();
        match effect {
            FocusEvent::None => self,
            FocusEvent::Next => self.focus_next(),
            FocusEvent::Prev => self.focus_prev(),
        }
    }

    #[must_use]
    pub fn pipe<U>(self, value: (U, impl Into<FocusEvent>)) -> (U, Self) {
        (value.0, self.handle_event(value.1))
    }
}

impl<'a> FocusGroupItems<'a, ()> {
    #[must_use]
    pub fn next_untagged(self, item: &'a impl Focus) -> Self {
        self.next_dyn_untagged(item as _)
    }
    #[must_use]
    pub fn next_dyn_untagged(mut self, item: &'a dyn Focus) -> Self {
        self.items.push((item, ()));
        self
    }
}

impl<'a, T: Copy> FocusGroupItems<'a, T> {
    #[must_use]
    pub fn next(self, (item, tag): (&'a impl Focus, T)) -> Self {
        self.next_dyn((item as _, tag))
    }

    #[must_use]
    pub fn next_dyn(mut self, (item, tag): (&'a dyn Focus, T)) -> Self {
        self.items.push((item, tag));
        self
    }

    pub fn commit(mut self) {
        for (i, (item, _)) in self.items.iter().enumerate() {
            if let HitEvent::Clicked(_, _) = item.hit_test() {
                self.index = i;
                self.group.index.store(self.index, Ordering::Relaxed);
                break;
            }
        }

        self.group.len.store(self.items.len(), Ordering::Relaxed);
        self.group
            .current_keymaps
            .store(self.items[self.index].0.keymaps());
        let mut group_transitions = Transitions {
            down: self.index,
            up: self.index,
            left: self.index,
            right: self.index,
        };
        self.group.transitions.store(group_transitions);

        let mut distances = Transitions::splat(usize::MAX);

        let focused_rect = self.items[self.index].0.rect();

        for (i, item) in self.items.iter_mut().enumerate() {
            if i == self.index {
                continue;
            }

            item.0.set_focus(false);

            let Some(focused_rect) = focused_rect else {
                continue;
            };
            let Some(rect) = item.0.rect() else {
                continue;
            };
            let pos = rect.as_position();
            let h_distance = pos.x.abs_diff(focused_rect.x);
            let v_distance = pos.y.abs_diff(focused_rect.y);

            if v_distance > h_distance {
                if pos.y > focused_rect.y && distances.down > v_distance as usize {
                    distances.down = v_distance as usize;
                    group_transitions.down = i;
                }
                if pos.y < focused_rect.y && distances.up > v_distance as usize {
                    distances.up = v_distance as usize;
                    group_transitions.up = i;
                }
            } else {
                if pos.x > focused_rect.x && distances.right > h_distance as usize {
                    distances.right = h_distance as usize;
                    group_transitions.right = i;
                }
                if pos.x < focused_rect.x && distances.left > h_distance as usize {
                    distances.left = h_distance as usize;
                    group_transitions.left = i;
                }
            }
        }
        self.group.transitions.store(group_transitions);
        self.group.tag.store(Some(self.items[self.index].1));
        self.items[self.index].0.set_focus(true);
    }
}

#[derive(Default, Clone, Debug)]
pub struct KeyMap {
    pub next: Option<KeyEvent>,
    pub prev: Option<KeyEvent>,
    pub down: Option<KeyEvent>,
    pub up: Option<KeyEvent>,
    pub left: Option<KeyEvent>,
    pub right: Option<KeyEvent>,
}

#[derive(Default, Clone, Debug, Copy)]
struct Transitions {
    down: usize,
    up: usize,
    left: usize,
    right: usize,
}

impl Transitions {
    const fn splat(value: usize) -> Self {
        Transitions {
            down: value,
            up: value,
            left: value,
            right: value,
        }
    }
}

pub const DEFAULT_KEYMAP: KeyMap = KeyMap {
    next: Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
    prev: Some(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
    down: Some(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
    up: Some(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
    left: Some(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
    right: Some(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
};

pub const CYCLE_KEYMAP: KeyMap = KeyMap {
    next: Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
    prev: Some(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
    down: None,
    up: None,
    left: None,
    right: None,
};

pub const CTRL_KEYMAP: KeyMap = KeyMap {
    next: Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
    prev: Some(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
    down: Some(KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL)),
    up: Some(KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL)),
    left: Some(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
    right: Some(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
};

pub const VIM_KEYMAP: KeyMap = KeyMap {
    next: Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
    prev: Some(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
    down: Some(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
    up: Some(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
    left: Some(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE)),
    right: Some(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
};

pub const VIM_CTRL_KEYMAP: KeyMap = KeyMap {
    next: Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
    prev: Some(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
    down: Some(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL)),
    up: Some(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)),
    left: Some(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL)),
    right: Some(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)),
};

pub const VIM_CTRL_KEYMAP_NO_CYCLE: KeyMap = KeyMap {
    next: None,
    prev: None,
    down: Some(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL)),
    up: Some(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)),
    left: Some(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL)),
    right: Some(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)),
};

#[derive(Default, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum FocusEvent {
    #[default]
    None,
    Next,
    Prev,
}
