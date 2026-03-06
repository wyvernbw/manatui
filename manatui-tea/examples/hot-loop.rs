use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{EnableMouseCapture, KeyEvent, KeyModifiers};
use manatui_layout::prelude::*;
use manatui_layout::ui::View;
use manatui_macros::ui;
use manatui_tea::backends::DefaultEvent;
use manatui_tea::focus::handlers::On;
use manatui_tea::{Effect, HotLoop, Message, run};
use manatui_utils::key;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    fn should_quit(_: &Model, event: &AppMsg) -> bool {
        matches!(event, AppMsg::Quit)
    }
    run()
        .init(init)
        .view(view)
        .update(update)
        .quit_signal(should_quit)
        .fps(Duration::from_millis(16)) // ~60fps hot loop
        .run()
        .await
        .unwrap();
}

#[derive(Debug, Clone)]
struct Model {
    started_at: Instant,
    elapsed: Duration,
    running: bool,
    laps: Vec<Duration>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            elapsed: Duration::ZERO,
            running: false,
            laps: vec![],
        }
    }
}

impl manatui_tea::Model for Model {
    // Returning Some here opts into the hot render loop
    fn delta_time_mut(&mut self) -> Option<&mut Duration> {
        Some(&mut self.elapsed)
    }
}

#[derive(Debug, Clone)]
enum AppMsg {
    StartStop,
    Lap,
    Reset,
    Quit,
}

impl Message for AppMsg {
    type Model = Model;
}

async fn init() -> (Model, Effect<AppMsg>) {
    _ = crossterm::execute!(stdout(), EnableMouseCapture);
    (Model::default(), Effect::none())
}

async fn view(model: &Model) -> View {
    let elapsed = if model.running {
        model.started_at.elapsed()
    } else {
        model.elapsed
    };

    let mins = elapsed.as_secs() / 60;
    let secs = elapsed.as_secs() % 60;
    let millis = elapsed.subsec_millis();
    let time_str = format!("{mins:02}:{secs:02}.{millis:03}");

    let status = if model.running { "Running" } else { "Paused" };

    ui! {
        <Block
            .rounded
            .title_top="Stopwatch"
            Center
            Gap(1)
            Height::grow() Width::grow()
            On::new(handle_keys)
        >
            <Block>
            {[
                if model.running {
                    ui! { <Block HotLoop/> }
                } else {
                    ui! { <Block/> }
                }
            ]}
            </Block>
            <Block Center Width::fixed(20) Height::fixed(1)>
                {time_str}
            </Block>
            <Block Center Width::fixed(10) Height::fixed(1)>
                {status}
            </Block>
            <Block Direction::Horizontal Center Gap(4)>
                <Block Center Width::fixed(20) Height::fixed(1)>
                    "Space: Start/Stop"
                </Block>
                <Block Center Width::fixed(12) Height::fixed(1)>
                    "L: Lap"
                </Block>
                <Block Center Width::fixed(12) Height::fixed(1)>
                    "R: Reset"
                </Block>
            </Block>
            <Block Height::fixed(15)>
            {[
                if model.laps.is_empty() {
                    ui! { <Block></Block> }
                } else {
                    let laps = model.laps.iter().enumerate().map(|(i, d)| {
                        let s = d.as_secs();
                        let ms = d.subsec_millis();
                        let text = format!("Lap {}: {:02}:{:02}.{:03}", i + 1, s / 60, s % 60, ms);
                        ui! {
                            <Text>"{text}"</Text>
                        }
                    });
                    ui! {
                        <Block .rounded .title_top="Laps" Width::fixed(20)>
                            {laps}
                        </Block>
                    }
                }
            ]}
            </Block>
        </Block>
    }
}

fn handle_keys(_: &Model, event: &DefaultEvent) -> Option<(AppMsg, Effect<AppMsg>)> {
    match event {
        key!(Char(' ')) => Some((AppMsg::StartStop, Effect::none())),
        key!(Char('l')) => Some((AppMsg::Lap, Effect::none())),
        key!(Char('r')) => Some((AppMsg::Reset, Effect::none())),
        key!(Char('q')) | key!(Char('c'), KeyModifiers::CONTROL) => {
            Some((AppMsg::Quit, Effect::none()))
        }
        _ => None,
    }
}

async fn update(mut model: Model, msg: AppMsg) -> (Model, Effect<AppMsg>) {
    match msg {
        AppMsg::StartStop => {
            if model.running {
                // Pause: snapshot elapsed time
                (
                    Model {
                        elapsed: model.started_at.elapsed(),
                        running: false,
                        ..model
                    },
                    Effect::none(),
                )
            } else {
                // Resume: reset started_at so elapsed is relative to now
                (
                    Model {
                        started_at: Instant::now(),
                        running: true,
                        ..model
                    },
                    Effect::none(),
                )
            }
        }
        AppMsg::Lap => {
            let lap_time = if model.running {
                model.started_at.elapsed()
            } else {
                model.elapsed
            };
            model.laps.push(lap_time);
            (model, Effect::none())
        }
        AppMsg::Reset => (Model::default(), Effect::none()),
        AppMsg::Quit => (model, Effect::none()),
    }
}
