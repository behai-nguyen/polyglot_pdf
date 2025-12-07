// 14/10/2025

/// Represents a working font program.
#[derive(Default, Debug, Clone)]
pub struct FontInfo {
    /// Absolute path of the font program file.
    path: String,
    /// See PDF 1.7 Reference Document: 
    ///     https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf
    /// section "9.6.4 Font Subsets", page 258.    
    embedded_name: String,
    /// Index of the font in the font program file.
    face_index: u32,
}

impl FontInfo {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn embedded_name(&self) -> &str {
        &self.embedded_name
    }

    pub fn face_index(&self) -> u32 {
        self.face_index
    }
}

#[cfg(target_os = "linux")]
/// Hardcoded some font program files to keep the illustration simple.
/// In a production setting, I imagine we need to do discovery?
/// Only use "NotoSansTC-Regular".
const FONT_PATHS: &[(&str, &str, u32)] = &[
    ("Arial Unicode MS", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0),
    ("Arial", "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf", 0),
    ("Noto Sans CJK", "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc", 0),
    ("NotoSansCJK-Regular", "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 2),
    ("NotoSansTC-Regular", "/home/behai/Noto_Sans_TC/NotoSansTC-Regular.ttf", 0),
];

#[cfg(target_os = "windows")]
/// Hardcoded some font program files to keep the illustration simple.
/// In a production setting, I imagine we need to do discovery?
/// We use only "Arial Unicode MS".
const FONT_PATHS: &[(&str, &str, u32)] = &[
    ("Arial", "C:/Windows/Fonts/arial.ttf", 0),
    ("Arial Unicode MS", "C:/Windows/Fonts/arialuni.ttf", 0),
    ("Blade Runner Movie Font", "c:/windows/fonts/bladrmf_.ttf", 0),
    ("Times New Romman", "C:/Windows/Fonts/times.ttf", 0),
    ("Tahoma", "C:/Windows/Fonts/tahoma.ttf", 0),
    ("NotoSansCJK-Regular", "F:/rust/polyglot_pdf/font/NotoSansCJK-Regular.ttc", 7),
    ("NotoSansCJK-Regular-2", "F:/rust/polyglot_pdf/font/NotoSansCJK-Regular_2.ttf", 0),
    ("NotoSansCJK-Regular-3", "F:/rust/polyglot_pdf/font/NotoSansCJK-Regular_3.ttf", 0),
    ("NotoSansCJK-Regular-7", "F:/rust/polyglot_pdf/font/NotoSansCJK-Regular_7.ttf", 0),
    ("NotoSansCJK-Regular-8", "F:/rust/polyglot_pdf/font/NotoSansCJK-Regular_8.ttf", 0),
];

fn resolve_font_path(name: &str) -> Option<(String, u32)> {
    for &(alias, path, face_index) in FONT_PATHS {
        if alias.eq_ignore_ascii_case(name) {
            return Some((path.to_string(), face_index));
        }
    }
    None
}

// See PDF 1.7 Reference Document: 
//     https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf
// section "9.6.4 Font Subsets", page 258.
fn unique_font_tag() -> String {
    use rand::{distr::Alphabetic, Rng};

    {
        rand::rng()
            .sample_iter(&Alphabetic)
            .take(6)
            .map(char::from)
            .collect::<String>()
            .to_uppercase()
    }
}

pub fn get_font_info(name: &str) -> Option<FontInfo> {
    let res = resolve_font_path(name);

    if res.is_none() { return None; }

    let font = res.unwrap();
    Some(FontInfo {
        path: font.0.to_string(),
        embedded_name: format!("{}+{}", unique_font_tag(), name),
        face_index: font.1,
    })
}