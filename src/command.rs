use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    Net {

    },
    StdIO {

    }
}