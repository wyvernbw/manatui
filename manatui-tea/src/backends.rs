use glam::{I16Vec2, U16Vec2};
use ratatui::prelude::Backend;

use crate::{Chan, RuntimeMsg};

pub trait ManaBackend: Backend {
    type Events: EventStream<Self::Event>;
    type KeyEvent;
    type Event;

    #[allow(async_fn_in_trait)]
    async fn create_events(&mut self) -> Self::Events;

    fn default_cycle_event() -> Self::Event;

    fn event_as_key(ev: Self::Event) -> Option<Self::KeyEvent>;

    fn event_as_direction(ev: &Self::Event) -> Option<I16Vec2>;

    fn event_is_confirm(ev: &Self::Event) -> bool;
}

pub trait EventStream<Out> {
    type Err;

    #[allow(async_fn_in_trait)]
    async fn read(&mut self) -> Result<Out, Self::Err>;
}

pub(crate) struct MsgStream<Msg, W: std::io::Write> {
    pub(crate) event_stream: <DefaultBackend<W> as ManaBackend>::Events,
    pub(crate) dispatch: Chan<Msg>,
}

impl<Msg, W: std::io::Write> MsgStream<Msg, W> {
    pub(crate) async fn next(this: &mut Self) -> RuntimeMsg<Msg> {
        loop {
            tokio::select! {
                event = this.event_stream.read() => {
                    if let Ok(event) = event { return RuntimeMsg::Term(event) }
                }
                msg = this.dispatch.1.recv_async() => {
                    let more_queued = !this.dispatch.1.is_empty();
                    if let Ok(msg) = msg { return RuntimeMsg::App(msg, more_queued) }
                }
            }
        }
    }
}

#[cfg(feature = "crossterm")]
pub(crate) mod crossterm_backend {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use glam::{I16Vec2, U16Vec2};
    use manatui_utils::key;
    use ratatui::prelude::CrosstermBackend;
    use tokio_stream::StreamExt;

    use crate::backends::{EventStream, ManaBackend};

    impl<W: std::io::Write> ManaBackend for CrosstermBackend<W> {
        type Events = crossterm::event::EventStream;
        type KeyEvent = crossterm::event::KeyEvent;
        type Event = crossterm::event::Event;

        async fn create_events(&mut self) -> Self::Events {
            crossterm::event::EventStream::new()
        }

        fn default_cycle_event() -> Self::Event {
            Event::Key(crossterm::event::KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            })
        }

        fn event_as_key(ev: Self::Event) -> Option<Self::KeyEvent> {
            ev.as_key_event()
        }

        fn event_as_direction(ev: &Self::Event) -> Option<I16Vec2> {
            match ev {
                key!(Char('h')) | key!(Left) => Some(I16Vec2::new(-1, 0)),
                key!(Char('j')) | key!(Down) => Some(I16Vec2::new(0, 1)),
                key!(Char('k')) | key!(Up) => Some(I16Vec2::new(0, -1)),
                key!(Char('l')) | key!(Right) => Some(I16Vec2::new(1, 0)),
                _ => None,
            }
        }

        fn event_is_confirm(ev: &Self::Event) -> bool {
            matches!(ev, key!(Enter))
        }
    }

    impl EventStream<crossterm::event::Event> for crossterm::event::EventStream {
        type Err = std::io::Error;

        async fn read(&mut self) -> Result<crossterm::event::Event, Self::Err> {
            loop {
                let res = self.next().await;
                if let Some(event) = res {
                    return event;
                }
            }
        }
    }

    pub type DefaultBackend<W> = CrosstermBackend<W>;
    pub type DefaultEvent = <DefaultBackend<std::io::Stdout> as ManaBackend>::Event;
    pub type DefaultKeyEvent = <DefaultBackend<std::io::Stdout> as ManaBackend>::KeyEvent;

    pub trait KeyEventExt {
        #[must_use]
        fn char(c: char) -> KeyEvent {
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            }
        }
    }

    impl KeyEventExt for KeyEvent {}
}

#[cfg(feature = "crossterm")]
pub use crossterm_backend::*;
