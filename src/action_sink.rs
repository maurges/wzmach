use uinput::event::keyboard::Key;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Failure executing action: {0}")]
pub struct ActionError(pub String);

pub trait Action {
    fn execute(&mut self) -> Result<(), ActionError>;
}

pub struct UinputAction {
    pub device: std::rc::Rc<std::cell::RefCell<uinput::Device>>,
    pub modifiers: Vec<Key>,
    pub sequence: Vec<Key>,
}

impl UinputAction {
    pub fn default_device() -> std::rc::Rc<std::cell::RefCell<uinput::Device>> {
        let device = uinput::open("/dev/uinput").unwrap()
            .name("wzmach_virtual").unwrap()
            .event(uinput::event::Keyboard::All).unwrap()
            .create().unwrap();
        std::rc::Rc::new(std::cell::RefCell::new(device))
    }
}

impl From<uinput::Error> for ActionError {
    fn from(err: uinput::Error) -> ActionError {
        ActionError(format!("{}", err))
    }
}

impl Action for UinputAction {
    fn execute(&mut self) -> Result<(), ActionError> {
        let mut device = self.device.borrow_mut();
        log::debug!("Execute action {:?} + {:?}", self.modifiers, self.sequence);
        for modifier in &self.modifiers {
            device.press(modifier)?;
        }
        for key in &self.sequence {
            device.click(key)?;
        }
        for modifier in self.modifiers.iter().rev() {
            device.release(modifier)?;
        }
        device.synchronize()?;
        Ok(())
    }
}
