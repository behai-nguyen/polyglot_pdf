// 14/09/2025

use std::fs;
use ttf_parser::Face;
use rustybuzz::{Face as RbFace, UnicodeBuffer, GlyphBuffer};

fn main() {
    let input_font_file = if cfg!(target_os = "windows") {
        "C:/Windows/Fonts/arialuni.ttf"
    } else {
        "/home/behai/Noto_Sans_TC/NotoSansTC-Regular.ttf"
    };

    // Load font file into memory
    let font_data = fs::read(input_font_file).expect("cannot read font");

    // Parse with ttf-parser
    let face = Face::parse(&font_data, 0).expect("bad font");

    // Wrap for rustybuzz shaping
    let rb_face = RbFace::from_slice(&font_data, 0).expect("bad rb face");

    // Text to measure
    let text = "Kỷ độ Long Tuyền đới nguyệt ma.";

    // Shape text
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    let glyph_buffer: GlyphBuffer = rustybuzz::shape(&rb_face, &[], buffer);

    // Collect advances (in font units)
    let units_per_em = face.units_per_em() as f32;
    let font_size_pt = 12.0; // desired size in PostScript points

    let scale = font_size_pt / units_per_em;

    let mut total_advance = 0.0;

    for glyph in glyph_buffer.glyph_positions() {
        // Each advance is in font units
        total_advance += glyph.x_advance as f32 * scale;
    }

    println!(
        "\"{}\" in {} pt {} is {:.2} pt wide",
        text, font_size_pt, input_font_file, total_advance
    );
}