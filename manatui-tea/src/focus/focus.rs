use ratatui::crossterm::event::{Event, KeyModifiers};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;

pub trait Focus {
    fn set_focus(&mut self, value: bool);
    fn focus(&self) -> bool;
    fn rect(&self) -> Option<Rect>;
    fn keymaps(&self) -> &'static [KeyMap];
}

#[derive(Default, Clone, Debug)]
pub struct FocusGroup {
    index: usize,
    len: usize,
    current_keymaps: &'static [KeyMap],
    transitions: Transitions,
}

pub struct FocusGroupItems<'a> {
    group: &'a mut FocusGroup,
    items: Vec<&'a mut dyn Focus>,
    index: usize,
}

macro_rules! match_key {
    ($keymaps:expr,$key:ident,$event:ident) => {
        $keymaps
            .iter()
            .flat_map(|map| map.$key)
            .any(|key| key.eq(&$event))
    };
}

pub enum EventOutcome<T> {
    Consumed(T),
    Unhandled(T),
}

impl FocusGroup {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items<'a>(&'a mut self, first: &'a mut impl Focus) -> FocusGroupItems<'a> {
        FocusGroupItems {
            items: vec![first as &'a mut dyn Focus],
            index: self.index,
            group: self,
        }
    }

    #[must_use]
    pub fn focus_next(mut self) -> Self {
        self.index += 1;
        self.index = self.index.clamp(0, self.len - 1);
        self
    }

    #[must_use]
    pub fn focus_prev(mut self) -> Self {
        self.index = self.index.saturating_sub(1);
        self.index = self.index.clamp(0, self.len - 1);
        self
    }

    #[must_use]
    pub fn focus_at(mut self, index: usize) -> Self {
        self.index = index;
        self.index = self.index.clamp(0, self.len - 1);
        self
    }

    #[must_use]
    pub fn update(self, event: &Event) -> EventOutcome<Self> {
        if let Some(event) = event.as_key_event() {
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
                let next = self.transitions.down;
                return EventOutcome::Consumed(self.focus_at(next));
            }
            let is_right = match_key!(self.current_keymaps, right, event);
            if is_right {
                let next = self.transitions.right;
                return EventOutcome::Consumed(self.focus_at(next));
            }
            let is_left = match_key!(self.current_keymaps, left, event);
            if is_left {
                let next = self.transitions.left;
                return EventOutcome::Consumed(self.focus_at(next));
            }
            let is_up = match_key!(self.current_keymaps, up, event);
            if is_up {
                let next = self.transitions.up;
                return EventOutcome::Consumed(self.focus_at(next));
            }
        }
        EventOutcome::Unhandled(self)
    }
}

impl<'a> FocusGroupItems<'a> {
    #[must_use]
    pub fn next(mut self, item: &'a mut impl Focus) -> Self {
        self.items.push(item as &'a mut dyn Focus);
        self
    }

    pub fn commit(mut self) {
        self.group.len = self.items.len();
        self.group.current_keymaps = self.items[self.index].keymaps();
        self.group.transitions = Transitions {
            down: self.index,
            up: self.index,
            left: self.index,
            right: self.index,
        };

        let mut distances = Transitions::splat(usize::MAX);

        let focused_rect = self.items[self.index].rect();

        for (i, item) in self.items.iter_mut().enumerate() {
            if i == self.index {
                continue;
            }

            item.set_focus(false);

            let Some(focused_rect) = focused_rect else {
                continue;
            };
            let Some(rect) = item.rect() else {
                continue;
            };
            let pos = rect.as_position();
            let h_distance = pos.x.abs_diff(focused_rect.x);
            let v_distance = pos.y.abs_diff(focused_rect.y);

            if v_distance > h_distance {
                if pos.y > focused_rect.y && distances.down > v_distance as usize {
                    distances.down = v_distance as usize;
                    self.group.transitions.down = i;
                }
                if pos.y < focused_rect.y && distances.up > v_distance as usize {
                    distances.up = v_distance as usize;
                    self.group.transitions.up = i;
                }
            } else {
                if pos.x > focused_rect.x && distances.right > h_distance as usize {
                    distances.right = h_distance as usize;
                    self.group.transitions.right = i;
                }
                if pos.x < focused_rect.x && distances.left > h_distance as usize {
                    distances.left = h_distance as usize;
                    self.group.transitions.left = i;
                }
            }
        }
        self.items[self.index].set_focus(true);
    }
}

#[derive(Default, Clone, Debug)]
pub struct KeyMap {
    next: Option<KeyEvent>,
    prev: Option<KeyEvent>,
    down: Option<KeyEvent>,
    up: Option<KeyEvent>,
    left: Option<KeyEvent>,
    right: Option<KeyEvent>,
}

#[derive(Default, Clone, Debug)]
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
