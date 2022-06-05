mod action_sink;
mod common;
mod config;
mod gesture_event;
mod input_producer;

#[derive(PartialEq, Eq, Debug, Clone)]
enum Opts {
    Run,
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

    let run = bpaf::Parser::pure(Opts::Run);

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
            let c = config::Config::load(path);
            println!("{:?}", c);
        }

        Opts::DebugGestures => debug_events(),

        Opts::DebugEvents => {
            let producer = input_producer::GestureProducer::new();
            log::debug!("Created input connection");
            for event in producer {
                log::debug!("update: {:?}", event);
            }
        }

        Opts::Run => run(),
    }
}

fn run() {
    // read config
    let home = std::env::var_os("HOME").unwrap().into_string().unwrap();
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(|x| x.into_string().unwrap())
        .unwrap_or_else(|| home + "/.config");
    let config_dir = config_home + "/wzmach/";
    let config: config::Config = {
        let common = config::Config::load(config_dir.clone() + "config.ron").unwrap_or_default();
        // TODO maybe: search other locations
        common
    };
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
                Err(action_sink::ActionError(msg)) => log::error!("{}", msg),
            }
        }
    }
}

fn debug_events() {
    let producer = input_producer::GestureProducer::new();
    log::debug!("Created input connection");
    let triggers = {
        let mut ts = Vec::new();
        use common::{Direction, PinchDirection};
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
