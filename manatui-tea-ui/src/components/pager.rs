use std::sync::{Arc, RwLock};

use manatui::prelude::*;
use manatui::ratatui::crossterm::event::Event;
use manatui::tea::focus::{CTRL_KEYMAP, Focus, VIM_CTRL_KEYMAP};
use manatui::utils::keyv2;

use crate::common::FocusItemState;

#[derive(Debug, Default)]
pub struct Pager {
    focus: FocusItemState,
    inner_state: Arc<RwLock<ScrollViewState>>,
}

impl Focus for Pager {
    fn set_focus(&self, value: bool) {
        self.focus.set_focus(value);
    }

    fn focus(&self) -> bool {
        self.focus.focus()
    }

    fn rect(&self) -> Option<manatui::ratatui::prelude::Rect> {
        self.focus.rect()
    }

    fn keymaps(&self) -> &'static [manatui::tea::focus::KeyMap] {
        &[VIM_CTRL_KEYMAP, CTRL_KEYMAP]
    }

    fn hit_test(&self) -> manatui::tea::observe::HitEvent {
        self.focus.hit_test()
    }
}

impl Pager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn scroll_down_by(self, scroll: usize) -> Self {
        {
            let mut state = self.inner_state.write().unwrap();
            for _ in 0..scroll {
                state.scroll_down();
            }
        }
        self
    }

    #[must_use]
    pub fn scroll_up_by(self, scroll: usize) -> Self {
        {
            let mut state = self.inner_state.write().unwrap();
            for _ in 0..scroll {
                state.scroll_up();
            }
        }
        self
    }

    #[must_use]
    pub fn scroll_down(self) -> Self {
        self.scroll_down_by(1)
    }

    #[must_use]
    pub fn scroll_up(self) -> Self {
        self.scroll_up_by(1)
    }

    #[must_use]
    pub fn update(self, event: &Event) -> Self {
        if !self.focus() {
            return self;
        }
        match event {
            keyv2!('j') | keyv2!(down) => self.scroll_down(),
            keyv2!('k') | keyv2!(up) => self.scroll_up(),
            keyv2!('h') | keyv2!(left) => {
                self.inner_state.write().unwrap().scroll_left();
                self
            }
            keyv2!('l') | keyv2!(right) => {
                self.inner_state.write().unwrap().scroll_right();
                self
            }
            keyv2!('g') => {
                self.inner_state.write().unwrap().scroll_to_top();
                self
            }
            keyv2!(shift + 'G') => {
                self.inner_state.write().unwrap().scroll_to_bottom();
                self
            }
            _ => self,
        }
    }
}

#[subview]
pub fn pager_view(state: &Pager, content: View) -> View {
    ui! {
        <Block>
            <Block
                ScrollView::default()
                ScrollbarVisibility::Never
                Width::grow() Height::grow()
                {state.inner_state.clone()}
                {state.focus.area_ref.clone()}
                {state.focus.hit_test.clone()}
            >
                {content}
            </Block>
        </Block>
    }
}
