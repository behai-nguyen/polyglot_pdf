/* Date Created: 02/03/2026 */

//! Scale PNG image by the width to fit across the page.
//! 1. Render the scaled image onto a PDF:
//!     a. A4.width, A4.height
//!     b. context.translate() before context.scale().
//!     c. context.save() and context.restore().
//! 
//! 2. Write some text just below the image.
//! 
//! 3. Render another scaled image onto the PDF followed by some text.
//! 
//! 4. All on a single page.

use cairo::{Context, PdfSurface};
use pango::{Layout, FontDescription};
use pangocairo::functions::{create_layout, show_layout};

mod page_geometry;
use page_geometry::{
    A4,
    a4_default_content_width,
    A4_DEFAULT_MARGINS,
};

mod image_layout;
use image_layout::render_image_block;

fn layout_ink_metrics(layout: &Layout) -> (f64, f64) {
    let (ink_rect, _) = layout.extents();
    let scale = pango::SCALE as f64;

    let y_bearing: f64 = ink_rect.y() as f64 / scale;
    let height: f64 = ink_rect.height() as f64 / scale;

    (y_bearing, height)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (font_description, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Arial Unicode MS", "win_output_08.pdf")
    } else {
        ("NotoSansTC-Regular", "ubuntu_output_08.pdf")
    };

    let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)?;

    let context = Context::new(&surface)?;

    // Define the input PNG image file name (ensure this file exists).
    let png_file_name = "./img/139015.png";
    let image_bottom = render_image_block(png_file_name, 
        0.75, true, A4_DEFAULT_MARGINS.top, &context)?;

    // Add some text.
    let layout = create_layout(&context);
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    let desc = FontDescription::from_string(font_description);
    layout.set_font_description(Some(&desc));    

    // Text appears below the image: there should be a natural vertical space 
    // between the image and the text line.
    let baseline_y = image_bottom;

    // Current y coordinate.
    let mut y: f64 = baseline_y;

    layout.set_text("Cassowary, an Australia native, and “the world's most dangerous bird”.");
    context.move_to(A4_DEFAULT_MARGINS.left, y);
    show_layout(&context, &layout);

    // The text block height of the previous text: "Cassowary, an Australia native, \
    //     and “the world's most dangerous bird”."
    let (_, height) = layout_ink_metrics(&layout);
    y += height;

    layout.set_text("Cassowary, chim bản địa Úc, và là “loài chim nguy hiểm nhất thế giới”.");
    context.move_to(A4_DEFAULT_MARGINS.left, y);
    show_layout(&context, &layout);

    let (_, height) = layout_ink_metrics(&layout);
    y += height + (height / 2.0);

    // Define the input PNG image file name (ensure this file exists).
    let png_file_name = "./img/KTmCgCBjQXKLsO2JeBMVrA.png";
    let image_bottom = render_image_block(png_file_name, 
        0.75, false, y, &context)?;

    layout.set_text("Can Roadrunners outrun Cassowaries?");
    context.move_to(A4_DEFAULT_MARGINS.left, image_bottom);
    show_layout(&context, &layout);        

    context.show_page()?;

    // Finish the surface to ensure all data is written to the file stream.
    surface.finish();

    println!("Successfully generated PDF: {}", pdf_file_name);

    Ok(())
}