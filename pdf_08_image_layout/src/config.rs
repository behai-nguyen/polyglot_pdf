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
    headers: Vec<FontSpec>,
    paragraph: FontSpec,
    caption: FontSpec,
    page_number: FontSpec,
}

#[derive(Debug, Deserialize)]
pub struct LayoutConfig {
    image_block_spacing: f64,
}

#[derive(Debug, Deserialize)]
pub struct ImageBlockConfig {
    reduction_factor: f64,
    centre_aligned: bool,
    step_scale_factor: f64,
    min_allowed_scale: f64,
}

#[derive(Deserialize)]
pub struct Config {
    fonts: FontConfig,
    layout: LayoutConfig,
    image_block: ImageBlockConfig,
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
    pub fn headers(&self) -> &[FontSpec] {
        &self.headers
    }

    pub fn paragraph(&self) -> &FontSpec {
        &self.paragraph
    }

    pub fn caption(&self) -> &FontSpec {
        &self.caption
    }

    pub fn page_number(&self) -> &FontSpec {
        &self.page_number
    }
}

impl LayoutConfig {
    pub fn image_block_spacing(&self) -> f64 {
        self.image_block_spacing
    }
}

impl ImageBlockConfig {
    pub fn reduction_factor(&self) -> f64 {
        self.reduction_factor
    }

    pub fn centre_aligned(&self) -> bool {
        self.centre_aligned
    }

    pub fn step_scale_factor(&self) -> f64 {
        self.step_scale_factor
    }

    pub fn min_allowed_scale(&self) -> f64 {
        self.min_allowed_scale
    }
}

impl Config {
    pub fn fonts(&self) -> &FontConfig {
        &self.fonts
    }

    pub fn layout(&self) -> &LayoutConfig {
        &self.layout
    }

    pub fn image_block(&self) -> &ImageBlockConfig {
        &self.image_block
    }
}

/*
pub fn load_font_config(file_path: &str) -> Result<FontConfig, Box<dyn std::error::Error>> {
    let config: Config = toml::from_str(&fs::read_to_string(file_path)?)?;
    Ok(config.fonts)
}
*/

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config: Config = toml::from_str(&fs::read_to_string(file_path)?)?;
    Ok(config)
}