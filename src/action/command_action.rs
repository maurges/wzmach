use std::os::unix::prelude::CommandExt;

use super::{Action, ActionError};

pub struct CommandAction {
    pub path: String,
    pub args: Vec<String>,
}

pub struct ShellCommandAction {
    pub command: String,
}

impl Action for CommandAction {
    fn execute(&mut self) -> Result<(), ActionError> {
        log::debug!("Execute command {} {:?}", self.path, self.args);

        std::process::Command::new(&self.path)
            .args(self.args.iter())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .env_remove("DBUS_SYSTEM_BUS_ADDRESS")
            .detach()?;
        log::trace!("Spawned the command");

        Ok(())
    }
}

impl Action for ShellCommandAction {
    fn execute(&mut self) -> Result<(), ActionError> {
        log::debug!("Execute command {:?}", self.command);

        std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(&self.command)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .env_remove("DBUS_SYSTEM_BUS_ADDRESS")
            .detach()?;
        log::trace!("Spawned the command");

        Ok(())
    }
}

/// Extension to be able to detach child processes without creating zombies
trait DetachExt {
    fn detach(&mut self) -> std::io::Result<()>;
}
impl DetachExt for std::process::Command {
    fn detach(&mut self) -> std::io::Result<()> {
        // Safety: usual daemonization stuff. Parent exits immediately before
        // exec, child goes on to exec what it wants
        use nix::unistd::ForkResult;
        unsafe {
            self.pre_exec(|| {
                match nix::unistd::fork().unwrap() {
                    ForkResult::Parent { child: _ } => std::process::exit(0),
                    ForkResult::Child => (),
                }
                Ok(())
            })
        }
        .status()
        .map(|_| ())
    }
}

impl From<std::io::Error> for ActionError {
    fn from(err: std::io::Error) -> ActionError {
        ActionError(format!("{}", err))
    }
}
