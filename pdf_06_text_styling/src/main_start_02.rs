/* 16/02/2026 */

//! Apply `Span`s, `parse_inline()` and `reserve_asterisk()`, to style inline text.

use cairo::{PdfSurface, Context};
use pango::{FontDescription, Attribute, AttrInt, Weight, Style/*, Underline, AttrColor*/};
use pangocairo::functions::{create_layout, show_layout};

mod page_geometry;
use page_geometry::{
    a4_default_content_width,
    a4_default_content_height,
    A4_DEFAULT_MARGINS,
};

mod document;
use document::{Span, SpanStyle};

mod inline_parser;
use inline_parser::{parse_inline, reserve_asterisk};

fn create_font_attrs(span: &Span) -> Vec<Attribute> {
    let mut attrs: Vec<Attribute> = Vec::new();

    match *span.style() {
        SpanStyle::Normal => {}
        SpanStyle::Bold => {
            let mut bold = AttrInt::new_weight(Weight::Bold); 
            bold.set_start_index(span.start() as u32); 
            bold.set_end_index(span.end() as u32); 
            attrs.push(bold.into());

            /* 
            // Valid code.            
            let mut colour = AttrColor::new_foreground(0, 65535, 0); 
            colour.set_start_index(span.start() as u32); 
            colour.set_end_index(span.end() as u32); 
            attrs.push(colour.into());            

            let underline = AttrInt::new_underline(Underline::Single);
            attrs.push(underline.into());
            */
        }
        SpanStyle::Italic => {
            let mut italic = AttrInt::new_style(Style::Italic);
            italic.set_start_index(span.start() as u32);
            italic.set_end_index(span.end() as u32);
            attrs.push(italic.into());

            /* 
            // Valid code.
            let mut colour = AttrColor::new_foreground(65535, 0, 0); 
            colour.set_start_index(span.start() as u32); 
            colour.set_end_index(span.end() as u32); 
            attrs.push(colour.into());            

            let underline = AttrInt::new_underline(Underline::Single);
            attrs.push(underline.into());
            */
        }        
    }

    attrs
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (font_description, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Be Vietnam Pro 10", "win_output_02.pdf")
    } else {
        ("Be Vietnam Pro 10", "ubuntu_output_02.pdf")
    };

    let surface = PdfSurface::new(a4_default_content_width(), 
        a4_default_content_height(), pdf_file_name)?;

    let context = Context::new(&surface)?;

    // Add some text text
    let layout = create_layout(&context);
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    let desc = FontDescription::from_string(font_description);
    layout.set_font_description(Some(&desc));

    let markdown_text = r"**Không đọc *sử* không đủ tư cách nói chuyện *chính trị*.** \*";
    // let markdown_text = "***Không đọc sử không đủ tư cách nói chuyện chính trị.***";
    // let markdown_text = "( **Chính Ðạo, *Việt Nam Niên Biểu*, *Tập 1A***, trang 347 )";

    let res = parse_inline(markdown_text);

    let attrs = pango::AttrList::new();
    for span in res.spans() {
        for attr in create_font_attrs(span) {
            attrs.insert(attr);
        }
    }
    layout.set_attributes(Some(&attrs));
    layout.set_text(&reserve_asterisk(res.text()));

    context.move_to(A4_DEFAULT_MARGINS.left, A4_DEFAULT_MARGINS.top);

    show_layout(&context, &layout);

    // Finish the page and surface
    context.show_page()?;
    surface.finish();

    println!("PDF generated successfully as {pdf_file_name}");

    Ok(())    
}