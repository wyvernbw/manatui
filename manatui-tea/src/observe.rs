use std::sync::Arc;

use crossbeam::atomic::AtomicCell;
use crossterm::event::MouseEvent;
use hecs::View;
use manatui_layout::layout::{Children, Element, Props};
use ratatui::layout::Rect;
use tailcall::tailcall;

#[derive(Default, Clone, Debug)]
pub struct AreaRef(Arc<AtomicCell<Option<Rect>>>);

impl AreaRef {
    #[must_use]
    pub fn get(&self) -> Option<Rect> {
        self.0.load()
    }
    pub(crate) fn set(&self, value: Rect) {
        self.0.store(Some(value));
    }
    #[must_use]
    pub fn new(value: Rect) -> Self {
        Self(Arc::new(AtomicCell::new(Some(value))))
    }
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn contains(&self, x: u16, y: u16) -> bool {
        self.get()
            .is_some_and(|rect| rect.contains(ratatui::layout::Position { x, y }))
    }
}

#[derive(Default, Clone, Debug)]
pub struct HitTest(Arc<AtomicCell<HitEvent>>);

#[derive(Default, Clone, Debug, Copy)]
pub enum HitEvent {
    #[default]
    None,
    Clicked,
    Hovered,
}

impl HitTest {
    #[must_use]
    pub fn get(&self) -> HitEvent {
        self.0.load()
    }
    pub(crate) fn set(&self, value: HitEvent) {
        self.0.store(value);
    }
    #[must_use]
    pub fn new(value: HitEvent) -> Self {
        Self(Arc::new(AtomicCell::new(value)))
    }
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
    #[tailcall]
    pub(crate) fn hit_test(
        query: &View<'_, (&Props, &Children, Option<&HitTest>)>,
        root: Element,
        mouse_event: MouseEvent,
    ) {
        if mouse_event.kind.is_up() {
            return;
        }

        let Some((props, children, hit_test)) = query.get(root) else {
            return;
        };

        let rect = Rect::new(
            props.position.x,
            props.position.y,
            props.size.x,
            props.size.y,
        );
        if let Some(hit_test) = hit_test {
            if rect.contains(ratatui::layout::Position::new(
                mouse_event.column,
                mouse_event.row,
            )) {
                let hit = match mouse_event.kind {
                    crossterm::event::MouseEventKind::Down(_) => HitEvent::Clicked,
                    crossterm::event::MouseEventKind::Drag(_) => HitEvent::None,
                    crossterm::event::MouseEventKind::Moved => HitEvent::Hovered,
                    _ => HitEvent::None,
                };
                hit_test.set(hit);
            } else {
                hit_test.set(HitEvent::None);
            }
        }

        let children = children.clone();
        for entity in children.iter() {
            Self::hit_test(query, *entity, mouse_event);
        }
    }
}
