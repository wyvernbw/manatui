pub mod handlers;

use std::{
    any::TypeId,
    collections::HashMap,
    hash::{BuildHasherDefault, DefaultHasher, Hash, Hasher},
    ops::Index,
};

use anyhow::anyhow;
use hecs::{Entity, Or, TypeIdMap, World};
use im::Vector;
use manatui_layout::layout::{Children, Props};
use manatui_utils::resource::Resources;
use ratatui::{layout::Rect, style::Style};

use crate::{
    DefaultEvent, Effect, Message,
    backends::{DefaultBackend, ManaBackend},
    focus::handlers::{ClickOnEnter, On, OnClick, OnKey},
};

#[derive(Debug, Clone, Copy)]
pub enum FocusPolicy {
    Popup,
    Pass,
    Block,
}

impl FocusPolicy {
    /// Returns `true` if the focus policy is [`Pass`].
    ///
    /// [`Pass`]: FocusPolicy::Pass
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    /// Returns `true` if the focus policy is [`Block`].
    ///
    /// [`Block`]: FocusPolicy::Block
    #[must_use]
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block)
    }
}

#[derive(Debug, Clone, Default)]
pub enum Navigation {
    Cycle(DefaultEvent),
    #[default]
    Directional,
}

#[derive(Debug, Clone, Default)]
pub struct NavGroup {
    nav: Navigation,
    elements: Vector<Entity>,
}

type FocusMap = HashMap<u128, UiIndex, BuildHasherDefault<NoOpHasher>>;

#[derive(Default)]
struct NoOpHasher {
    hash: u64,
}

/// Adapted from [`hecs`].
impl Hasher for NoOpHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write_u64(&mut self, n: u64) {
        // Only a single value can be hashed, so the old hash should be zero.
        debug_assert_eq!(self.hash, 0);
        self.hash = n;
    }

    // Tolerate TypeId being either u64 or u128.
    fn write_u128(&mut self, n: u128) {
        debug_assert_eq!(self.hash, 0);
        self.hash = n as u64;
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hasher = DefaultHasher::new();
        hasher.write(bytes);
        self.hash = hasher.finish();
    }
}

#[derive(Debug, Clone, Default)]
pub struct UiStack {
    stack: Vector<NavGroup>,
    focus_map: FocusMap,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct UiIndex(usize, usize);

impl Index<UiIndex> for UiStack {
    type Output = Entity;

    fn index(&self, index: UiIndex) -> &Self::Output {
        &self.stack[index.0].elements[index.1]
    }
}

impl UiStack {
    fn get_group(&self, idx: UiIndex) -> &NavGroup {
        &self.stack[idx.0]
    }
}

pub(crate) fn generate_ui_stack(world: &mut World, root: Entity) {
    let mut stack = Vector::new();
    let mut focus_map = FocusMap::default();
    let last_group =
        generate_ui_stack_impl(world, root, &mut stack, &mut focus_map, NavGroup::default());
    if !last_group.elements.is_empty() {
        stack.push_back(last_group);
    }
    world.insert_or_update_resource(UiStack { stack, focus_map });
}

#[tracing::instrument(skip(world, focus_map, stack))]
pub(crate) fn generate_ui_stack_impl(
    world: &World,
    root: Entity,
    stack: &mut Vector<NavGroup>,
    focus_map: &mut FocusMap,
    mut current_group: NavGroup,
) -> NavGroup {
    current_group.elements.push_back(root);
    let mut query = world.query_one::<(Option<&Navigation>, Option<&FocusTarget>)>(root);
    let Ok((nav, focus_target)) = query.get() else {
        return current_group;
    };

    if nav.is_some() {
        if !current_group.elements.is_empty() {
            stack.push_back(current_group.clone());
        }
        current_group = NavGroup::default();
    }

    if let Some(focus_target) = focus_target {
        focus_map.insert(
            focus_target.as_hash(),
            UiIndex(stack.len(), current_group.elements.len() - 1),
        );
    }

    let children = world.get::<&Children>(root);
    if let Ok(children) = children {
        for child in children.iter() {
            current_group =
                generate_ui_stack_impl(world, *child, stack, focus_map, current_group.clone());
        }
    } else {
        stack.push_back(current_group.clone());
    }

    current_group
}

#[must_use]
#[derive(Debug, Clone, Copy)]
pub enum FocusTarget {
    Static(TypeId),
    Dynamic(u128),
}

impl FocusTarget {
    #[inline(always)]
    const fn as_hash(&self) -> u128 {
        match self {
            FocusTarget::Static(type_id) => unsafe {
                // SAFETY: [`TypeId`] is just a `u128` hash
                std::mem::transmute::<TypeId, u128>(*type_id)
            },
            FocusTarget::Dynamic(value) => *value,
        }
    }
}

#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct FocusPopup;

impl FocusTarget {
    pub fn new<T: 'static>() -> Self {
        Self::Static(TypeId::of::<T>())
    }
    pub fn new_dyn<T: Hash + ?Sized>(value: &T) -> Self {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        Self::Dynamic(u128::from(hasher.finish()))
    }
}

pub(crate) fn init_focus_system(world: &mut World) {
    let _ = world.get_or_insert_resource_with::<&FocusContext>(|world| {
        let ui_stack = world.get_resource::<&UiStack>();
        let first_focus = ui_stack
            .ok()
            .and_then(|stack| stack.stack.iter().next().cloned())
            .and_then(|nav_group| nav_group.elements.iter().next().copied());
        let mut ctx = FocusContext { stack: Vec::new() };
        if let Some(entity) = first_focus {
            if let Ok(target) = world.get::<&FocusTarget>(entity) {
                ctx.push(target.as_hash());
            }
        }
        ctx
    });
}

macro_rules! try_handler {
    ($world:ident, $entity:ident, $on:ident, $model:ident, $msg:ident, $policy_blocks:expr, $accumulator:ident) => {
        let value = $on($model, $msg);
        if let Some(value) = value {
            _ = try_grab_focus($world, $entity);
            if $policy_blocks {
                return Ok(Some(value));
            }
            $accumulator = $accumulator.or(Some(value));
        }
    };
    ($world:ident, $entity:ident, Key($key:ident), $on:ident, $model:ident, $msg:ident, $policy_blocks:expr, $accumulator:ident) => {
        if let Some(key_event) = DefaultBackend::<std::io::Stdout>::event_as_key($msg.clone())
            && &key_event == $key
        {
            let value = $on($model, $msg);
            if let Some(value) = value {
                _ = try_grab_focus($world, $entity);
                if $policy_blocks {
                    return Ok(Some(value));
                }
                $accumulator = $accumulator.or(Some(value));
            }
        }
    };
}

pub(crate) fn propagate_key_event<Msg: Message>(
    world: &World,
    model: &Msg::Model,
    msg: &DefaultEvent,
) -> Result<Option<(Msg, Effect<Msg>)>, anyhow::Error> {
    let mut accumulator = None;
    if DefaultBackend::<std::io::Stdout>::event_is_confirm(msg) {
        let focus_ctx = world.get_resource::<&FocusContext>()?;
        if let Some(focused_on) = focus_ctx.top() {
            drop(focus_ctx);

            let uistack = world.get_resource::<&UiStack>()?;
            let idx = uistack.focus_map[&focused_on];
            let focused_on = uistack[idx];

            drop(uistack);

            let mut query = world.query::<(&OnClick<Msg>, &ClickOnEnter, Option<&FocusPolicy>)>();
            let query = query.view();
            if let Some((OnClick(on_click), _, focus_policy)) = query.get(focused_on) {
                let focus_policy = focus_policy.unwrap_or(&FocusPolicy::Pass);
                try_handler!(
                    world,
                    focused_on,
                    on_click,
                    model,
                    msg,
                    focus_policy.is_block(),
                    accumulator
                );
            }
        }
    }

    let stack = world.get_resource::<&UiStack>()?;
    let mut query = world.query::<(Or<&On<Msg>, &OnKey<Msg>>, Option<&FocusPolicy>)>();
    let query = query.view();
    for group in &stack.stack {
        for entity in group.elements.iter().copied() {
            if let Some((value, policy)) = query.get(entity) {
                let blocks = policy.unwrap_or(&FocusPolicy::Pass).is_block();
                match value {
                    Or::Left(On(on)) => {
                        try_handler!(world, entity, on, model, msg, blocks, accumulator);
                    }
                    Or::Right(OnKey(key, cb)) => {
                        try_handler!(world, entity, Key(key), cb, model, msg, blocks, accumulator);
                    }
                    Or::Both(On(on), OnKey(key, on_key)) => {
                        try_handler!(
                            world,
                            entity,
                            Key(key),
                            on_key,
                            model,
                            msg,
                            blocks,
                            accumulator
                        );
                        try_handler!(world, entity, on, model, msg, blocks, accumulator);
                    }
                }
            }
        }
    }
    Ok(accumulator)
}

pub(crate) fn propagate_mouse_event<Msg: Message>(
    world: &World,
    model: &Msg::Model,
    msg: &DefaultEvent,
    x_coord: u16,
    y_coord: u16,
) -> Result<Option<(Msg, Effect<Msg>)>, anyhow::Error> {
    #[cfg(feature = "crossterm")]
    {
        use crossterm::event::{Event, MouseEvent, MouseEventKind};
        if !matches!(
            msg,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(_),
                ..
            })
        ) {
            return Ok(None);
        }
    }
    let mut accumulator = None;
    let stack = world.get_resource::<&UiStack>()?;
    let mut query = world.query::<(&OnClick<Msg>, &Props, Option<&FocusPolicy>)>();
    let query = query.view();
    for group in &stack.stack {
        for entity in group.elements.iter().copied() {
            if let Some((OnClick(on_click), props, focus_policy)) = query.get(entity) {
                let blocks = focus_policy.unwrap_or(&FocusPolicy::Pass).is_block();
                let area = Rect {
                    x: props.position.x,
                    y: props.position.y,
                    width: props.size.x,
                    height: props.size.y,
                };
                if area.contains(ratatui::layout::Position {
                    x: x_coord,
                    y: y_coord,
                }) {
                    try_handler!(world, entity, on_click, model, msg, blocks, accumulator);
                }
            }
        }
    }
    Ok(accumulator)
}

pub(crate) fn propagate_event<Msg: Message>(
    world: &World,
    model: &Msg::Model,
    msg: &DefaultEvent,
) -> Result<Option<(Msg, Effect<Msg>)>, anyhow::Error> {
    #[cfg(feature = "crossterm")]
    {
        match msg {
            crossterm::event::Event::Key(_) => propagate_key_event(world, model, msg),
            crossterm::event::Event::Mouse(ev) => {
                propagate_mouse_event(world, model, msg, ev.column, ev.row)
            }
            _ => Ok(None),
        }
    }
}

pub(crate) fn navigation_system<B: ManaBackend>(
    world: &mut World,
    msg: &B::Event,
) -> anyhow::Result<()> {
    let Some(direction) = B::event_as_direction(msg) else {
        return Ok(());
    };
    let direction = direction.as_vec2();
    let focus_ctx = world.get_resource::<&FocusContext>()?;
    let current = focus_ctx.top();

    match current {
        None => {}
        Some(current) => {
            let uistack = world.get_resource::<&UiStack>()?;
            let idx = *uistack
                .focus_map
                .get(&current)
                .ok_or(anyhow!("focus target was not inserted into the focus map"))?;
            let nav_group = uistack.get_group(idx);
            let current_entity = uistack[idx];

            let mut query = world.query::<(&Props, &FocusTarget)>();
            let query = query.view();
            let (current_node_props, _) = query.get(current_entity).ok_or(anyhow!(
                "currently focused node has no props or focus target component"
            ))?;
            let current_node_position = current_node_props.position.as_vec2();

            let next_node = nav_group
                .elements
                .iter()
                .find_map(|&entity| match query.get(entity) {
                    Some((props, focus_target)) => {
                        if focus_target.as_hash() == current {
                            return None;
                        }

                        let to_node = props.position.as_vec2() - (current_node_position);
                        let to_node = to_node.normalize_or_zero();
                        let pointing_towards = to_node.dot(direction);
                        if (0.5..=1.0).contains(&pointing_towards) {
                            Some(focus_target.as_hash())
                        } else {
                            None
                        }
                    }
                    None => None,
                });

            if let Some(next_node_id) = next_node {
                drop(focus_ctx);
                let mut focus_ctx = world.get_resource::<&mut FocusContext>()?;
                focus_ctx.pop();
                focus_ctx.push(next_node_id);
            }
        }
    }

    Ok(())
}

pub(crate) fn try_grab_focus(world: &World, entity: Entity) -> anyhow::Result<()> {
    let mut query = world.query_one::<(&FocusTarget, Option<&FocusPopup>)>(entity);
    let (&focus_target, popup) = query.get()?;
    let popup = popup.is_some();

    let mut focus_ctx = world.get_resource::<&mut FocusContext>()?;
    if popup {
        if focus_ctx.top() != Some(focus_target.as_hash()) {
            focus_ctx.push(focus_target.as_hash());
        }
    } else {
        focus_ctx.pop();
        focus_ctx.push(focus_target.as_hash());
    }

    Ok(())
}

pub(crate) struct FocusContext {
    stack: Vec<u128>,
}

impl FocusContext {
    fn top(&self) -> Option<u128> {
        self.stack.last().copied()
    }
    fn push(&mut self, value: u128) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<u128> {
        self.stack.pop()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FocusStyle(pub Style);

pub(crate) fn set_focus_style(world: &mut World) -> anyhow::Result<()> {
    let focus_ctx = world.get_resource::<&FocusContext>()?;
    let current = focus_ctx.top();
    drop(focus_ctx);

    if let Some(current) = current {
        let uistack = world.get_resource::<&UiStack>()?;
        let focused_on = uistack.focus_map.get(&current);
        if let Some(focused_on) = focused_on {
            let entity = uistack[*focused_on];
            drop(uistack);
            let mut query = world.query_one::<(&Props, &FocusStyle)>(entity);
            if let Ok((&props, &style)) = query.get() {
                drop(query);
                (props.set_style)(world, entity, style.0);
            }
        }
    }

    Ok(())
}
