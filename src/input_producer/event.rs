/* Gesture Information */

use input::event::gesture::{
    GestureEndEvent, GestureEventCoordinates, GestureEventTrait, GesturePinchEventTrait,
};

#[derive(PartialEq, Debug, Clone)]
pub enum Gesture {
    None,
    Swipe(SwipeGesture),
    Pinch(PinchGesture),
    Hold(HoldGesture),
}

#[derive(PartialEq, Debug, Clone)]
pub struct SwipeGesture {
    pub begin_time: u32,
    pub fingers: i32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(PartialEq, Debug, Clone)]
pub struct PinchGesture {
    pub begin_time: u32,
    pub fingers: i32,
    pub scale: f64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(PartialEq, Debug, Clone)]
pub struct HoldGesture {
    pub begin_time: u32,
    pub fingers: i32,
}

#[derive(PartialEq, Debug)]
pub enum GestureState {
    /// Arg is current event time
    Ongoing(u32),
    /// Args are: event that just finished, time of finish
    Ended(Gesture, u32),
    /// Args are: event that just finished, time of finish
    Cancelled(Gesture, u32),
}

impl Gesture {
    pub(crate) fn update(&mut self, gest: &input::event::GestureEvent) -> GestureState {
        use input::event::gesture::*;
        match gest {
            GestureEvent::Swipe(sw) => match sw {
                GestureSwipeEvent::Begin(_ev) => {
                    *self = Gesture::Swipe(SwipeGesture {
                        begin_time: gest.time(),
                        fingers: gest.finger_count(),
                        dx: 0.0,
                        dy: 0.0,
                    });
                    GestureState::Ongoing(gest.time())
                }
                GestureSwipeEvent::Update(ev) => {
                    self.update_coords(ev);
                    GestureState::Ongoing(gest.time())
                }
                GestureSwipeEvent::End(ev) => self.end_gesture(ev),
                _ => {
                    eprintln!("WARNING: swipe update from the future");
                    GestureState::Ongoing(gest.time())
                }
            },
            GestureEvent::Pinch(pc) => match pc {
                GesturePinchEvent::Begin(_ev) => {
                    *self = Gesture::Pinch(PinchGesture {
                        begin_time: gest.time(),
                        fingers: gest.finger_count(),
                        scale: pc.scale(),
                        dx: 0.0,
                        dy: 0.0,
                    });
                    GestureState::Ongoing(gest.time())
                }
                GesturePinchEvent::Update(ev) => {
                    self.update_coords(ev);
                    self.update_scale(ev);
                    GestureState::Ongoing(gest.time())
                }
                GesturePinchEvent::End(ev) => {
                    self.update_scale(ev);
                    self.end_gesture(ev)
                }
                _ => {
                    eprintln!("WARNING: pinch update from the future");
                    GestureState::Ongoing(gest.time())
                }
            },
            GestureEvent::Hold(ho) => match ho {
                GestureHoldEvent::Begin(_ev) => {
                    *self = Gesture::Hold(HoldGesture {
                        begin_time: gest.time(),
                        fingers: gest.finger_count(),
                    });
                    GestureState::Ongoing(gest.time())
                }
                GestureHoldEvent::End(_ev) => {
                    GestureState::Ended(std::mem::replace(self, Gesture::None), gest.time())
                }
                _ => {
                    eprintln!("WARNING: hold update from the future");
                    GestureState::Ongoing(gest.time())
                }
            },
            _ => {
                eprintln!("WARNING: event from the future");
                GestureState::Ongoing(gest.time())
            }
        }
    }

    fn update_coords(&mut self, upd: &dyn GestureEventCoordinates) {
        match *self {
            Gesture::Swipe(ref mut swipe) => {
                swipe.dx += upd.dx();
                swipe.dy += upd.dy();
            }
            Gesture::Pinch(ref mut pinch) => {
                pinch.dx += upd.dx();
                pinch.dy += upd.dy();
            }
            _ => eprintln!("ERROR: impossible coords update!"),
        }
    }

    fn update_scale(&mut self, upd: &dyn GesturePinchEventTrait) {
        match *self {
            Gesture::Pinch(ref mut pinch) => pinch.scale = upd.scale(),
            _ => eprintln!("ERROR: impossible scale update!"),
        }
    }

    fn end_gesture<T>(&mut self, upd: &T) -> GestureState
    where
        T: GestureEventTrait + GestureEndEvent,
    {
        if upd.cancelled() {
            GestureState::Cancelled(std::mem::replace(self, Gesture::None), upd.time())
        } else {
            GestureState::Ended(std::mem::replace(self, Gesture::None), upd.time())
        }
    }
}

// Second arg is latest time for event
#[derive(PartialEq, Debug, Clone)]
pub enum InputEvent {
    Ongoing(Gesture, u32),
    Ended(Gesture, u32),
    Cancelled(Gesture, u32),
}

impl InputEvent {
    pub(crate) fn from_state(state: GestureState, current: &Gesture) -> Self {
        match state {
            GestureState::Ongoing(time) => InputEvent::Ongoing(current.clone(), time),
            GestureState::Ended(g, t) => InputEvent::Ended(g, t),
            GestureState::Cancelled(g, t) => InputEvent::Cancelled(g, t),
        }
    }
}
