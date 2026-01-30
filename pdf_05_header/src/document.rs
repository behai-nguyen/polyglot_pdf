/// 25/01/2026
/// 
/// Types describe the structure of a document.
///

/// Semantic document structure.
#[derive(Debug, Clone)]
pub enum Block {
    /// The text block / line is a header.
    Header { level: u8, text: String },
    /// The text block / paragraph / blank line is a normal text.
    Paragraph { text: String },
}

/// Physical layout fragments.
#[derive(Debug)]
pub struct PositionedBlock {
    /// Index to the original semantic Block.
    block_index: usize,
    /// Which page this fragment belongs to.
    page: usize,
    /// Where on the page it starts.
    y_offset: f64,
    /// Height of lines. Assumption: the block share the same font.
    /// Later on, when supporting bold, italic, etc., it will need refactoring.
    line_height: f64,
    /// First line of this fragment (for paragraphs).
    line_start: usize,
    /// Last line of this fragment (exclusive)
    line_end: usize,
}

pub const MAX_HEADER_LEVEL: usize = 6;

impl PositionedBlock {
    pub fn new(block_index: usize, 
        page: usize,
        y_offset: f64,
        line_height: f64,
        line_start: usize,
        line_end: usize,
    ) -> Self {
        PositionedBlock {
            block_index,
            page,
            y_offset,
            line_height, 
            line_start, 
            line_end
        }
    }

    /// Index to the original semantic block.
    pub fn block_index(&self) -> usize {
        self.block_index
    }

    /// Which page this fragment belongs to.
    pub fn page(&self) -> usize {
        self.page
    }

    /// Where on the page it starts.
    pub fn y_offset(&self) -> f64 {
        self.y_offset
    }

    /// Height of lines. Assumption: the block share the same font.
    /// Later on, when supporting bold, italic, etc., it will need refactoring.
    pub fn line_height(&self) -> f64 {
        self.line_height
    }

    /// First line of this fragment (for paragraphs).
    pub fn line_start(&self) -> usize {
        self.line_start
    }

    /// Last line of this fragment (exclusive)
    pub fn line_end(&self) -> usize {
        self.line_end
    }
}
