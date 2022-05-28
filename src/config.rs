mod key;

use crate::action_sink as action;
use crate::gesture_event::trigger;
use serde::Deserialize;

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct Config {
    triggers: Vec<ConfigTrigger>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct ConfigTrigger {
    pub trigger: trigger::Trigger,
    pub action: ConfigAction,
}

#[derive(PartialEq, Debug, Deserialize)]
pub enum ConfigAction {
    UinputAction(UinputAction),
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct UinputAction {
    pub modifiers: Vec<key::ConfigKey>,
    pub sequence: Vec<key::ConfigKey>,
}

impl ConfigAction {
    pub fn make(
        self,
        input_device: &std::rc::Rc<std::cell::RefCell<uinput::Device>>,
    ) -> Box<dyn action::Action> {
        match self {
            ConfigAction::UinputAction(a) => Box::new(action::UinputAction {
                device: input_device.clone(),
                modifiers: a.modifiers.iter().map(|x| x.0).collect(),
                sequence: a.sequence.iter().map(|x| x.0).collect(),
            }),
        }
    }
}

impl Config {
    pub fn load<P>(path: P) -> std::io::Result<Config>
    where
        P: AsRef<std::path::Path> + std::fmt::Display,
    {
        log::trace!("Reading {}", path);
        let s = std::fs::read_to_string(path).map_err(|e| {
            log::debug!("Error reading: {}", e);
            e
        })?;
        ron::from_str(&s).map_err(|e| {
            log::debug!("Error decoding RON: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })
    }

    pub fn combine(self, mut other: Self) -> Self {
        let mut triggers = self.triggers;
        triggers.append(&mut other.triggers);
        Self { triggers }
    }

    pub fn make_triggers(self) -> (Vec<trigger::Trigger>, Vec<Box<dyn action::Action>>) {
        let input_device = action::UinputAction::default_device();
        self.triggers
            .into_iter()
            .map(|x| (x.trigger, x.action.make(&input_device)))
            .unzip()
    }
}
