/// High-level gesture completion events, produced from observing low-level
/// gesture events. Register your 'Trigger's for events and observe them
/// triggered
pub mod trigger;
use trigger::{CardinalTrigger, Direction, Origin, PinchDirection, Trigger};

use crate::input_producer::event::{Gesture, InputEvent};
use sorted_vec::SortedSet;

/// Adapt low-level gesture events into high-level events by triggers
pub struct EventAdapter<T: Iterator<Item = InputEvent>> {
    source: T,
    triggers: Vec<Trigger>,
    /// When trigger has happened, adjust the event displacements for triggers in
    /// other directions
    adjust: Origin,
    triggered: SortedSet<usize>,
}

impl<T: Iterator<Item = InputEvent>> EventAdapter<T> {
    /// Create event source from a low-level source. The created adapter will
    /// observe the given triggers. If the triggers conflict, the harder ones
    /// may never trigger
    pub fn new(source: T, triggers: &Vec<Trigger>) -> Self {
        EventAdapter {
            source,
            triggers: (*triggers).clone(),
            adjust: Origin {
                x: 0.0,
                y: 0.0,
                scale: 1.0,
            },
            triggered: SortedSet::new(),
        }
    }

    /// Returns index of matched trigger
    fn adapt(&mut self, event: InputEvent) -> Vec<usize> {
        let (gesture, ctime, ended) = match event {
            InputEvent::Ongoing(g, t) => (g, t, false),
            InputEvent::Ended(g, t) => (g, t, true),
            InputEvent::Cancelled(_, t) => (Gesture::None, t, true),
        };
        // first collect matching indicies
        let inds = self.triggers.iter().enumerate().filter_map(|(i, t)| {
            match (&gesture, t) {
                (Gesture::None, _) => false,
                (Gesture::Swipe(gs), Trigger::Swipe(ts)) => ts.matches_swipe(gs, self.adjust),
                (Gesture::Swipe(_), _) => false,

                (Gesture::Pinch(gp), Trigger::Pinch(tp)) => tp.matches(gp, self.adjust.scale),
                (Gesture::Pinch(gs), Trigger::Shear(ts)) => ts.matches_shear(gs, self.adjust),
                (Gesture::Pinch(_), _) => false,

                (Gesture::Hold(gh), Trigger::Hold(th)) => th.matches(gh, ctime),
                (Gesture::Hold(_), _) => false,
            }
            .then(|| i)
        });
        // remove the ones that were triggered and are not repeated
        let inds = inds
            .filter(|i| {
                if !self.triggers[*i].repeated() {
                    match self.triggered.find_or_insert(*i) {
                        sorted_vec::FindOrInsert::Found(_present_at) => false,
                        sorted_vec::FindOrInsert::Inserted(_inserted_at) => true,
                    }
                } else {
                    true
                }
            })
            .collect::<Vec<usize>>();
        if ended {
            // adjust to neutral when end
            self.adjust = Origin {
                x: 0.0,
                y: 0.0,
                scale: 1.0,
            };
            // we can retrigger everything again
            self.triggered = sorted_vec::SortedSet::new();
        } else {
            // adjust the origin from the triggers. Only adjust once in each
            // direction, in case several triggers were in one direction
            let mut adjusted_h = false;
            let mut adjusted_v = false;
            let mut adjusted_s = false;
            let mut adjust_directional = |t: CardinalTrigger| {
                if t.direction == Direction::Up && !adjusted_v {
                    adjusted_v = true;
                    // up is negative
                    self.adjust.y -= t.distance.0;
                } else if t.direction == Direction::Down && !adjusted_v {
                    adjusted_v = true;
                    // down is positive
                    self.adjust.y += t.distance.0;
                } else if t.direction == Direction::Left && !adjusted_h {
                    adjusted_h = true;
                    // left is positive
                    self.adjust.x -= t.distance.0;
                } else if t.direction == Direction::Right && !adjusted_h {
                    adjusted_h = true;
                    // right is negative
                    self.adjust.x += t.distance.0;
                }
            };
            for i in &inds {
                match self.triggers[*i] {
                    Trigger::Swipe(t) => adjust_directional(t),
                    Trigger::Shear(t) => adjust_directional(t),
                    Trigger::Pinch(t) if !adjusted_s => {
                        adjusted_s = true;
                        match t.direction {
                            PinchDirection::In => {
                                self.adjust.scale *= t.scale;
                            }
                            PinchDirection::Out => {
                                self.adjust.scale /= t.scale;
                            }
                        }
                    }

                    _ => (),
                }
            }
        }
        inds
    }
}

impl<T: Iterator<Item = InputEvent>> Iterator for EventAdapter<T> {
    type Item = Vec<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        // should I maybe yield all the empty events?
        loop {
            if let Some(event) = self.source.next() {
                let r = self.adapt(event);
                if r.len() != 0 {
                    break Some(r);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::gesture_event::trigger::{
        CardinalTrigger, Direction, Distance, FingerCount, Trigger,
    };

    #[test]
    fn swipe_up_down() {
        let trigger_up = Trigger::Swipe(CardinalTrigger {
            fingers: FingerCount::new(3),
            direction: Direction::Up,
            distance: Distance::new(200),
            repeated: false,
        });
        let trigger_down = Trigger::Swipe(CardinalTrigger {
            fingers: FingerCount::new(3),
            direction: Direction::Down,
            distance: Distance::new(200),
            repeated: false,
        });
        let mut adapter = super::EventAdapter::new(
            crate::input_producer::GestureProducer::new(),
            &vec![trigger_up, trigger_down],
        );

        use crate::input_producer::event::*;
        let event_up_half = InputEvent::Ongoing(
            Gesture::Swipe(SwipeGesture {
                begin_time: 0,
                fingers: 3,
                dx: 10.0,
                dy: -101.0,
            }),
            10,
        );
        let event_up = InputEvent::Ongoing(
            Gesture::Swipe(SwipeGesture {
                begin_time: 0,
                fingers: 3,
                dx: -20.0,
                dy: -202.0,
            }),
            10,
        );
        let event_down = InputEvent::Ongoing(
            Gesture::Swipe(SwipeGesture {
                begin_time: 0,
                fingers: 3,
                dx: 30.0,
                dy: 10.0,
            }),
            20,
        );

        let r = adapter.adapt(event_down.clone());
        assert_eq!(r, Vec::new());
        let r = adapter.adapt(event_up_half.clone());
        assert_eq!(r, Vec::new());
        let r = adapter.adapt(event_up.clone());
        assert_eq!(r, vec![0]);
        let r = adapter.adapt(event_up_half.clone());
        assert_eq!(r, Vec::new());
        let r = adapter.adapt(event_down.clone());
        assert_eq!(r, vec![1]);
    }
}
