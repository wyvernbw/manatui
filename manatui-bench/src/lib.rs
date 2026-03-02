use manatui::{prelude::*, ratatui};
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

    debug_assert_eq!(
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

#[inline]
#[test]
pub fn complex_render() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="root" Width::grow() Height::grow() Direction::Horizontal Gap(1)>
            <Block .rounded .title_top="sidebar" Width::percentage(25) Height::grow() Direction::Vertical Gap(1)>
                <Block .rounded .title_top="nav" Width::grow() Height::percentage(50)>
                    "item 1"
                    "item 2"
                    "item 3"
                    "item 4"
                </Block>
                <Block .rounded .title_top="info" Width::grow() Height::grow()>
                    "status: ok"
                    "mem: 128mb"
                </Block>
            </Block>
            <Block .rounded .title_top="main" Width::grow() Height::grow() Direction::Vertical Gap(1)>
                <Block .rounded .title_top="toolbar" Width::grow() Height::fixed(5) Direction::Horizontal Gap(2)>
                    <Block .rounded Width::fixed(12) Height::grow()>"file"</Block>
                    <Block .rounded Width::fixed(12) Height::grow()>"edit"</Block>
                    <Block .rounded Width::fixed(12) Height::grow()>"view"</Block>
                    <Block .rounded Width::fixed(12) Height::grow()>"help"</Block>
                </Block>
                <Block .rounded .title_top="content" Width::grow() Height::grow() Direction::Horizontal Gap(1)>
                    <Block .rounded .title_top="left" Width::percentage(50) Height::grow() Direction::Vertical Gap(1)>
                        <Block .rounded Width::grow() Height::percentage(50)>
                            "lorem ipsum dolor sit amet"
                            "consectetur adipiscing elit"
                            "sed do eiusmod tempor"
                        </Block>
                        <Block .rounded Width::grow() Height::grow()>
                            "more content here"
                            "and here"
                        </Block>
                    </Block>
                    <Block .rounded .title_top="right" Width::grow() Height::grow() Direction::Vertical Gap(1)>
                        <Block .rounded Width::grow() Height::percentage(33)>"panel a"</Block>
                        <Block .rounded Width::grow() Height::percentage(33)>"panel b"</Block>
                        <Block .rounded Width::grow() Height::grow()>"panel c"</Block>
                    </Block>
                </Block>
                <Block .rounded .title_top="statusbar" Width::grow() Height::fixed(5) Direction::Horizontal Gap(1)>
                    <Block .rounded Width::percentage(50) Height::grow()>"left status"</Block>
                    <Block .rounded Width::grow() Height::grow()>"right status"</Block>
                </Block>
            </Block>
        </Block>
    };

    let root = ctx.spawn_ui(root);
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));
    ctx.calculate_layout(root, buf.area).unwrap();
    ctx.render(root, buf.area, &mut buf);

    println!("{buf:?}");
}
