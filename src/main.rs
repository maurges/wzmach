use bpaf::construct;

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
    let debug_config = bpaf::command(
        "debug-config",
        Some("Parse config file and check it for errors"),
        bpaf::Info::default()
            .descr("Parse and print a config file")
            .for_parser(bpaf::positional("FILENAME")),
    )
    .map(|path| Opts::DebugConfig { path });

    let debug_gestures = bpaf::command(
        "debug-gestures",
        Some("Print all completed gestures but execute no actions"),
        bpaf::Info::default()
            .descr("If you see nothing you need to `export RUST_LOG=debug` or `trace`")
            .for_parser(bpaf::Parser::pure(Opts::DebugGestures)),
    );

    let debug_events = bpaf::command(
        "debug-events",
        Some("Print all incoming libinput gesture events and execute nothing"),
        bpaf::Info::default()
            .descr("If you see nothing you need to `export RUST_LOG=debug` or `trace`")
            .for_parser(bpaf::Parser::pure(Opts::DebugEvents)),
    );

    let config_path = bpaf::long("config")
        .help("Path to a config file to use instead of default")
        .argument("PATH")
        .optional();
    let run = construct!(Opts::Run { config_path });

    let parser = debug_config
        .or_else(debug_gestures)
        .or_else(debug_events)
        .or_else(run);

    bpaf::Info::default()
        .descr("Touchpad gesture engine")
        .for_parser(parser)
        .run()
}

fn main() {
    env_logger::init();
    log::trace!("initialized logging");

    match parse_opts() {
        Opts::DebugConfig { path } => {
            match config::Config::load(path) {
                Ok(c) => println!("Load successful:\n{:?}", c),
                Err(e) => println!("Error during loading:\n{}", e),
            }
        }

        Opts::DebugGestures => debug_events(),

        Opts::DebugEvents => {
            let producer = input_producer::GestureProducer::new();
            log::debug!("Created input connection");
            for event in producer {
                log::debug!("update: {:?}", event);
            }
        }

        Opts::Run { config_path } => {
            match load_config(config_path) {
                Ok(x) => run(x),
                Err(e) => startup_error(e),
            }
        }
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
    // read config
    let is_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();

    // run

    let (triggers, mut actions) = config.make_triggers(is_wayland);

    log::info!("Starting up");
    let producer = input_producer::GestureProducer::new();
    log::debug!("Created input connection");
    let events = gesture_event::EventAdapter::new(producer, &triggers);
    for action_inds in events {
        for index in action_inds {
            match actions[index].execute() {
                Ok(()) => (),
                Err(action::ActionError(msg)) => log::error!("{}", msg),
            }
        }
    }
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
    let events = gesture_event::EventAdapter::new(producer, &triggers);
    for event in events {
        for i in event {
            log::debug!("triggered: {:?}", triggers[i]);
        }
    }
}
