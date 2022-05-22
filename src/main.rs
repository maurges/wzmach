mod input_event;
mod gesture_event;

fn main() {
    let producer = input_event::GestureProducer::new();

    let mut args = std::env::args();
    let _name = args.next().unwrap();
    if let Some(s) = args.next() {
        if s == "all" {
            for event in producer {
                println!("update: {:?}", event);
            }
            return;
        }
    }

    let triggers = {
        let mut ts = Vec::new();
        use gesture_event::*;
        for fingers in 2..5 {
            ts.push(Trigger::Swipe(SwipeTrigger {
                fingers: FingerCount::new(fingers),
                direction: Direction::Up,
                distance: Distance::new(100),
            }));
            ts.push(Trigger::Swipe(SwipeTrigger {
                fingers: FingerCount::new(fingers),
                direction: Direction::Down,
                distance: Distance::new(100),
            }));
            ts.push(Trigger::Swipe(SwipeTrigger {
                fingers: FingerCount::new(fingers),
                direction: Direction::Left,
                distance: Distance::new(100),
            }));
            ts.push(Trigger::Swipe(SwipeTrigger {
                fingers: FingerCount::new(fingers),
                direction: Direction::Right,
                distance: Distance::new(100),
            }));
            ts.push(Trigger::Pinch(PinchTrigger {
                fingers: FingerCount::new(fingers),
                direction: PinchDirection::In,
                scale: 1.3,
            }));
            ts.push(Trigger::Pinch(PinchTrigger {
                fingers: FingerCount::new(fingers),
                direction: PinchDirection::Out,
                scale: 1.3,
            }));
            ts.push(Trigger::PinchRotate(PinchRotateTrigger {
                fingers: FingerCount::new(fingers),
                distance: Distance::new(100),
            }));
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
            println!("triggered: {:?}", triggers[i]);
        }
    }
}
