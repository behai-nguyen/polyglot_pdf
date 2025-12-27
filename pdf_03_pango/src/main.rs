// 11/12/2025
// True justification, the left and right margins are flush.
// Page number correctly aligned in the middle of the effective page width.

use std::{fs, process};
use cairo::{Context, PdfSurface};
use cairo::glib::translate::ToGlibPtr;
use pango::{Layout, FontDescription, WrapMode};
use pango_sys::pango_layout_set_justify;
use pangocairo::functions::*;

mod page_geometry;
use page_geometry::{
    A4,
    A4_DEFAULT_MARGINS,
    a4_default_content_width,
    a4_default_content_height,
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

fn total_pages(layout: &Layout, 
    margin_top: f64, 
    page_height: f64
) -> usize {
    let mut y: f64 = margin_top;
    let mut result: usize = 0;
	
    for i in 0..layout.line_count() {
        if let Some(line) = layout.line(i) {
            let (_ink, logical) = line.extents();
            let line_height = logical.height() as f64 / pango::SCALE as f64;

            y += line_height;
            if y > page_height {
                result += 1;
                y = margin_top;
            }
        }
    }
    
    if y > margin_top { result + 1 } else { result }
}

fn page_number(cr: &Context, 
    page_no: usize,
    total_pages: usize,
    page_width: f64,
    page_height: f64,
) {
    // Draw page number centered at bottom
    let footer_layout = create_layout(cr);

    footer_layout.set_text(&format!("{} of {}", page_no, total_pages));
    footer_layout.set_font_description(Some(&FontDescription::from_string("Arial Bold 10")));

    // Measure width of the page number
    let (ink, _) = footer_layout.extents();
    let text_width = ink.width() as f64 / pango::SCALE as f64;

    let x = ((page_width - text_width) / 2.0) + A4_DEFAULT_MARGINS.left;
    let y = page_height - A4_DEFAULT_MARGINS.bottom;

    cr.move_to(x, y);
    show_layout(&cr, &footer_layout);
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

    let page_width: f64 = a4_default_content_width();

    let layout = create_layout(&cr);
    layout.set_text(&text);
    layout.set_width((page_width * pango::SCALE as f64) as i32);
    layout.set_justify(true);
    layout.set_wrap(WrapMode::WordChar);

    let desc = FontDescription::from_string(font_description);
    layout.set_font_description(Some(&desc));

    let total_pages = total_pages(&layout, 
        A4_DEFAULT_MARGINS.top, a4_default_content_height());

    let mut y: f64 = A4_DEFAULT_MARGINS.top;
    let mut page_no: usize = 1;

    for i in 0..layout.line_count() {
        if let Some(line) = layout.line(i) {
            let (_ink, logical) = line.extents();
            let line_height = logical.height() as f64 / pango::SCALE as f64;

            cr.move_to(A4_DEFAULT_MARGINS.left, y);
            pangocairo::functions::show_layout_line(&cr, &line);

            y += line_height;

            if y > a4_default_content_height() {
                page_number(&cr, page_no, total_pages, 
                    a4_default_content_width(), A4.height);

                let _ = cr.show_page(); // finish current page, start new one
                y = A4_DEFAULT_MARGINS.top;
                page_no += 1;
            }
        }
    }

    if y > 50.0 {
        page_number(&cr, page_no, total_pages, 
            a4_default_content_width(), A4.height);
    };

    surface.finish();

    println!("PDF written to: {pdf_file_name}");
}