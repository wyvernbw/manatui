//! [`manatui-layout`]: manatui_layout
//!
#![doc = include_str!("../readme.md")]

extern crate self as manatui;

pub use manatui_layout as layout;
#[cfg(feature = "macros")]
pub use manatui_macros as macros;
pub use manatui_tea as tea;
pub use manatui_utils as utils;
pub use ratatui;

pub mod prelude {
    pub use manatui_layout::prelude::*;

    #[cfg(feature = "macros")]
    pub use manatui_macros::*;
    #[cfg(feature = "macros")]
    pub extern crate bon;

    pub use ratatui::style::palette;
    pub use ratatui::style::*;
}

mod tests;

// TODO: lock behind crossterm feature
#[macro_export]
macro_rules! key {
    // Variant with no arguments, e.g. Enter, Esc
    ($code:ident, $kind:ident) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code,
            kind: ratatui::crossterm::event::KeyEventKind::$kind,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        }
    };
    // Variant with arguments, e.g. Char('x'), Char(_)
    ($code:ident ( $($arg:tt)* ), $kind:ident) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code($($arg)*),
            kind: ratatui::crossterm::event::KeyEventKind::$kind,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            ..
        }
    };
    // Variant with no arguments and optional modifiers
    ($code:ident, $kind:ident, $mods:expr ) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code,
            kind: ratatui::crossterm::event::KeyEventKind::$kind,
            modifiers: $mods,
            ..
        }
    };
    ($code:ident ( $($arg:tt)* ), $kind:ident, $mods:pat) => {
        ratatui::crossterm::event::KeyEvent {
            code: ratatui::crossterm::event::KeyCode::$code($($arg)*),
            kind: ratatui::crossterm::event::KeyEventKind::$kind,
            modifiers: $mods,
            ..
        }
    };
}
