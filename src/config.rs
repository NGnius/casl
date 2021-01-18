use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub model: String,
    pub scorer: Option<String>,
    pub carryover_buffer_size: usize,
    pub refresh_buffer_threshold: usize,
}

impl Config {

}
