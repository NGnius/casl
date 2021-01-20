use regex::Regex;

use crate::config::{Mapping};

pub trait ITextPreprocessor {
    fn process(&self, input: & str) -> String;
}

#[derive(Clone)]
pub struct Remapper {
    mappings: Vec<Mapping>,
}

impl Remapper {
    pub fn from_mappings(maps: &[Mapping]) -> Remapper {
        Remapper {
            mappings: maps.to_vec(),
        }
    }
}

impl ITextPreprocessor for Remapper {
    fn process(&self, input: &str) -> String {
        for map in &self.mappings {
            let re = Regex::new(&map.search)
                .expect(&format!("Failed to compile the regex `{}`", map.search));
            if re.is_match(input) {
                let replacement: &str = &map.replace.clone();
                return re.replace_all(input, replacement).to_string();
            }
        }
        input.to_string()
    }
}