mod gesture_event;
mod input_producer;
mod action_sink;

use action_sink::Action;

fn main() {
    env_logger::init();
    log::info!("initialized logging");

    let producer = input_producer::GestureProducer::new();

    let mut args = std::env::args();
    let _name = args.next().unwrap();
    if let Some(s) = args.next() {
        if s == "all" {
            for event in producer {
                log::debug!("update: {:?}", event);
            }
        } else if s == "events" {
            debug_events(producer);
        }
        return;
    }

    let device = action_sink::UinputAction::default_device();

    use gesture_event::trigger::*;
    use uinput::event::keyboard::Key;

    log::info!("Creating pipeline");

    let mut triggers = Vec::new();
    let mut actions = Vec::new();
    let trigger_3_up = Trigger::Swipe(CardinalTrigger {
        fingers: FingerCount::new(3),
        direction: Direction::Up,
        distance: Distance::new(100),
        repeated: false,
    });
    let action_3_up = action_sink::UinputAction {
        device: device.clone(),
        modifiers: vec![Key::RightControl],
        sequence: vec![Key::T],
    };
    triggers.push(trigger_3_up);
    actions.push(action_3_up);

    let trigger_3_down = Trigger::Swipe(CardinalTrigger {
        fingers: FingerCount::new(3),
        direction: Direction::Down,
        distance: Distance::new(100),
        repeated: false,
    });
    let action_3_down = action_sink::UinputAction {
        device: device.clone(),
        modifiers: vec![Key::RightControl],
        sequence: vec![Key::W],
    };
    triggers.push(trigger_3_down);
    actions.push(action_3_down);

    let trigger_3_left = Trigger::Swipe(CardinalTrigger {
        fingers: FingerCount::new(3),
        direction: Direction::Left,
        distance: Distance::new(100),
        repeated: false,
    });
    let action_3_left = action_sink::UinputAction {
        device: device.clone(),
        modifiers: vec![Key::RightControl],
        sequence: vec![Key::PageDown],
    };
    triggers.push(trigger_3_left);
    actions.push(action_3_left);

    let trigger_3_right = Trigger::Swipe(CardinalTrigger {
        fingers: FingerCount::new(3),
        direction: Direction::Right,
        distance: Distance::new(100),
        repeated: false,
    });
    let action_3_right = action_sink::UinputAction {
        device: device.clone(),
        modifiers: vec![Key::RightControl],
        sequence: vec![Key::PageUp],
    };
    triggers.push(trigger_3_right);
    actions.push(action_3_right);

    log::info!("Starting up");
    let events = gesture_event::EventAdapter::new(producer, &triggers);
    for action_inds in events {
        for index in action_inds {
            match actions[index].execute() {
                Ok(()) => (),
                Err(action_sink::ActionError(msg)) =>
                    log::error!("{}", msg),
            }
        }
    }
}

fn debug_events(producer: input_producer::GestureProducer) {
    let triggers = {
        let mut ts = Vec::new();
        use gesture_event::trigger::*;
        for fingers in 2..5 {
            for repeated in [false, true] {
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: Direction::Up,
                    distance: Distance::new(100),
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: Direction::Down,
                    distance: Distance::new(100),
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: Direction::Left,
                    distance: Distance::new(100),
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: Direction::Right,
                    distance: Distance::new(100),
                    repeated,
                }));
                ts.push(Trigger::Pinch(PinchTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: PinchDirection::In,
                    scale: 1.3,
                    repeated,
                }));
                ts.push(Trigger::Pinch(PinchTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: PinchDirection::Out,
                    scale: 1.3,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: Direction::Up,
                    distance: Distance::new(100),
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: Direction::Down,
                    distance: Distance::new(100),
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: Direction::Left,
                    distance: Distance::new(100),
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers: FingerCount::new(fingers),
                    direction: Direction::Right,
                    distance: Distance::new(100),
                    repeated,
                }));
            }
            ts.push(Trigger::Hold(HoldTrigger {
                fingers: FingerCount::new(fingers),
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
