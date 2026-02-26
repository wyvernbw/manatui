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
