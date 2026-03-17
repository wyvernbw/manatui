#![allow(clippy::match_bool)]

use std::sync::{LazyLock, Mutex};

use arboard::Clipboard;

pub mod common;
#[path = "./components/components.rs"]
pub mod components;

static CLIPBOARD: LazyLock<Mutex<Clipboard>> =
    LazyLock::new(|| Mutex::new(Clipboard::new().expect("failed to init system clipboard!")));
