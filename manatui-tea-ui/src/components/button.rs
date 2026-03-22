use manatui::prelude::*;
use manatui::ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};
use manatui::tea::focus::{CTRL_KEYMAP, DEFAULT_KEYMAP, Focus, VIM_CTRL_KEYMAP, VIM_KEYMAP};
use manatui::tea::observe::HitEvent;
use manatui::utils::keyv2;

use crate::common::FocusItemState;

#[derive(Debug, Default)]
pub struct Button {
    focus_item: FocusItemState,
    btn_down: Option<MouseButton>,
    key_down: Option<KeyCode>,
}

impl Focus for Button {
    fn set_focus(&self, value: bool) {
        self.focus_item.set_focus(value);
    }

    fn focus(&self) -> bool {
        self.focus_item.focus()
    }

    fn rect(&self) -> Option<manatui::ratatui::prelude::Rect> {
        self.focus_item.rect()
    }

    fn keymaps(&self) -> &'static [manatui::tea::focus::KeyMap] {
        &[VIM_KEYMAP, DEFAULT_KEYMAP, VIM_CTRL_KEYMAP, CTRL_KEYMAP]
    }

    fn hit_test(&self) -> manatui::tea::observe::HitEvent {
        self.focus_item.hit_test()
    }
}

pub enum ButtonEvent {
    Clicked,
    None,
}

impl Button {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    fn with_event(self, ev: ButtonEvent) -> (Self, ButtonEvent) {
        (self, ev)
    }
    fn no_event(self) -> (Self, ButtonEvent) {
        self.with_event(ButtonEvent::None)
    }
    #[must_use]
    pub fn update(mut self, event: &Event) -> (Self, ButtonEvent) {
        if !self.focus() {
            self.btn_down = None;
            self.key_down = None;
            return self.no_event();
        }

        match event {
            Event::Key(KeyEvent {
                code: code @ KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.key_down = Some(*code);
                self.with_event(ButtonEvent::Clicked)
            }
            Event::Key(KeyEvent {
                code: code @ KeyCode::Enter,
                kind: KeyEventKind::Release,
                ..
            }) => {
                if Some(*code) == self.key_down {
                    self.key_down = None;
                }
                self.with_event(ButtonEvent::Clicked)
            }
            Event::Mouse(mouse) => {
                let pos = ratatui::layout::Position::new(mouse.column, mouse.row);
                let Some(rect) = self.rect() else {
                    return self.no_event();
                };
                if !rect.contains(pos) {
                    return self.no_event();
                }
                match mouse.kind {
                    MouseEventKind::Down(btn) => {
                        self.btn_down = Some(btn);
                        self.with_event(ButtonEvent::Clicked)
                    }
                    MouseEventKind::Up(btn) => {
                        if Some(btn) == self.btn_down {
                            self.btn_down = None;
                        }
                        self.no_event()
                    }
                    _ => self.no_event(),
                }
            }
            _ => self.no_event(),
        }
    }
    pub fn is_down(&self) -> bool {
        (self.key_down.is_some() || self.btn_down.is_some()) && self.focus()
    }
}

#[subview]
pub fn button_view(
    state: &Button,
    hover_style: Option<Style>,
    clicked_style: Option<Style>,
) -> View {
    let hover_style = hover_style.unwrap_or_else(|| Style::new().bold());
    let clicked_style = clicked_style.unwrap_or_else(|| Style::new().dim());
    let is_hover = matches!(state.hit_test(), HitEvent::Hovered(_, _));
    let is_down = state.is_down();
    let style = match (is_hover || state.focus(), is_down) {
        (_, true) => clicked_style,
        (true, false) => hover_style,
        (false, false) => Style::default(),
    };
    ui! {
        <Block
            .style={style}
            {state.focus_item.hit_test.clone()}
            {state.focus_item.area_ref.clone()}
        >
        </Block>
    }
}
