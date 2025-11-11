use std::{fs, process};
use ttf_parser::Face;

fn main() {
    // Load font file
    let font_data = match fs::read("C:/Windows/Fonts/arialuni.ttf") {
        Ok(res) => res,
        Err(err) => {
            println!("Error: {}", err);
            process::exit(1);
        }
    };

   // Font metric
    let face = Face::parse(&font_data, 0).expect("TTF parse");
    // Parse with ttf-parser for glyph indices + metrics
    let units_per_em = face.units_per_em() as f32;

    println!("units_per_em: {}", units_per_em);
}
