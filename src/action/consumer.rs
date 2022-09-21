use super::{Action, ActionError};

/// Iterator adaptor inteface for executing events. Now you can use a
/// haskell-conduit like interface
pub trait EventConsumerExt: Iterator<Item = Vec<usize>> {
    fn consume_events(self, actions: &mut [Box<dyn Action>]);
}

impl<I: Iterator<Item = Vec<usize>>> EventConsumerExt for I {
    fn consume_events(self, actions: &mut [Box<dyn Action>]) {
        for action_inds in self {
            for index in action_inds {
                match actions[index].execute() {
                    Ok(()) => (),
                    Err(ActionError(msg)) => log::error!("{}", msg),
                }
            }
        }
    }
}
