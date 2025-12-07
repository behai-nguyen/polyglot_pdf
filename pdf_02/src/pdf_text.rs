// 28/11/2025

use std::fs;
use std::collections::HashSet;

use ttf_parser::Face;

use crate::hb::{
    hb_direction_t,
    hb_script_t,
};

use super::{
    FontInfo,
    get_font_info,
    get_font_subset,
    a4_default_content_width,
    text_to_lines,
};

/// Represents the document input pages.
pub struct PdfTextContent {
    /// Input. Text direction.
    hb_direction: hb_direction_t,
    /// Input. Text script.
    hb_script: hb_script_t,
    /// Input. Language. Assumption: Input text is of a single language only!
    language: String,
    /// Input. Font size. Assumption: PDF text is of only a single font size!
    font_size_pt: f32,
    /// Font program info, set using font name parameter.
    font_info: FontInfo, 
    /// Font subset for input text.
    font_subset: Vec<u8>,
    /// Unique character IDs identified from all pages text. Despite being 
    /// named CID, it is actually the Glyph ID, which is a font program identifier 
    /// corresponding to Unicode value that identifies a character from the text.
    used_cids: Vec<u16>,
    /// Paragraphs input text have been broken into individual lines which fit 
    /// a designated page width. The break algorithm is a simple greedy line 
    /// breaking based on words' boundaries. These lines are called shaped lines. 
    /// Each individual `Vec<u8>` in this vector represents the glyph bytes 
    /// of the shaped lines. Individual `Vec<u8>` are ready for PDF Tj text 
    /// operator. 
    lines_glyph_bytes: Vec<Vec<u8>>,
    /// ToUnicode CMap stream for the entire input text.
    copy_paste_unicodes: Vec<u16>,
}

impl PdfTextContent {
    pub fn new(hb_direction: hb_direction_t,
        hb_script: hb_script_t,
        language: &str,
        font_size_pt: f32,        
    ) -> Self {
        Self { 
            hb_direction, 
            hb_script, 
            language: language.to_string(),
            font_size_pt,
            font_info: Default::default(),
            font_subset: Default::default(),
            used_cids: Vec::<u16>::new(),
            lines_glyph_bytes: Vec::<Vec<u8>>::new(),
            copy_paste_unicodes: Vec::<u16>::new(),
        }
    }

    #[allow(dead_code)]
    pub fn hb_direction(&self) -> &hb_direction_t {
        &self.hb_direction
    }

    #[allow(dead_code)]
    pub fn hb_script(&self) -> &hb_script_t {
        &self.hb_script
    }

    #[allow(dead_code)]
    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn font_size_pt(&self) -> &f32 {
        &self.font_size_pt
    }

    pub fn font_info(&self) -> &FontInfo {
        &self.font_info
    }

    pub fn font_subset(&self) -> &Vec<u8> {
        &self.font_subset
    }

    pub fn used_cids(&self) -> &Vec<u16> {
        &self.used_cids
    }

    pub fn lines_glyph_bytes(&self) -> &Vec<Vec<u8>> {
        &self.lines_glyph_bytes
    }

    pub fn copy_paste_unicodes(&self) -> &Vec<u16> {
        &self.copy_paste_unicodes
    }

    /// Prepare `font_subset` for the entire input text.
    fn text_font_subset(&mut self, font_name: &str, text: &str) -> Result<(), String> {
        // Get the font program information.
        self.font_info = match get_font_info(font_name) {
            Some(fi) => fi,
            None => return Err(format!("Font: {} not found.", font_name)),
        };

        // Font subset for all text in PDF document
        self.font_subset = match get_font_subset(
            &self.font_info.path(), 
            self.font_info.face_index(), 
            &text
        ) {
            Ok(result) => result,
            Err(err) => return Err(err),
        };

        Ok(())
    }

    /// Prepare `used_cids` and `lines_glyph_bytes` for the input text, which have 
    /// been broken into shaped lines.
    fn text_used_cids_glyph_bytes(&mut self, shaped_lines: &Vec<String>, face: Face) {
        for line in shaped_lines {
            let mut glyphs_for_text: Vec<u16> = Vec::<u16>::new();

            for ch in line.chars() {
                if let Some(gid) = face.glyph_index(ch) {
                    let gid_u16 = gid.0;
                    glyphs_for_text.push(gid_u16);
                    if !self.used_cids.contains(&gid_u16) {
                        self.used_cids.push(gid_u16);
                    }
                } else {
                    // fallback to glyph 0 (missing glyph)
                    glyphs_for_text.push(0u16);
                    if !self.used_cids.contains(&0u16) {
                        self.used_cids.push(0u16);
                    }
                }
            }

            let mut glyph_bytes = Vec::<u8>::new();
            for gid in glyphs_for_text.iter() {
                glyph_bytes.extend_from_slice(&gid.to_be_bytes());
            }

            self.lines_glyph_bytes.push(glyph_bytes);
        };
    }

    /// Prepare `copy_paste_unicodes` for the entire input text.
    fn text_copy_paste_unicodes(&mut self, text: String) {
        let utf16: Vec<u16> = text.encode_utf16().collect();
        // Unique u16.
        self.copy_paste_unicodes = utf16.into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
    }

    pub fn prepare(&mut self, 
        input_text_file: &str, 
        input_font_name: &str
    ) -> Result<(), String> {
        // Read input text file.
        let text = match fs::read_to_string(input_text_file) {
            Ok(str) => str,
            Err(err) => return Err(err.to_string()),
        };

        // Prepare the font subset for the entire text.
        self.text_font_subset(input_font_name, &text)?;

        // Font metric.
        // Cloning to break free of self immutable borrow, so that it can be 
        // borrowed as mutable later.
        let font_subset = self.font_subset.clone();
        let face = Face::parse(&font_subset, 0).expect("TTF parse");

        // Parse with ttf-parser for glyph indices + metrics
        let units_per_em = face.units_per_em() as f32;
        let scale = self.font_size_pt / units_per_em;

        // Break each individual paragraph in the input text file into lines 
        // which fit `A4_57M.width`.
        let shaped_lines = text_to_lines(&text, 
            a4_default_content_width(), 
            self.hb_direction, self.hb_script, &self.language, 
            &self.font_subset, scale);

        // Prepare `used_cids` and `lines_glyph_bytes` for the input text, which 
        // have been broken into shaped lines.
        self.text_used_cids_glyph_bytes(&shaped_lines, face);

        // Prepare `copy_paste_unicodes` for the entire input text.
        self.text_copy_paste_unicodes(text);

        Ok(())

    }
    
}