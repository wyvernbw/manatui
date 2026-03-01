#![doc = include_str!("../readme.md")]
#![forbid(missing_docs)]
#![cfg_attr(feature = "nightly", feature(trait_alias))]

extern crate self as manatui_elemental;

pub mod layout;
pub mod prelude;
pub mod ui;
