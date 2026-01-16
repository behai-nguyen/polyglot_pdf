// 15/01/2026
// 
// Rotate text progressively by 360 degrees, stepping 10 degrees at a time.
//

// use std::f64::consts::PI;
use cairo::{PdfSurface, Context};
use pango::FontDescription;
use pangocairo::functions::{create_layout, show_layout};

mod page_geometry;
use page_geometry::{
    a4_default_content_width,
    a4_default_content_height,
    A4_DEFAULT_MARGINS,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (font_description, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Arial Unicode MS 12", "win_output.pdf")
    } else {
        ("NotoSansTC-Regular 12", "ubuntu_output.pdf")
    };
    
    let surface = PdfSurface::new(a4_default_content_width(), 
        a4_default_content_height(), pdf_file_name)?;

    let context = Context::new(&surface)?;

    // Add some text text
    let layout = create_layout(&context);
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    let desc = FontDescription::from_string(font_description);
    layout.set_font_description(Some(&desc));

    layout.set_text("Kỷ độ Long Tuyền đới nguyệt ma");

    context.move_to(A4_DEFAULT_MARGINS.left, A4_DEFAULT_MARGINS.top);
    show_layout(&context, &layout);

    // Save the current state
    context.save()?;

    context.move_to(a4_default_content_width() / 2.0, 
        a4_default_content_height() / 2.0);

    for degree in (10..370).step_by(10) {
        context.rotate((degree as f64).to_radians());
        show_layout(&context, &layout);
        context.set_matrix(cairo::Matrix::identity());
    }

    /* 
    // This loop is equivalent to the loop above. 
    // If using this loop, remove the outer pair of `context.save()?;` 
    // and `context.restore()?;`
    for degree in (10..370).step_by(10) {
        context.save()?;
        context.rotate((degree as f64).to_radians());
        show_layout(&context, &layout);
        context.restore()?;
    }
    */

    // Restore the context to the original matrix state for subsequent drawing operations
    context.restore()?;

    // Finish the page and surface
    context.show_page()?;
    surface.finish();

    println!("PDF generated successfully as {pdf_file_name}");

    Ok(())
}