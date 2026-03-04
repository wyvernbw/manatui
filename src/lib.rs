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
    pub use manatui_utils::key;
    pub use ratatui;

    #[cfg(feature = "macros")]
    pub use manatui_macros::*;
    #[cfg(feature = "macros")]
    pub extern crate bon;

    pub use ratatui::style::palette;
    pub use ratatui::style::*;
}

mod tests;
