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

    let strs = vec![
        "Kỷ độ Long Tuyền đới nguyệt ma.",
        "幾度龍泉戴月磨。",
    ];  
    
    for str in strs {
        let mut glyphs_for_text = Vec::<u16>::new();    

        for ch in str.chars() {
            if let Some(gid) = face.glyph_index(ch) {
                glyphs_for_text.push(gid.0);
            } else {
                println!("Glyph ID not found for {ch}.");
            }            
        };

        println!("Text: [{str}].");
        println!("glyphs_for_text: {:?}", glyphs_for_text);
    }
}

/* 
Text: [Kỷ độ Long Tuyền đới nguyệt ma.].
glyphs_for_text: [46, 2985, 3, 211, 2955, 3, 47, 82, 81, 74, 3, 55, 88, 92, 2931, 81, 3, 211, 2957, 76, 3, 81, 74, 88, 92, 2937, 87, 3, 80, 68, 17]

Text: [幾度龍泉戴月磨。].
glyphs_for_text: [12557, 12597, 29212, 16216, 13507, 14743, 19319, 4589]
*/