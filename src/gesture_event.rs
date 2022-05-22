use crate::input_event::{GestureProducer, InputEvent, Gesture, SwipeGesture, PinchGesture, HoldGesture};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

const VSLOPE: f64 = 1.0;
const HSLOPE: f64 = 1.0 / VSLOPE;

impl Direction {
    fn matches(&self, dx: f64, dy: f64) -> bool {
        // from running libinput: up is negative, left is positive
        match self {
            Direction::Up =>
                dy <= VSLOPE * dx && dy <= -VSLOPE * dx,
            Direction::Down =>
                dy >= VSLOPE * dx && dy >= -VSLOPE * dx,
            Direction::Right =>
                dx >= HSLOPE * dy && dx >= -HSLOPE * dy,
            Direction::Left =>
                dx <= HSLOPE * dy && dx <= -HSLOPE * dy,
        }
    }
}

/// In means scale goes 1.0 -> 1.5
/// Out means scale goes 1.0 -> 0.5
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PinchDirection {
    In, Out
}

impl PinchDirection {
    fn signum(&self) -> f64 {
        // sign obtained from running libinput
        match self {
            PinchDirection::In => 1.0,
            PinchDirection::Out => -1.0,
        }
    }
}

// i32 for easier comparing with raw events, but create from unsigned
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct FingerCount(i32);
impl FingerCount {
    pub fn new(c: u32) -> Self {
        FingerCount(c.try_into().expect("You don't have that much fingers!"))
    }
}

// f64 for easier comparing with raw events, but create from unsigned int
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Distance(f64);
impl Distance {
    pub fn new(d: u32) -> Self {
        Distance(d.try_into().expect("I though u32 -> f64 doesn't fail"))
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Trigger {
    Swipe(SwipeTrigger),
    /// Here distance means scale
    Pinch(PinchTrigger),
    /// Here distance means... something, i'm not sure how rotation works
    PinchRotate(PinchRotateTrigger),
    /// Sent only when hold ended
    Hold(HoldTrigger),
    // TODO: hold in progress. Need to track my own time, bleh
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct SwipeTrigger {
    pub fingers: FingerCount,
    pub direction: Direction,
    pub distance: Distance,
}
impl SwipeTrigger {
    fn matches(&self, gest: &SwipeGesture, o: Origin) -> bool {
        self.fingers.0 == gest.fingers
            && self.direction.matches(gest.dx - o.x, gest.dy - o.y)
            && (   (gest.dx - o.x).abs() >= self.distance.0
                || (gest.dy - o.y).abs() >= self.distance.0 )
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct PinchTrigger {
    pub fingers: FingerCount,
    pub direction: PinchDirection,
    pub scale: f64,
}
impl PinchTrigger {
    fn matches(&self, gest: &PinchGesture, origin: f64) -> bool {
        println!("consider {:?}, adjusted {}", gest, labs(gest.scale / origin));
        self.fingers.0 == gest.fingers
            && self.scale <= labs(gest.scale / origin)
            && lsign(gest.scale) == self.direction.signum()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct PinchRotateTrigger {
    pub fingers: FingerCount,
    /// FIXME: what does distance even mean here? I tested a little and I don't
    /// get it
    pub distance: Distance,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct HoldTrigger {
    pub fingers: FingerCount,
    pub time: u32,
}
impl HoldTrigger {
    fn matches(&self, gest: &HoldGesture, ctime: u32) -> bool {
        self.fingers.0 == gest.fingers
            && ctime.saturating_sub(gest.begin_time) >= self.time
    }
}

/// Adapt raw gesture events into triggers
pub struct EventAdapter {
    source: GestureProducer,
    triggers: Vec<Trigger>,
    /// When trigger has happened, adjust the event displacements for triggers in
    /// other directions
    adjust: Origin,
}

#[derive(PartialEq, Debug, Clone, Copy)]
struct Origin {
    x: f64,
    y: f64,
    scale: f64,
}

impl EventAdapter {
    pub fn new(source: GestureProducer, triggers: &Vec<Trigger>) -> Self {
        EventAdapter {
            source,
            triggers: (*triggers).clone(),
            adjust: Origin { x: 0.0, y: 0.0, scale: 1.0 },
        }
    }

    /// returns index of matched trigger
    fn adapt(&mut self, event: InputEvent) -> Vec<usize> {
        let (gesture, ctime, ended) = match event {
            InputEvent::Ongoing(g, t) => (g, t, false),
            InputEvent::Ended(g, t) => (g, t, false),
            InputEvent::Cancelled(_, _) => return Vec::new(),
        };
        // first collect matching indicies
        let inds = self.triggers.iter().enumerate().filter_map(|(i, t)| {
            match (&gesture, t) {
                (Gesture::None, _) => false,
                (Gesture::Swipe(gs), Trigger::Swipe(ts)) =>
                    ts.matches(gs, self.adjust),
                (Gesture::Swipe(_), _) => false,
                (Gesture::Pinch(gp), Trigger::Pinch(tp)) =>
                    tp.matches(gp, self.adjust.scale),
                (Gesture::Pinch(_gp), Trigger::PinchRotate(_tp)) =>
                    false, // TODO
                (Gesture::Pinch(_), _) => false,
                (Gesture::Hold(gh), Trigger::Hold(th)) =>
                    th.matches(gh, ctime),
                (Gesture::Hold(_), _) => false
            }.then(|| i)
        }).collect::<Vec<usize>>();
        // adjust the origin from triggers
        if ended {
            // adjust to neutral when end
            self.adjust = Origin {
                x: 0.0,
                y: 0.0,
                scale: 1.0,
            }
        } else {
            // adjust the origin from the triggers. Only adjust once in each
            // direction, in case several triggers were in one direction
            let mut adjusted_h = false;
            let mut adjusted_v = false;
            let mut adjusted_s = false;
            for i in &inds {
                match self.triggers[*i] {
                    Trigger::Swipe(t)
                        if t.direction == Direction::Up && !adjusted_v => {
                            adjusted_v = true;
                            // up is negative
                            self.adjust.y -= t.distance.0;
                        }
                    Trigger::Swipe(t)
                        if t.direction == Direction::Down && !adjusted_v => {
                            adjusted_v = true;
                            // down is positive
                            self.adjust.y += t.distance.0;
                        }
                    Trigger::Swipe(t)
                        if t.direction == Direction::Left && !adjusted_h => {
                            adjusted_h = true;
                            // left is positive
                            self.adjust.x -= t.distance.0;
                        }
                    Trigger::Swipe(t)
                        if t.direction == Direction::Right && !adjusted_h => {
                            adjusted_h = true;
                            // right is negative
                            self.adjust.x += t.distance.0;
                        }
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

impl Iterator for EventAdapter {
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

/// sign of logarithm
fn lsign(x: f64) -> f64 {
    if x >= 1.0 {
        1.0
    } else if x <= 1.0 {
        -1.0
    } else {
        0.0
    }
}
/// logarithmic abs
fn labs(x: f64) -> f64 {
    if x >= 1.0 {
        x
    } else if x == 0.0 {
        0.0
    } else {
        1.0 / x
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn swipe_up_down() {
        let trigger_up = super::Trigger::Swipe(super::SwipeTrigger {
            fingers: super::FingerCount::new(3),
            direction: super::Direction::Up,
            distance: super::Distance::new(200),
        });
        let trigger_down = super::Trigger::Swipe(super::SwipeTrigger {
            fingers: super::FingerCount::new(3),
            direction: super::Direction::Down,
            distance: super::Distance::new(200),
        });
        let mut adapter = super::EventAdapter::new (
            crate::input_event::GestureProducer::new(),
            vec![trigger_up, trigger_down],
        );

        use crate::input_event::*;
        let event_up_half = InputEvent::Ongoing(
            Gesture::Swipe(SwipeGesture {
                begin_time: 0,
                fingers: 3,
                dx: 10.0,
                dy: -101.0
            }),
            10,
        );
        let event_up = InputEvent::Ongoing(
            Gesture::Swipe(SwipeGesture {
                begin_time: 0,
                fingers: 3,
                dx: -20.0,
                dy: -202.0
            }),
            10,
        );
        let event_down = InputEvent::Ongoing(
            Gesture::Swipe(SwipeGesture {
                begin_time: 0,
                fingers: 3,
                dx: 30.0,
                dy: 10.0
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
