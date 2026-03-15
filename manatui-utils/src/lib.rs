use std::ops::{Deref, DerefMut};

use hecs::World;

pub mod ext;
pub mod resource;
pub mod systems;

pub trait Ecs: Deref<Target = World> {}
pub trait EcsMut: DerefMut<Target = World> {}

impl Ecs for &mut World {}
impl Ecs for &World {}

impl EcsMut for &mut World {}

// TODO: lock behind crossterm feature
#[macro_export]
macro_rules! key {
    // Variant with no arguments, e.g. Enter, Esc
    ($code:ident) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    // Variant with arguments, e.g. Char('x'), Char(_)
    ($code:ident ( $($arg:tt)* )) => {
        ratatui::crossterm::event::Event::Key(
            ratatui::crossterm::event::KeyEvent {
                code: ratatui::crossterm::event::KeyCode::$code($($arg)*),
                kind: ratatui::crossterm::event::KeyEventKind::Press,
                modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
                ..
            }
        )
    };

    // Variant with no arguments and optional modifiers
    ($code:ident, $mods:pat ) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: $mods,
            ..
        })
    };
    ($code:ident ( $($arg:tt)* ), $mods:pat) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code($($arg)*),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: $mods,
            ..
        })
    };
}

#[macro_export]
macro_rules! keyv2 {
    // Modified chars
    (ctrl+$c:literal) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::CONTROL,
            ..
        })
    };
    (shift+$c:literal) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::SHIFT,
            ..
        })
    };
    (alt+$c:literal) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::ALT,
            ..
        })
    };
    (ctrl+shift+$c:literal) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::CONTROL
                | ratatui::crossterm::event::KeyModifiers::SHIFT,
            ..
        })
    };
    (ctrl+alt+$c:literal) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::CONTROL
                | ratatui::crossterm::event::KeyModifiers::ALT,
            ..
        })
    };

    // Named keys (no modifier)
    (enter) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Enter,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (space) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char(' '),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (backspace) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Backspace,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (delete) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Delete,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (esc) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Esc,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (tab) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Tab,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (backtab) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::BackTab,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (up) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Up,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (down) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Down,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (left) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Left,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (right) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Right,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (home) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Home,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (end) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::End,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (pageup) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::PageUp,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (pagedown) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::PageDown,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
    (insert) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Insert,
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };

    // Function keys
    (f($n:literal)) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::F($n),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };

    // Plain char fallthrough
    ($c:literal) => {
        ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::Char($c),
            kind: ratatui::crossterm::event::KeyEventKind::Press,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        })
    };
}
