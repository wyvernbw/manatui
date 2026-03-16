use ratatui::style::Color;

use crate::{Ctx, backends::ManaBackend};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CursorAt(pub u16, pub u16);

impl<B: ManaBackend> Ctx<B> {
    pub(crate) fn query_and_show_cursor(&mut self) -> Result<(), B::Error> {
        self.terminal.hide_cursor()?;
        for cursor in self.el_ctx.query_mut::<&CursorAt>().into_iter().take(1) {
            self.terminal.set_cursor_position((cursor.0, cursor.1))?;
            self.terminal.show_cursor()?;
        }
        Ok(())
    }
}
