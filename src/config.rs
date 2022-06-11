mod key;
mod trigger;

use crate::action_sink as action;
use crate::gesture_event::trigger as gesture;
use trigger::Trigger;

use serde::Deserialize;

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct Config {
    /// Distance for fingers to travel to trigger. Default: 100
    #[serde(default = "default_distance")]
    swipe_distance: u32,

    /// Distance for fingers to travel to trigger. Default: 100
    #[serde(default = "default_distance")]
    shear_distance: u32,

    /// Scale to achieve to trigger. Default: 1.4
    #[serde(default = "default_pinch")]
    pinch_distance: f64,

    /// Scale to achieve to trigger. Default: 1.4
    #[serde(default = "default_rotation")]
    rotation_distance: f64,

    /// Triggers executed with any display manager and any window
    #[serde(default = "default_triggers")]
    global_triggers: Vec<ConfigTrigger>,

    /// Triggers executed in x11 on any window
    #[serde(default = "default_triggers")]
    x11_triggers: Vec<ConfigTrigger>,

    /// Triggers executed in wayland on any window
    #[serde(default = "default_triggers")]
    wayland_triggers: Vec<ConfigTrigger>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct ConfigTrigger {
    pub trigger: Trigger,
    pub action: ConfigAction,
}

#[derive(PartialEq, Debug, Deserialize)]
pub enum ConfigAction {
    UinputAction {
        modifiers: Vec<key::ConfigKey>,
        sequence: Vec<key::ConfigKey>,
    },
}

impl ConfigAction {
    pub fn make(
        self,
        input_device: &std::rc::Rc<std::cell::RefCell<uinput::Device>>,
    ) -> Box<dyn action::Action> {
        match self {
            ConfigAction::UinputAction {
                modifiers,
                sequence,
            } => Box::new(action::UinputAction {
                device: input_device.clone(),
                modifiers: modifiers.iter().map(|x| x.0).collect(),
                sequence: sequence.iter().map(|x| x.0).collect(),
            }),
        }
    }
}

/* Impls */

impl Config {
    pub fn load<P>(path: P) -> std::io::Result<Config>
    where
        P: AsRef<std::path::Path> + std::fmt::Display,
    {
        log::trace!("Reading {}", path);
        let s = std::fs::read_to_string(path).map_err(|e| {
            log::error!("Error reading: {}", e);
            e
        })?;
        ron::from_str(&s).map_err(|e| {
            log::error!("Error decoding RON: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })
    }

    pub fn make_triggers(
        self,
        is_wayland: bool,
    ) -> (Vec<gesture::Trigger>, Vec<Box<dyn action::Action>>) {
        let input_device = action::UinputAction::default_device();
        self.global_triggers
            .into_iter()
            .chain(
                if is_wayland {
                    self.wayland_triggers
                } else {
                    self.x11_triggers
                }
                .into_iter(),
            )
            .map(|x| {
                (
                    x.trigger.make(
                        self.swipe_distance,
                        self.shear_distance,
                        self.pinch_distance,
                        self.rotation_distance,
                    ),
                    x.action.make(&input_device),
                )
            })
            .unzip()
    }
}

/* Serde defaults */

fn default_distance() -> u32 {
    log::debug!("Using default distance");
    100
}
fn default_pinch() -> f64 {
    log::debug!("Using default pinch");
    1.4
}
fn default_rotation() -> f64 {
    log::debug!("Using default rotation");
    60.0
}
fn default_triggers() -> Vec<ConfigTrigger> {
    log::debug!("Using default triggers");
    Vec::new()
}
