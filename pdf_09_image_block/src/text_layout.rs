/* 09/03/2026 */

use cairo::glib::translate::ToGlibPtr;
use pango_sys::pango_layout_set_justify;
use cairo::Context;
use pango::{Layout, WrapMode};
use pangocairo::functions::create_layout;

use crate::page_geometry::a4_default_content_width;
use crate::config::FontSpec;

pub trait LayoutExtJustify {
    fn set_justify(&self, justify: bool);
}

impl LayoutExtJustify for Layout {
    /// Fully justified a block of text: left and right justified.
    fn set_justify(&self, justify: bool) {
        unsafe {
            pango_layout_set_justify(self.to_glib_none().0, justify as i32);
        }
    }
}

/// A4 default content width in [`pango::SCALE`].
pub fn a4_layout_width() -> i32 {
    (a4_default_content_width() * pango::SCALE as f64) as i32
}

/// Create a [`pango::Layout`] text layout for.
/// 
/// # Arguments
/// 
/// * `fully_justified` — left and right justified the text block.
/// 
pub fn create_text_layout(layout_width: i32,
    text: &str, 
    text_font: &FontSpec, 
    fully_justified: bool,
    context: &Context) -> Layout {
    let layout: Layout = create_layout(context);

    // Set width, wrap, justify
    layout.set_width(layout_width);
    layout.set_wrap(WrapMode::WordChar);
    if fully_justified { layout.set_justify(true) };
    layout.set_font_description(Some(&text_font.to_pango_description()));
    layout.set_text(text);
    
    layout
}

/// Work out and return a fully prepared [`pango::Layout`] height for 
/// a block of text.
/// 
pub fn layout_block_height(layout: &Layout) -> f64 {
    let (_, height) = layout.extents();

    height.height() as f64 / pango::SCALE as f64
}

/// Centre a [`pango::Layout`] block. Note, the text block is either 
/// left and right justified, or only left-justified.
/// 
/// Centre the text block based on the longest width in the block.
/// 
pub fn center_layout_block(context: &Context, layout: &Layout, page_width: f64) {
    let mut max_width = 0.0;
    let scale = pango::SCALE as f64;

    for i in 0..layout.line_count() {
        let line = layout.line(i).unwrap();
        let (ink_rect, _) = line.extents();
        let w = ink_rect.width() as f64 / scale;
        if w > max_width {
            max_width = w;
        }
    }

    // Compute horizontal offset to center the block.
    let offset_x = (page_width - max_width) / 2.0;

    context.rel_move_to(offset_x, 0.0);
}
