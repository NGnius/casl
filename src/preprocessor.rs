use regex::RegexBuilder;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::config;

pub trait ITextPreprocessor {
    fn process(&self, input: &str) -> String;
}

#[derive(Clone)]
pub struct SimpleMapper {
    mappings: HashMap<String, String>,
}

impl SimpleMapper {
    pub fn from_mappings(maps: &HashMap<String, String>) -> SimpleMapper {
        SimpleMapper {
            mappings: maps.clone(),
        }
    }
}

impl ITextPreprocessor for SimpleMapper {
    fn process(&self, input: &str) -> String {
        let mut result = String::from(input.clone());
        for (key, val) in &self.mappings {
            let re = RegexBuilder::new(key)
                .case_insensitive(true)
                .build()
                .expect(&format!("Failed to compile the regex {} for SimpleMapper", key));
            if re.is_match(&result) {
                let replacement: &str = &val.clone();
                result = re.replace_all(&result, replacement).to_string();
            }
        }
        result
    }
}

pub struct RedirectConfig {
    path: PathBuf,
    processor: Box<dyn ITextPreprocessor>,
}

impl RedirectConfig {
    pub fn from_path(path: &Path) -> RedirectConfig {
        let json_file = std::fs::File::open(path).unwrap();
        let json_reader = std::io::BufReader::new(json_file);
        let preprocessor_conf: config::PreprocessorConfig = serde_json::from_reader(json_reader)
            .expect(&("Unable to parse JSON file ".to_owned() + path.to_str().unwrap()));
        RedirectConfig {
            path: path.clone().to_owned(),
            processor: preprocessor_conf.preprocessor(),
        }
    }
}

impl ITextPreprocessor for RedirectConfig {
    fn process(&self, input: &str) -> String {
        self.processor.process(input)
    }
}

impl Clone for RedirectConfig {
    fn clone(&self) -> Self {
        RedirectConfig::from_path(&self.path)
    }
}