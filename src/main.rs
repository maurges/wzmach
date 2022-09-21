use bpaf::{construct, Parser};

mod action;
mod common;
mod config;
mod gesture_event;
mod input_producer;

#[derive(PartialEq, Eq, Debug, Clone)]
enum Opts {
    Run { config_path: Option<String> },
    DebugConfig { path: String },
    DebugGestures,
    DebugEvents,
}

fn parse_opts() -> Opts {
    let debug_config = bpaf::positional("FILENAME")
        .to_options()
        .descr("Parse and print a config file")
        .command("debug-config")
        .help("Parse config file and check it for errors")
        .map(|path| Opts::DebugConfig { path });

    let debug_gestures = bpaf::pure(Opts::DebugGestures)
        .to_options()
        .descr("If you see nothing you need to `export RUST_LOG=debug` or `trace`")
        .command("debug-gestures")
        .help("Print all completed gestures but execute no actions");

    let debug_events = bpaf::pure(Opts::DebugEvents)
        .to_options()
        .descr("If you see nothing you need to `export RUST_LOG=debug` or `trace`")
        .command("debug-events")
        .help("Print all incoming libinput gesture events and execute nothing");

    let config_path = bpaf::long("config")
        .help("Path to a config file to use instead of default")
        .argument("PATH")
        .optional();

    let run = construct!(Opts::Run { config_path });

    (construct!([debug_config, debug_gestures, debug_events, run]))
        .to_options()
        .descr("Touchpad gesture engine")
        .run()
}

fn main() {
    env_logger::init();
    log::trace!("initialized logging");

    match parse_opts() {
        Opts::DebugConfig { path } => match config::Config::load(path) {
            Ok(c) => println!("Load successful:\n{:?}", c),
            Err(e) => println!("Error during loading:\n{}", e),
        },

        Opts::DebugGestures => debug_events(),

        Opts::DebugEvents => {
            let producer = input_producer::GestureProducer::new();
            log::debug!("Created input connection");
            for event in producer {
                log::debug!("update: {:?}", event);
            }
        }

        Opts::Run { config_path } => match load_config(config_path) {
            Ok(x) => run(x),
            Err(e) => startup_error(e),
        },
    }
}

fn load_config(mb_path: Option<String>) -> Result<config::Config, std::io::Error> {
    let config_path = mb_path.unwrap_or_else(|| {
        let home = std::env::var_os("HOME").unwrap().into_string().unwrap();
        let config_home = std::env::var_os("XDG_CONFIG_HOME")
            .map(|x| x.into_string().unwrap())
            .unwrap_or_else(|| home + "/.config");
        let config_dir = config_home + "/wzmach/";

        let local_path = config_dir + "config.ron";
        let etc_path = "/etc/wzmach/config.ron".to_owned();
        if std::path::Path::new(&local_path).exists() {
            local_path
        } else {
            etc_path
        }
    });
    config::Config::load(config_path)
}

fn run(config: config::Config) {
    use crate::{action::consumer::EventConsumerExt, gesture_event::EventAdapterExt};

    // read config
    let is_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let (triggers, mut actions) = config.make_triggers(is_wayland);
    log::info!("Starting up");

    // run
    input_producer::GestureProducer::new()
        .adapt_events(&triggers)
        .consume_events(&mut actions);
}

fn startup_error(e: std::io::Error) {
    log::error!("Failed to start up: {}", e);
    notify_rust::Notification::new()
        .summary("Wzmach failed to start")
        .body(&format!("{}", e))
        .show()
        .unwrap();
}

fn debug_events() {
    use crate::gesture_event::EventAdapterExt;

    let producer = input_producer::GestureProducer::new();
    log::debug!("Created input connection");
    let triggers = {
        let mut ts = Vec::new();
        use common::{Direction, PinchDirection, RotateDirection};
        use gesture_event::trigger::*;
        for fingers in 2..5 {
            for repeated in [false, true] {
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers,
                    direction: Direction::Up,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers,
                    direction: Direction::Down,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers,
                    direction: Direction::Left,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers,
                    direction: Direction::Right,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Pinch(PinchTrigger {
                    fingers,
                    direction: PinchDirection::In,
                    scale: 1.3,
                    repeated,
                }));
                ts.push(Trigger::Pinch(PinchTrigger {
                    fingers,
                    direction: PinchDirection::Out,
                    scale: 1.3,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers,
                    direction: Direction::Up,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers,
                    direction: Direction::Down,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers,
                    direction: Direction::Left,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers,
                    direction: Direction::Right,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Rotate(RotateTrigger {
                    fingers,
                    direction: RotateDirection::Anticlockwise,
                    distance: 45.0,
                    repeated,
                }));
                ts.push(Trigger::Rotate(RotateTrigger {
                    fingers,
                    direction: RotateDirection::Clockwise,
                    distance: 45.0,
                    repeated,
                }));
            }
            ts.push(Trigger::Hold(HoldTrigger { fingers, time: 50 }));
        }
        ts
    };
    let events = producer.adapt_events(&triggers);
    for event in events {
        for i in event {
            log::debug!("triggered: {:?}", triggers[i]);
        }
    }
}
