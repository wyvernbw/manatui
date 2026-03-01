#![cfg(test)]

use criterion::*;
use manatui::prelude::*;
use manatui_macros::ui;
use ratatui::prelude::*;

#[inline]
pub fn basic_render() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="parent" Width::grow() Height::grow() Direction::Horizontal>
            <Block .rounded .title_top="child" Width::percentage(70) MaxWidth::fixed(8) Height::grow()>
                "hello"
            </Block>
        </Block>
    };
    let root = ctx.spawn_ui(root);

    let mut buf = Buffer::empty(Rect::new(0, 0, 28, 5));

    ctx.calculate_layout(root, buf.area).unwrap();
    ctx.render(root, buf.area, &mut buf);

    assert_eq!(
        buf,
        Buffer::with_lines([
            "╭parent────────────────────╮",
            "│╭child─╮                  │",
            "││hello │                  │",
            "│╰──────╯                  │",
            "╰──────────────────────────╯",
        ])
    );
}
