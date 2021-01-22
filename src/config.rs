use serde::{Deserialize, Serialize};
use crate::preprocessor::{ITextPreprocessor, SimpleMapper, RedirectConfig};
use crate::command::{ICommand, SocketCommand, StdIOCommand, ShellCommand, RedirectCommand, AutoActionCommand};
use std::collections::HashMap;
use regex::{RegexBuilder};
use crate::command_api::CommandAction;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: String,
    pub scorer: Option<String>,
    pub carryover_buffer_size: usize,
    pub refresh_buffer_threshold: usize,
    pub gap_detection_ms: usize,
    pub preprocessors: Vec<PreprocessorConfig>,
    pub commands: Vec<CommandConfig>,
}

impl Config {

}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PreprocessorConfig {
    Remap {
        mappings: HashMap<String, String>,
    },
    Redirect {
        path: String,
    }
}

impl PreprocessorConfig {
    pub fn preprocessor(&self) -> Box<dyn ITextPreprocessor> {
        match self {
            PreprocessorConfig::Remap { mappings } => {
                return Box::new(SimpleMapper::from_mappings(mappings));
            },
            PreprocessorConfig::Redirect { path } => {
                return Box::new(RedirectConfig::from_path(std::path::Path::new(path)));
            }
        }
    }
}

impl Clone for PreprocessorConfig {
    fn clone(&self) -> Self {
        match self {
            PreprocessorConfig::Remap { mappings} => PreprocessorConfig::Remap {
                mappings: mappings.clone(),
            },
            PreprocessorConfig::Redirect { path } => PreprocessorConfig::Redirect {
                path: path.clone(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum CommandConfig {
    Net {
        precondition: String,
        use_raw_text: bool,
        dst_port: usize,
        src_port: usize,
        src_addr: Option<String>,
        dst_addr: String,
    },
    StdIO {
        precondition: String,
        command: String,
        use_raw_text: bool,
    },
    Shell { /* !! Does not use API !! */
        precondition: String,
        command: String,
        shell: String,
        use_raw_text: bool,
    },
    Action {
        precondition: String,
        use_raw_text: bool,
        action: CommandAction,
    },
    Redirect {
        precondition: String,
        use_raw_text: bool,
        path: String,
    }
}

impl CommandConfig {
    pub fn command(&self) -> Box<dyn ICommand> {
        match self {
            CommandConfig::Net { .. } => Box::new(SocketCommand::new(self)),
            CommandConfig::StdIO { .. } => Box::new(StdIOCommand::new(self)),
            CommandConfig::Shell { .. } => Box::new(ShellCommand::new(self)),
            CommandConfig::Redirect { .. } => Box::new(RedirectCommand::new(self)),
            CommandConfig::Action { .. } => Box::new(AutoActionCommand::new(self)),
        }
    }

    pub fn use_raw(&self) -> bool {
        match self {
            CommandConfig::Net { use_raw_text, .. } => *use_raw_text,
            CommandConfig::StdIO { use_raw_text, .. } => *use_raw_text,
            CommandConfig::Shell { use_raw_text, .. } => *use_raw_text,
            CommandConfig::Redirect { use_raw_text, .. } => *use_raw_text,
            CommandConfig::Action { use_raw_text, .. } => *use_raw_text,
        }
    }

    pub fn is_match(&self, text: &str) -> bool {
        let precondition = match self {
            CommandConfig::Net { precondition, .. } => precondition,
            CommandConfig::StdIO { precondition, .. } => precondition,
            CommandConfig::Shell { precondition, .. } => precondition,
            CommandConfig::Redirect { precondition, .. } => precondition,
            CommandConfig::Action { precondition, .. } => precondition,
        };
        let re = RegexBuilder::new(precondition)
            .case_insensitive(true)
            .build()
            .expect(&format!("Failed to compile the regex {} for CommandConfig", precondition));
        re.is_match(text)
    }
}
