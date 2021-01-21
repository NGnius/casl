use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Payload {
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Response {
    pub error: Option<String>,
    pub action: CommandAction
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum CommandAction {
    // TODO
}