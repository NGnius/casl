use serde::{Deserialize, Serialize};
use crate::preprocessor::{Remapper, ITextPreprocessor};

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
        mappings: Vec<Mapping>,
    },
}

impl PreprocessorConfig {
    pub fn preprocessor(&self) -> Box<dyn ITextPreprocessor> {
        match self {
            PreprocessorConfig::Remap { mappings } => {
                return Box::new(Remapper::from_mappings(mappings));
            },
        }
    }
}

impl Clone for PreprocessorConfig {
    fn clone(&self) -> Self {
        match self {
            PreprocessorConfig::Remap { mappings} => PreprocessorConfig::Remap {
                mappings: mappings.clone(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub search: String,
    pub replace: String,
    pub name: Option<String>,
}

/*impl Clone for Mapping {
    fn clone(&self) -> Self {
        Mapping {
            search: self.search.clone(),
            replace: self.replace.clone(),
        }
    }
}*/
