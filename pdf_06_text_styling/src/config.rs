/// 26/01/2026
/// 
/// Configuration.
///

use serde::Deserialize;
use std::fs;
use toml;

#[derive(Debug, Deserialize)]
pub struct FontSpec {
    family: String,
    size: i32,
    weight: String,
    style: String,
}

#[derive(Debug, Deserialize)]
pub struct FontConfig {
    paragraph: FontSpec,
    headers: Vec<FontSpec>,
    page_number: FontSpec,
}

#[derive(Deserialize)]
struct Config {
    fonts: FontConfig,
}

impl FontSpec {
    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn size(&self) -> i32 {
        self.size
    }
    pub fn weight(&self) -> &str {
        &self.weight
    }

    pub fn style(&self) -> &str {
        &self.style
    }
}

impl FontConfig {
    pub fn paragraph(&self) -> &FontSpec {
        &self.paragraph
    }

    pub fn headers(&self) -> &[FontSpec] {
        &self.headers
    }

    pub fn page_number(&self) -> &FontSpec {
        &self.page_number
    }
}

pub fn load_font_config(file_path: &str) -> Result<FontConfig, Box<dyn std::error::Error>> {
    let config: Config = toml::from_str(&fs::read_to_string(file_path)?)?;
    Ok(config.fonts)
}