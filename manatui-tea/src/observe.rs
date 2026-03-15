use std::sync::Arc;

use crossbeam::atomic::AtomicCell;
use ratatui::layout::Rect;

#[derive(Default, Clone)]
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
