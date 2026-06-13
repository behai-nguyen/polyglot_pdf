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
pub struct ImageBlockConfig {
    reduction_factor: f64,
    centre_aligned: bool,
    step_scale_factor: f64,
    min_allowed_scale: f64,
}

#[derive(Debug, Deserialize)]
pub struct HeadingSpacing {
    // Array of 6 floats for H1, H2, H3, H4, H5, H6.
    before: [f64; 6],
    after: [f64; 6],
}

#[derive(Debug, Deserialize)]
pub struct ElementSpacing {
    before: f64,
    after: f64,
}

#[derive(Debug, Deserialize)]
pub struct BlockSpacingConfig {
    heading: HeadingSpacing,
    paragraph: ElementSpacing,
    image: ElementSpacing,
    thematic: ElementSpacing,
}

#[derive(Deserialize)]
pub struct ColourRGB {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Deserialize)]
pub struct HorizontalBreakConfig {
    stroke_width: f64,
    colour: ColourRGB,
}

#[derive(Deserialize)]
pub struct Config {
    fonts: FontConfig,
    image_block: ImageBlockConfig,
    block_spacing: BlockSpacingConfig,
    horizontal_break: HorizontalBreakConfig,
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

impl HeadingSpacing {
    pub fn before(&self, level: u8) -> f64 {
        self.before[(level - 1) as usize]
    }

    pub fn after(&self, level: u8) -> f64 {
        self.after[(level - 1) as usize]
    }
}

impl ElementSpacing {
    pub fn before(&self) -> f64 {
        self.before
    }

    pub fn after(&self) -> f64 {
        self.after
    }
}

impl BlockSpacingConfig {
    pub fn heading(&self) -> &HeadingSpacing {
        &self.heading
    }

    pub fn paragraph(&self) -> &ElementSpacing {
        &self.paragraph
    }

    pub fn image(&self) -> &ElementSpacing {
        &self.image
    }

    pub fn thematic(&self) -> &ElementSpacing {
        &self.thematic
    }
}

impl ColourRGB {
    pub fn r(&self) -> f64 {
        self.r
    }

    pub fn g(&self) -> f64 {
        self.g
    }

    pub fn b(&self) -> f64 {
        self.b
    }
}

impl HorizontalBreakConfig {
    pub fn stroke_width(&self) -> f64 {
        self.stroke_width
    }

    pub fn colour(&self) -> &ColourRGB {
        &self.colour
    }
}

impl Config {
    pub fn fonts(&self) -> &FontConfig {
        &self.fonts
    }

    pub fn image_block(&self) -> &ImageBlockConfig {
        &self.image_block
    }

    pub fn block_spacing(&self) -> &BlockSpacingConfig {
        &self.block_spacing
    }

    pub fn horizontal_break(&self) -> &HorizontalBreakConfig {
        &self.horizontal_break
    }
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config: Config = toml::from_str(&fs::read_to_string(file_path)?)?;
    Ok(config)
}