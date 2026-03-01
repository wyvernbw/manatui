#![doc = include_str!("../readme.md")]
#![forbid(missing_docs)]
#![cfg_attr(feature = "nightly", feature(trait_alias))]

extern crate self as mana_tui_elemental;

pub mod layout;
pub mod prelude;
pub mod ui;
