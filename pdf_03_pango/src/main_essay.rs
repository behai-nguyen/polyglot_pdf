// 11/12/2025
// True justification, the left and right margins are flush.

use std::{fs, process};
use cairo::{Context, PdfSurface};
use cairo::glib::translate::ToGlibPtr;
use pango::{Layout, FontDescription, WrapMode};
use pango_sys::pango_layout_set_justify;
use pangocairo::functions::create_layout;

mod page_geometry;
use page_geometry::{
    A4,
    a4_default_content_width,
    a4_default_content_height,
    A4_DEFAULT_MARGINS,
};


pub trait LayoutExtJustify {
    fn set_justify(&self, justify: bool);
}

impl LayoutExtJustify for Layout {
    fn set_justify(&self, justify: bool) {
        unsafe {
            pango_layout_set_justify(self.to_glib_none().0, justify as i32);
        }
    }
}

fn main() {
    let (font_description, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Arial Unicode MS 12", "win_essay.pdf")
    } else {
        ("NotoSansTC-Regular 12", "ubuntu_essay.pdf")
    };

    // Read input text file.
    let text = match fs::read_to_string("./text/essay.txt") {
        Ok(str) => str,
        Err(err) => { 
            println!("{}", err.to_string());
            process::exit(1);
        }
    };

    let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name).unwrap();
    let cr = Context::new(&surface).unwrap();

    let layout = create_layout(&cr);
    layout.set_text(&text);
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    layout.set_justify(true);
    layout.set_wrap(WrapMode::WordChar);

    let desc = FontDescription::from_string(font_description);
    layout.set_font_description(Some(&desc));

    println!("total lines: {}", layout.line_count());

    let mut y = A4_DEFAULT_MARGINS.top;

    for i in 0..layout.line_count() {        
        if let Some(line) = layout.line(i) {
            let (_ink, logical) = line.extents();
            let line_height = logical.height() as f64 / pango::SCALE as f64;

            cr.move_to(A4_DEFAULT_MARGINS.left, y);
            pangocairo::functions::show_layout_line(&cr, &line);

            y += line_height;

            if y > a4_default_content_height() {
                let _ = cr.show_page(); // finish current page, start new one
                y = A4_DEFAULT_MARGINS.top;
            }
        }
    }

    surface.finish();

    println!("PDF written to: {pdf_file_name}");
}