/* 16/02/2026 */

//! Manually apply weight and style to text.

use cairo::{PdfSurface, Context};
use pango::{FontDescription, AttrInt, Weight, Style};
use pangocairo::functions::{create_layout, show_layout};

mod page_geometry;
use page_geometry::{
    a4_default_content_width,
    a4_default_content_height,
    A4_DEFAULT_MARGINS,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (font_description, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Be Vietnam Pro 12", "win_output_01.pdf")
    } else {
        ("Be Vietnam Pro 12", "ubuntu_output_01.pdf")
    };

    let surface = PdfSurface::new(a4_default_content_width(), 
        a4_default_content_height(), pdf_file_name)?;

    let context = Context::new(&surface)?;

    // Add some text text
    let layout = create_layout(&context);
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    let desc = FontDescription::from_string(font_description);
    layout.set_font_description(Some(&desc));

    let attrs = pango::AttrList::new();
    
    let mut bold = AttrInt::new_weight(Weight::Bold); 
    bold.set_start_index(0); 
    bold.set_end_index(9); 
    attrs.insert(bold);

    let mut italic = AttrInt::new_style(Style::Italic);
    italic.set_start_index(4);
    italic.set_end_index(5);
    attrs.insert(italic);

    let mut italic = AttrInt::new_style(Style::Italic);
    italic.set_start_index(8);
    italic.set_end_index(9);
    attrs.insert(italic);

    layout.set_attributes(Some(&attrs));
    
    layout.set_text("xy, bc, de");

    context.move_to(A4_DEFAULT_MARGINS.left, A4_DEFAULT_MARGINS.top);

    show_layout(&context, &layout);

    // Finish the page and surface
    context.show_page()?;
    surface.finish();

    println!("PDF generated successfully as {pdf_file_name}");

    Ok(())    
}