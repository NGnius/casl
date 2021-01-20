use serde::{Deserialize, Serialize};
use crate::preprocessor::{ITextPreprocessor, SimpleMapper, RedirectConfig};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: String,
    pub scorer: Option<String>,
    pub carryover_buffer_size: usize,
    pub refresh_buffer_threshold: usize,
    pub gap_detection_ms: usize,
    pub preprocessors: Vec<PreprocessorConfig>,
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
