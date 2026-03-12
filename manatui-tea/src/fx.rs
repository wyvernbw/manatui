use std::time::Duration;
pub use tachyonfx::fx::*;

use manatui_layout::layout::NodePostRenderSchedule;
use ratatui::prelude::Backend;

use crate::Ctx;

impl<B: Backend> Ctx<B> {
    pub(crate) fn setup_fx(&mut self) {
        self.el_ctx
            .add_system::<NodePostRenderSchedule>(|world, area, buf, node| {
                let Ok((fx, fx_list)) =
                    world.query_one_mut::<(Option<&mut Fx>, Option<&mut FxList>)>(node)
                else {
                    return;
                };
                if let Some(fx) = fx {
                    fx.0.process(Duration::default(), buf, area);
                }
                if let Some(fx_list) = fx_list {
                    for fx in &mut fx_list.0 {
                        fx.0.process(Duration::default(), buf, area);
                    }
                }
            });
    }
    pub(crate) fn advance_fx(&mut self, dt: Duration) {
        for fx in self.query_mut::<&mut Fx>() {
            fx.advance(dt);
        }
        for fx_list in self.query_mut::<&mut FxList>() {
            for fx in &mut fx_list.0 {
                fx.advance(dt);
            }
        }
    }
}

#[derive(Debug, derive_more::Deref, derive_more::DerefMut, derive_more::From, Clone)]
pub struct Fx(tachyonfx::Effect);

impl Default for Fx {
    fn default() -> Self {
        Self(tachyonfx::fx::sequence(&[]))
    }
}

impl Fx {
    /// construct a new shader effect value that can be used as a component.
    #[must_use]
    pub fn new(effect: tachyonfx::Effect) -> Self {
        Fx(effect)
    }

    pub fn advance(&mut self, dt: Duration) {
        if let Some(timer) = self.timer_mut() {
            timer.process(dt);
        }
    }
}

unsafe impl Send for Fx {}
unsafe impl Sync for Fx {}

pub struct FxList(Vec<Fx>);

impl FxList {
    /// construct a list of shader effects that can be used as a component.
    #[must_use]
    pub fn new(effects: impl IntoIterator<Item = Fx>) -> Self {
        FxList(effects.into_iter().collect())
    }
}
