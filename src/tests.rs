#![cfg(test)]

use crate::prelude::*;
use hecs::World;
use manatui_layout::layout::NodePostRenderSchedule;
use manatui_macros::subview;
use manatui_macros::ui;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Wrap};
use strum::IntoEnumIterator;

use manatui_layout::layout::{ElementCtx, TuiElMarker};
use test_log::test;

fn buffer_to_string(buf: &Buffer) -> String {
    buf.content()
        .chunks(buf.area.width as usize)
        .flat_map(|line| line.iter().map(|cell| cell.symbol()).chain(["\n"]))
        .collect()
}

#[test]
fn test_fit_01() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="block" Direction::Horizontal>
            <Block .rounded/>
            <Block .rounded>
                "content"
            </Block>
        </Block>
    };
    let root = ctx.spawn_ui(root);

    let mut buf = Buffer::empty(Rect::new(0, 0, 28, 7));

    ctx.calculate_layout(root, buf.area).unwrap();
    ctx.render(root, buf.area, &mut buf);

    assert_eq!(
        buf,
        Buffer::with_lines([
            "╭block──────╮               ",
            "│╭╮╭───────╮│               ",
            "│╰╯│content││               ",
            "│  ╰───────╯│               ",
            "╰───────────╯               ",
            "                            ",
            "                            ",
        ])
    );
}

#[test]
fn test_grow_02() {
    let mut ctx = ElementCtx::new();
    let block = || Block::bordered().border_type(BorderType::Rounded);
    let root = ui(block().title_top("parent"))
        .with((
            Width::fixed(36),
            Height::fixed(18),
            Direction::Horizontal,
            Padding::uniform(1),
        ))
        .children((
            ui(block().title_top("sidebar"))
                .with((Width::fixed(10), Height::grow(), Padding::uniform(1)))
                .child(ui(Paragraph::new(
                    "this sidebar is so amazing it can have long text that wraps around",
                )
                .wrap(ratatui::widgets::Wrap { trim: false }))),
            ui(block().title_top("child #1"))
                .with((
                    Width::grow(),
                    Height::grow(),
                    Padding::uniform(1),
                    Gap(1),
                    Direction::Vertical,
                ))
                .children((
                    ui(block().title_top("child #2")).with((Width::grow(), Height::grow())),
                    ui(block().title_top("child #3")).with((Width::grow(), Height::grow())),
                )),
        ));
    let root = ctx.spawn_ui(root);

    let area = Rect::new(0, 0, 50, 24);
    ctx.calculate_layout(root, area).unwrap();
    let mut buf = Buffer::empty(area);
    ctx.render(root, buf.area, &mut buf);
}

#[test]
fn test_grow_03() {
    let root = ui! {
        <Block .rounded Width::grow() Height::grow()>
            <Block .rounded Width::grow() Height::grow()></Block>
        </Block>
    };
    let area = Rect::new(0, 0, 16, 8);
    let mut ctx = ElementCtx::new();
    let mut buf = Buffer::empty(area);

    let root = ctx.spawn_ui(root);

    ctx.calculate_layout(root, area).unwrap();
    ctx.render(root, area, &mut buf);

    assert_eq!(
        buf,
        Buffer::with_lines([
            "╭──────────────╮",
            "│╭────────────╮│",
            "││            ││",
            "││            ││",
            "││            ││",
            "││            ││",
            "│╰────────────╯│",
            "╰──────────────╯",
        ])
    );
}

#[test]
fn test_grow_04() {
    let root = ui! {
        <Block .rounded Width::grow() Height::grow() Direction::Horizontal>
            <Block .rounded Width::grow() Height::grow()/>
            <Block .rounded Width::grow() Height::grow()/>
        </Block>
    };
    let area = Rect::new(0, 0, 16, 5);
    let mut ctx = ElementCtx::new();
    let mut buf = Buffer::empty(area);

    let root = ctx.spawn_ui(root);

    ctx.calculate_layout(root, area).unwrap();
    ctx.render(root, area, &mut buf);

    assert_eq!(
        buf,
        Buffer::with_lines([
            "╭──────────────╮",
            "│╭─────╮╭─────╮│",
            "││     ││     ││",
            "│╰─────╯╰─────╯│",
            "╰──────────────╯",
        ])
    );
}

#[test]
fn test_gap() {
    _ = color_eyre::install();

    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="parent" Width::fit() Height::fit() Direction::Horizontal Gap(2)>
            <Block .rounded Width::fixed(4) Height::fixed(3)>
                "01"
            </Block>
            <Block .rounded Width::fixed(4) Height::fixed(3)>
                "02"
            </Block>
            <Block .rounded Width::fixed(4) Height::fixed(3)>
                "03"
            </Block>
        </Block>
    };
    let root = ctx.spawn_ui(root);

    let mut buf = Buffer::empty(Rect::new(0, 0, 18, 5));
    ctx.calculate_layout(root, buf.area).unwrap();
    ctx.render(root, buf.area, &mut buf);
    assert_eq!(
        buf,
        Buffer::with_lines([
            "╭parent──────────╮",
            "│╭──╮  ╭──╮  ╭──╮│",
            "││01│  │02│  │03││",
            "│╰──╯  ╰──╯  ╰──╯│",
            "╰────────────────╯",
        ])
    );
}

#[test]
fn test_grow_05() {
    let mut ctx = ElementCtx::new();

    #[subview]
    fn sidebar() -> View {
        let value = "i am formatted";
        ui! {
            <Block .rounded .title_top="sidebar" Width::fixed(10) Height::grow()>
                <Paragraph .wrap={Wrap::default()} Height::grow()>
                ""
                </Paragraph>
            </Block>
        }
    }

    let root = ui! {
        <Block
            .rounded .title_top="parent"
            Width::fixed(36) Height::fixed(18) Direction::Horizontal Padding::uniform(1)
        >
            <Sidebar />
            <Block .rounded .title_top="child #1"
                Width::grow() Height::grow() Padding::uniform(1) Gap(1) Direction::Vertical
            >
                <Block .rounded .title_top="child #2" Width::grow() Height::grow()/>
                <Block .rounded .title_top="child #3" Width::grow() Height::grow()/>
            </Block>
        </Block>
    };
    let root = ctx.spawn_ui(root);
    let mut buf = Buffer::empty(Rect::new(0, 0, 36, 18));
    ctx.calculate_layout(root, buf.area).unwrap();
    ctx.render(root, buf.area, &mut buf);

    let expected = Buffer::with_lines(vec![
        "╭parent────────────────────────────╮",
        "│╭sidebar─╮╭child #1──────────────╮│",
        "││        ││╭child #2────────────╮││",
        "││        │││                    │││",
        "││        │││                    │││",
        "││        │││                    │││",
        "││        │││                    │││",
        "││        │││                    │││",
        "││        ││╰────────────────────╯││",
        "││        ││                      ││",
        "││        ││╭child #3────────────╮││",
        "││        │││                    │││",
        "││        │││                    │││",
        "││        │││                    │││",
        "││        │││                    │││",
        "││        ││╰────────────────────╯││",
        "│╰────────╯╰──────────────────────╯│",
        "╰──────────────────────────────────╯",
    ]);
    assert_eq!(buf, expected);
}

#[test]
fn test_list_justify() {
    _ = color_eyre::install();

    let mut ctx = ElementCtx::new();

    #[subview]
    fn numbered_box(idx: i32) -> View {
        ui! {
            <Block .rounded Width::fixed(4) Height::fixed(3)>
                // formatting out of the box :)
                "{idx:02}"
            </Block>
        }
    }
    #[subview]
    fn container(justify: MainJustify, children: impl AsChildren) -> View {
        ui! {
            <Block
                .title_top={format!("{justify:?}")}
                .rounded
                {justify}
                Width::fixed(24)
                Height::fixed(5)
                Direction::Horizontal
            >
                {children}
            </Block>
        }
    }
    #[subview]
    fn root() -> View {
        ui! {
            <Block>
            {
                MainJustify::iter().map(|justify|
                    ui! {
                        <Container
                            .justify={justify}
                            .children={
                                (0..3).map(|idx| ui!{
                                    <NumberedBox .idx={idx} />
                                })
                            }
                        />
                    }
                )
            }
            </Block>
        }
    }
    let root = ctx.spawn_ui(root());
    let mut buf = Buffer::empty(Rect::new(0, 0, 24, 30));
    ctx.calculate_layout(root, buf.area).unwrap();
    ctx.render(root, buf.area, &mut buf);
    let expected = Buffer::with_lines(vec![
        "╭Start─────────────────╮",
        "│╭──╮╭──╮╭──╮          │",
        "││00││01││02│          │",
        "│╰──╯╰──╯╰──╯          │",
        "╰──────────────────────╯",
        "╭Center────────────────╮",
        "│     ╭──╮╭──╮╭──╮     │",
        "│     │00││01││02│     │",
        "│     ╰──╯╰──╯╰──╯     │",
        "╰──────────────────────╯",
        "╭SpaceBetween──────────╮",
        "│╭──╮     ╭──╮     ╭──╮│",
        "││00│     │01│     │02││",
        "│╰──╯     ╰──╯     ╰──╯│",
        "╰──────────────────────╯",
        "╭SpaceAround───────────╮",
        "│ ╭──╮   ╭──╮   ╭──╮   │",
        "│ │00│   │01│   │02│   │",
        "│ ╰──╯   ╰──╯   ╰──╯   │",
        "╰──────────────────────╯",
        "╭SpaceEvenly───────────╮",
        "│  ╭──╮  ╭──╮  ╭──╮    │",
        "│  │00│  │01│  │02│    │",
        "│  ╰──╯  ╰──╯  ╰──╯    │",
        "╰──────────────────────╯",
        "╭End───────────────────╮",
        "│          ╭──╮╭──╮╭──╮│",
        "│          │00││01││02││",
        "│          ╰──╯╰──╯╰──╯│",
        "╰──────────────────────╯",
    ]);

    assert_eq!(buf, expected);
    tracing::info!("\ntest_list_justify\n{}", buffer_to_string(&buf));
}

#[test]
#[should_panic]
fn test_hecs() {
    _ = color_eyre::install();
    let mut world = World::new();
    let a = world.spawn((0i32, "hi"));
    let b = world.spawn((1i32, "hello"));
    let mut query1 = world.query_one::<&mut i32>(a);
    let mut query2 = world.query_one::<&mut i32>(b);
    let a = query1.get();
    let b = query2.get();
    assert_ne!(a, b);
}

#[test]
fn test_percentage_01() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="parent" Width::grow() Height::grow() Direction::Horizontal>
            <Block .rounded .title_top="#1" Width::percentage(25) Height::grow()>
                "25%"
            </Block>
            <Block .rounded .title_top="#2" Width::percentage(75) Height::grow()>
                "75%"
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
            "│╭#1───╮╭#2───────────────╮│",
            "││25%  ││75%              ││",
            "│╰─────╯╰─────────────────╯│",
            "╰──────────────────────────╯",
        ])
    );
}

/// tests an edge case where the children of a grow container with non zero gap
/// next to a percentage container overlap the parent.
///
/// Caused by the waterfall algorithm setting the size of the smallest child to
/// the size of the second smallest child indiscriminately, without clamping the
/// difference to the remaining space.
#[test]
fn test_percentage_03() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="root" Width::grow() Height::grow() Direction::Horizontal Gap(1)>
            <Block .rounded .title_top="#1.0" Width::percentage(25) Height::grow() Center>
                "25%"
            </Block>
            <Block .rounded Width::grow() Height::grow() Direction::Horizontal Gap(1)>
                <Block .rounded .title_top="#2.0" Width::percentage(50) Height::grow()>
                    "50%"
                </Block>
                <Block .rounded .title_top="#2.1" Width::grow() Height::grow()>
                    "50%"
                </Block>
            </Block>
        </Block>
    };
    let root = ctx.spawn_ui(root);

    let mut buf = Buffer::empty(Rect::new(0, 0, 33, 7));

    ctx.calculate_layout(root, buf.area).unwrap();
    ctx.render(root, buf.area, &mut buf);

    assert_eq!(
        buf,
        Buffer::with_lines([
            "╭root───────────────────────────╮",
            "│╭#1.0──╮ ╭────────────────────╮│",
            "││      │ │╭#2.0────╮ ╭#2.1───╮││",
            "││ 25%  │ ││50%     │ │50%    │││",
            "││      │ │╰────────╯ ╰───────╯││",
            "│╰──────╯ ╰────────────────────╯│",
            "╰───────────────────────────────╯",
        ])
    );
}

#[test]
fn test_percentage_02() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="parent" Width::grow() Height::grow() Direction::Horizontal>
            <Block .rounded .title_top="#1" Width::percentage(25) Height::grow()>
                "25%"
            </Block>
            <Block .rounded .title_top="#2" Width::grow() Height::grow()>
                "75%"
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
            "│╭#1───╮╭#2───────────────╮│",
            "││25%  ││75%              ││",
            "│╰─────╯╰─────────────────╯│",
            "╰──────────────────────────╯",
        ])
    );
}

#[test]
fn test_max_width_01() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="parent" Width::grow() Height::grow() Direction::Horizontal>
            <Block .rounded .title_top="child" Width::grow() MaxWidth::percentage(75) Height::grow()>
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
            "│╭child─────────────╮      │",
            "││hello             │      │",
            "│╰──────────────────╯      │",
            "╰──────────────────────────╯",
        ])
    );
}

#[test]
fn test_max_width_02() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block .rounded .title_top="parent" Width::grow() Height::grow() Direction::Horizontal>
            <Block .rounded .title_top="child" Width::percentage(30) MaxWidth::fixed(8) Height::grow()>
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

#[test]
fn test_cross_axis_fit() {
    let mut ctx = ElementCtx::new();
    let root = ui! {
        <Block>
            <Block .rounded>
                <Block Padding::horizontal(3)>
                    <Text>"Title"</Text>
                    <Text>"Subtitle   ^C to quit"</Text>
                </Block>
            </Block>
            <Block Padding::horizontal(5)>
                <Text>"Select:"</Text>
                <Text>"item #01"</Text>
                <Text>"item #02"</Text>
                <Text>"item #03"</Text>
                <Block .rounded Width::fit()>" Confirm "</Block>
                <Text>"↑↓ or J/K to navigate"</Text>
            </Block>
        </Block>
    };
    let root = ctx.spawn_ui(root);

    let mut buf = Buffer::empty(Rect::new(0, 0, 28, 14));

    ctx.calculate_layout(root, buf.area).unwrap();
    ctx.render(root, buf.area, &mut buf);

    assert_eq!(
        buf,
        Buffer::with_lines([
            "╭──────────────────────────╮",
            "│   Title                  │",
            "│   Subtitle   ^C to quit  │",
            "╰──────────────────────────╯",
            "     Select:                ",
            "     item #01               ",
            "     item #02               ",
            "     item #03               ",
            "     ╭─────────╮            ",
            "     │ Confirm │            ",
            "     ╰─────────╯            ",
            "     ↑↓ or J/K to navigate  ",
            "                            ",
            "                            ",
        ])
    );
}

#[test]
fn test_node_post_render_schedule() {
    let root = ui! {
        <Block .rounded Width::grow() Height::grow() Direction::Horizontal>
            <Block .rounded Width::grow() Height::grow()/>
            <Block .rounded Width::grow() Height::grow()/>
        </Block>
    };
    let area = Rect::new(0, 0, 16, 5);
    let mut ctx = ElementCtx::new();

    ctx.add_system::<NodePostRenderSchedule>(|_world, area, buf, _element| {
        Block::bordered()
            .border_type(BorderType::Double)
            .render(area, buf);
    });

    let mut buf = Buffer::empty(area);

    let root = ctx.spawn_ui(root);

    ctx.calculate_layout(root, area).unwrap();
    ctx.render(root, area, &mut buf);

    assert_eq!(
        buf,
        Buffer::with_lines([
            "╔══════════════╗",
            "║╔═════╗╔═════╗║",
            "║║     ║║     ║║",
            "║╚═════╝╚═════╝║",
            "╚══════════════╝",
        ])
    );
}
