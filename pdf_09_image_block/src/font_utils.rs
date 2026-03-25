/* 19/02/2026 */

//! A bridge between font configuration and Pango.

use pango::{FontDescription, Attribute, AttrInt, Weight, Style};

use crate::config::FontSpec;
use crate::document::{Span, SpanStyle};

impl FontSpec {
    pub fn to_pango_description(&self) -> FontDescription {
        let mut desc = FontDescription::new();

        desc.set_family(self.family());
        desc.set_size(self.size() * pango::SCALE);

        // Only takes effect if the font supports it.
        match self.weight() {
            "thin" => desc.set_weight(pango::Weight::Thin), 
            "light" => desc.set_weight(pango::Weight::Light), 
            "medium" => desc.set_weight(pango::Weight::Medium),            
            "bold" => desc.set_weight(pango::Weight::Bold),
            "normal" => desc.set_weight(pango::Weight::Normal),
            _ => desc.set_weight(pango::Weight::Normal),
        }

        // Only takes effect if the font supports it.
        match self.style() {
            "italic" => desc.set_style(pango::Style::Italic),
            "normal" => desc.set_style(pango::Style::Normal),
            _ => desc.set_style(pango::Style::Normal),
        }

        desc
    }
}

pub fn create_font_attrs(span: &Span) -> Vec<Attribute> {
    let mut attrs: Vec<Attribute> = Vec::new();

    match *span.style() {
        SpanStyle::Normal => {}
        SpanStyle::Bold => {
            let mut bold = AttrInt::new_weight(Weight::Bold); 
            bold.set_start_index(span.start() as u32); 
            bold.set_end_index(span.end() as u32); 
            attrs.push(bold.into());
        }
        SpanStyle::Italic => {
            let mut italic = AttrInt::new_style(Style::Italic);
            italic.set_start_index(span.start() as u32);
            italic.set_end_index(span.end() as u32);
            attrs.push(italic.into());
        }        
    }

    attrs
}