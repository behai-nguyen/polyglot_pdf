// 09/11/2025

use ttf_parser::Face;

/// Represents an input page.
pub struct PdfPage {
    /// Text content of the page.
    text: String,
    /// Page (text) content: we need to emit the glyph IDs (CIDs) as big-endian u16 bytes.
    glyph_bytes: Vec<u8>,
}

/// Represents the document input pages.
pub struct PdfPages {
    /// All input pages.
    pages: Vec<PdfPage>,
    /// Unique character IDs identified from all pages text. Despite being 
    /// named CID, it is actually the Glyph ID, which is a font program identifier 
    /// corresponding to Unicode value that identifies a character from the text.
    used_cids: Vec<u16>,
}

impl PdfPage {
    pub fn new(page_text: &str) -> Self {
        Self {
            text: page_text.to_string(),            
            glyph_bytes: Vec::<u8>::new(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn glyph_bytes(&self) -> &Vec<u8> {
        &self.glyph_bytes
    }
}

impl PdfPages {
    pub fn new() -> Self {
        Self {
            pages: Vec::<PdfPage>::new(),
            used_cids: Vec::<u16>::new(),
        }
    }

    pub fn add_page(mut self, page_text: &str) -> Self {
        self.pages.push( PdfPage::new(page_text) );
        self
    }

    pub fn pages(&self) -> &Vec<PdfPage> {
        &self.pages
    }

    pub fn all_text(&self) -> String {
        self.pages.iter().map(|p| p.text()).collect::<Vec<_>>().join(" ")
    }

    pub fn used_cids(&self) -> &Vec<u16> {
        &self.used_cids
    }

    /// Iterating through all input pages. For each page, works out the `glyph_bytes` 
    /// from the input `text`; and also works out the `used_cids` as going through 
    /// each page, and each charater in each page input `text`.
    pub fn prepare_used_cids_glyph_bytes(&mut self, face: &Face) {
        for page in &mut self.pages {
            let mut glyphs_for_text: Vec<u16> = Vec::<u16>::new();

            for ch in page.text.chars() {
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

            for gid in glyphs_for_text.iter() {
                page.glyph_bytes.extend_from_slice(&gid.to_be_bytes());
            }

        }
        // Sort used cids
        self.used_cids.sort_unstable();        
    }
}