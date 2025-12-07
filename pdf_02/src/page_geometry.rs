// 24/11/2025

pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

pub struct PageMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

pub const A4: PageSize = PageSize { width: 595.22, height: 842.0 };
// pub const A3: PageSize = PageSize { width: 842.0, height: 1191.0 };

// Margin: 20mm. 20 x (72 / 25.4) = 57 Postscript point.
pub const A4_DEFAULT_MARGINS: PageMargins = PageMargins {
    top: 57.0,
    // top: 230.0,
    right: 57.0,
    // right: 200.0,
    bottom: 57.0,
    // bottom: 114.0,
    left: 57.0,
    // left: 150.0,
};

// 20% extra spacing
pub const LINE_SPACING_FACTOR: f32 = 1.2;

pub fn a4_default_content_width() -> f32 {
    A4.width - A4_DEFAULT_MARGINS.right - A4_DEFAULT_MARGINS.left
}

pub fn a4_default_content_height() -> f32 {
    A4.height - A4_DEFAULT_MARGINS.top
}