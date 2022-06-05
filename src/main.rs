mod action_sink;
mod common;
mod config;
mod gesture_event;
mod input_producer;

fn main() {
    env_logger::init();
    log::info!("initialized logging");

    // debug stuff, controlled by args
    let mut args = std::env::args();
    let _name = args.next().unwrap();
    if let Some(s) = args.next() {
        let producer = input_producer::GestureProducer::new();
        log::debug!("Created input connection");
        if s == "all" {
            for event in producer {
                log::debug!("update: {:?}", event);
            }
        } else if s == "events" {
            debug_events(producer);
        } else if s == "config" {
            let location = args.next().unwrap_or("./config.ron".to_string());
            let c = config::Config::load(location);
            println!("{:?}", c);
        }
        return;
    }

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

fn debug_events(producer: input_producer::GestureProducer) {
    let triggers = {
        let mut ts = Vec::new();
        use gesture_event::trigger::*;
        use common::{Direction, PinchDirection};
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
            ts.push(Trigger::Hold(HoldTrigger {
                fingers,
                time: 50,
            }));
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
