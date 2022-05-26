pub trait Action {
    fn execute(&mut self);
}

use uinput::event::keyboard::Key;

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

impl Action for UinputAction {
    fn execute(&mut self) {
        let mut device = self.device.borrow_mut();
        eprintln!("Execute action {:?} + {:?}", self.modifiers, self.sequence);
        for modifier in &self.modifiers {
            device.press(modifier).expect("warn and abort action");
        }
        for key in &self.sequence {
            device.click(key).expect("warn and abort action");
        }
        for modifier in self.modifiers.iter().rev() {
            device.release(modifier).expect("warn and abort action");
        }
        device.synchronize().unwrap();
    }
}
