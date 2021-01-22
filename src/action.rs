use crate::command_api::CommandAction;
use std::process::Stdio;

pub trait IAction {
    fn act(&self);
}

#[derive(Clone)]
pub struct NoAction {}

impl NoAction {
    pub fn new(_conf: &CommandAction) -> NoAction {
        NoAction{}
    }
}

impl IAction for NoAction {
    fn act(&self) {}
}

#[derive(Clone)]
pub struct ShellAction {
    shell: String,
    command: String,
}

impl ShellAction {
    pub fn new(conf: &CommandAction) -> ShellAction {
        if let CommandAction::Shell {shell, command} = conf {
            ShellAction {
                command: command.clone(),
                shell: shell.clone().unwrap_or("/bin/sh".to_owned()),
            }
        } else {panic!("Non-Shell command action given to ShellAction");}
    }
}

impl IAction for ShellAction {
    fn act(&self) {
        std::process::Command::new(&self.shell)
            .arg("-c")
            .arg(&self.command)
            .stdout(Stdio::null()) // TODO pipe stdio somewhere
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect(&format!("Failed to run {} action {}", &self.shell, &self.command));
    }
}