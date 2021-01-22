use serde::{Deserialize, Serialize};
use crate::action::{IAction, NoAction, ShellAction};
use crate::casl_action::CASLAction;

// Payload JSON which is sent to command
#[derive(Serialize, Deserialize, Clone)]
pub struct Payload {
    pub text: String,
}

// Response JSON which is received from command
#[derive(Serialize, Deserialize, Clone)]
pub struct Response {
    pub error: Option<String>,
    pub action: CommandAction
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum CommandAction {
    Custom {
        // does nothing
        // (assume custom action is done by command that sent this response action)
    },
    Shell { // run shell command
        command: String,
        shell: Option<String>,
    },
    CASL { // CASL-specific actions
        operation: String,
        parameters: Vec<String>,
    },
    // TODO add more actions
}

impl CommandAction {
    pub fn action(&self) -> Box<dyn IAction> {
        match self {
            CommandAction::Custom { .. } => Box::new(NoAction::new(self)),
            CommandAction::Shell { .. } => Box::new(ShellAction::new(self)),
            CommandAction::CASL { .. } => Box::new(CASLAction::new(self)),
        }
    }
}