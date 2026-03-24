use std::sync::atomic::{AtomicBool, Ordering};

use manatui::ratatui::crossterm::event::{self, Event};
use manatui::ratatui::layout::Rect;
use manatui::tea::focus::{DEFAULT_KEYMAP, KeyMap};
use manatui::tea::observe::HitEvent;
use manatui::tea::{
    focus::Focus,
    observe::{AreaRef, HitTest},
};

#[derive(Debug, Default)]
pub struct FocusItemState {
    pub focused: AtomicBool,
    pub area_ref: AreaRef,
    pub hit_test: HitTest,
}

impl Focus for FocusItemState {
    fn set_focus(&self, value: bool) {
        self.focused.store(value, Ordering::Relaxed);
    }

    fn focus(&self) -> bool {
        self.focused.load(Ordering::Relaxed)
    }

    fn rect(&self) -> Option<Rect> {
        self.area_ref.get()
    }

    fn keymaps(&self) -> &'static [KeyMap] {
        &[DEFAULT_KEYMAP]
    }

    fn hit_test(&self) -> HitEvent {
        self.hit_test.get()
    }
}
