// 24/11/2025

pub struct PageSize {
    pub width: f64,
    pub height: f64,
}

pub struct PageMargins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
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
    // left: 200.0,
};

pub fn a4_default_content_width() -> f64 {
    A4.width - A4_DEFAULT_MARGINS.right - A4_DEFAULT_MARGINS.left
}

pub fn a4_default_content_height() -> f64 {
    A4.height - A4_DEFAULT_MARGINS.top - A4_DEFAULT_MARGINS.bottom
}