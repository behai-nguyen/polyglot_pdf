/* 17/03/2026 */

//! Manually parse `![caption](relative/path/to/image.png)` to identify and return 
//! `caption` and `relative/path/to/image.png`.
//! 
//! Note, using regular expression `!\[(?P<caption>[^\]]*)\]\((?P<path>[^)]+)\)` 
//! is a simpler approach, but it introduces more external dependency.
//! 
//! # Assumptions
//! 
//! 1. The image block text must be at least `![](relative/path/to/image.png)`.
//! 
//! 2. The parser handles only one image per line: i.e., the image block text 
//!    represents a single image block.
//! 
//! # Limitations
//! 
//! 1. Captions containing ] is not supported, the following:
//! 
//!        `![A caption with \]](path)`
//! 
//!    will be treated as invalid.
//! 
//! 2. Paths containing ) is not supported, the following:
//! 
//!        `![caption](path_(1).png)`
//! 
//!    will result in invalid path: the parser will stop at the first ).
//! 
//! 3. Multiple image blocks on one line is not supported. The parser handles 
//!    only one image block per line.
//! 

const ERROR_TEXT: &str = "Invalid image block syntax";

#[derive(Debug)]
pub struct ImageBlockInfo<'a> {
    caption: &'a str,
    path: &'a str,
}

impl<'a> ImageBlockInfo<'a> {
    pub fn new_empty() -> Self {
        ImageBlockInfo { caption: "", path: "" }
    }

    pub fn caption(&self) -> &str {
        self.caption
    }

    pub fn path(&self) -> &str {
        self.path
    }

    pub fn set_caption(&mut self, caption: &'a str) {
        self.caption = caption;
    }

    pub fn set_path(&mut self, path: &'a str) {
        self.path = path;
    }

    pub fn is_valid(&self) -> bool {
        self.path.len() > 0
    }
}

struct ImageBlockParser<'a> {
    text: &'a str,
    len: usize,
    char_index: usize,
    start: usize,
    byte_index: usize,
}

impl<'a> ImageBlockParser<'a> {
    pub fn from_str(text: &'a str) -> Self {
        ImageBlockParser {
            text: text.trim(),
            len: text.chars().count(),
            char_index: 0,
            start: usize::MAX,
            byte_index: 0,
        }
    }

    fn is_at_end(&self) -> bool {
        self.char_index >= self.len
    }

    fn increase_indexes(&mut self, count: usize) {
        self.char_index += 1;
        self.byte_index += count;
    }

    fn advance(&mut self) -> Option<char> {
        if let Some((_, c)) = self.text.char_indices().nth(self.char_index) {
            self.increase_indexes(c.len_utf8());
            Some(c)
        } else {
            None
        }
    }

    fn slice(&self, end_offset: usize) -> &'a str {
        &self.text[self.start..self.byte_index - end_offset]
    }    

    pub fn parse(&mut self) -> Result<ImageBlockInfo<'a>, Box<dyn std::error::Error>> {
        let mut stack: Vec<char> = Vec::new();
        let mut img_block_info = ImageBlockInfo::new_empty();

        while !self.is_at_end() {
            let c = self.advance().unwrap_or('\0');

            match c {
                '[' => {
                    stack.push(c);                    
                    self.start = self.byte_index
                },
                ']' => {
                    let last_char = stack.pop();

                    if last_char.is_none() { return Err(ERROR_TEXT.into()) }
                    if last_char.unwrap() != '[' { return Err(ERROR_TEXT.into()) }

                    img_block_info.set_caption(self.slice(']'.len_utf8()));
                },
                '(' => {
                    // There should be nothing in the stack.
                    let last_char = stack.pop();
                    if last_char.is_some() { return Err(ERROR_TEXT.into()) }

                    stack.push(c);
                    self.start = self.byte_index
                },
                ')' => {
                    let last_char = stack.pop();

                    if last_char.is_none() { return Err(ERROR_TEXT.into()) }
                    if last_char.unwrap() != '(' { return Err(ERROR_TEXT.into()) }

                    img_block_info.set_path(self.slice(')'.len_utf8()));
                },

                _ => {}
            }
        }

        if img_block_info.is_valid() { Ok(img_block_info) } else { Err(ERROR_TEXT.into()) }        
    }
}

/// Manually parse `![caption](relative/path/to/image.png)` to identify and return 
/// `caption` and `relative/path/to/image.png`.
pub fn parse_image_block<'a>(line: &'a str) -> Result<ImageBlockInfo<'a>, Box<dyn std::error::Error>> {
    let mut parser = ImageBlockParser::from_str(line);
    parser.parse()    
}

// To run test for this module only: 
// 
//     * cargo test image_block_parser::tests
//
//     * cargo test image_block_parser::tests::test_valid -- --exact [--nocapture]
//     * cargo test image_block_parser::tests::test_invalid -- --exact [--nocapture]
#[cfg(test)]
mod tests {
    use super::*;

    // Valid
    const VALID_01: &str = "![](relative/path/to/image.png)";
    const VALID_CAPTION_01: &str = "";
    const VALID_PATH_01: &str = "relative/path/to/image.png";

    const VALID_02: &str = "![Đây Là Chú Thích Của Hình](relative/path/to/image.png)";
    const VALID_CAPTION_02: &str = "Đây Là Chú Thích Của Hình";
    const VALID_PATH_02: &str = "relative/path/to/image.png";

    const VALID_03: &str = "![Đây Là Con Chim Cassowary](./img/139015.png)";
    const VALID_CAPTION_03: &str = "Đây Là Con Chim Cassowary";
    const VALID_PATH_03: &str = "./img/139015.png";

    const VALID_04: &str = "![Đây Là Con Chim Cassowary]\
        (F:/rust/polyglot_pdf/pdf_08_image_layout/img/139015.png)";
    const VALID_CAPTION_04: &str = "Đây Là Con Chim Cassowary";
    const VALID_PATH_04: &str = "F:/rust/polyglot_pdf/pdf_08_image_layout/img/139015.png";

    const VALID_MULTILINE_CAPTION: &str = "![Cassowary, an Australia native, and \
        “the world's most dangerous bird”.\nCassowary, chim bản địa Úc, và là \
        “loài chim nguy hiểm nhất thế giới”.](./img/139015.png)";
    const VALID_MULTILINE_CAPTION_CAPTION: &str = "Cassowary, an Australia native, and \
        “the world's most dangerous bird”.\nCassowary, chim bản địa Úc, và là “loài chim \
        nguy hiểm nhất thế giới”.";
    const VALID_MULTILINE_CAPTION_PATH: &str = "./img/139015.png";

    const VALID_MULTI_LINGUAL_CAPTION: &str = "![Mount Fuji / 富士山, ふじさ, Fujisan / \
        Núi Phú Sỹ](./img/fujisan.png)";
    const VALID_MULTI_LINGUAL_CAPTION_CAPTION: &str = "Mount Fuji / 富士山, ふじさ, Fujisan / \
        Núi Phú Sỹ";
    const VALID_MULTI_LINGUAL_PATH: &str = "./img/fujisan.png";

    // Invalid
    const INVALID_01: &str = "!Đây Là Chú Thích Của Hình](relative/path/to/image.png)";
    const INVALID_02: &str = "![Đây Là Chú Thích Của Hình(relative/path/to/image.png)";
    const INVALID_03: &str = "![Đây Là Chú Thích Của Hình]relative/path/to/image.png)";
    const INVALID_04: &str = "![Đây Là Chú Thích Của Hình](relative/path/to/image.png";
    const INVALID_05: &str = "![Đây Là Chú Thích Của Hình]()";

    struct ValidTest<'a> {
        text: &'a str,
        caption: &'a str,
        path: &'a str,
    }

    #[test]
    fn test_valid() {
        let test_data: Vec<ValidTest> = vec![
            ValidTest { text: VALID_01, caption: VALID_CAPTION_01, path: VALID_PATH_01 },
            ValidTest { text: VALID_02, caption: VALID_CAPTION_02, path: VALID_PATH_02 },
            ValidTest { text: VALID_03, caption: VALID_CAPTION_03, path: VALID_PATH_03 },
            ValidTest { text: VALID_04, caption: VALID_CAPTION_04, path: VALID_PATH_04 },
            ValidTest { text: VALID_MULTILINE_CAPTION, 
                caption: VALID_MULTILINE_CAPTION_CAPTION, path: VALID_MULTILINE_CAPTION_PATH },
            ValidTest { text: VALID_MULTI_LINGUAL_CAPTION, 
                caption: VALID_MULTI_LINGUAL_CAPTION_CAPTION, 
                path: VALID_MULTI_LINGUAL_PATH },                
        ];

        for (index, data) in test_data.iter().enumerate() {
            let res = parse_image_block(data.text);

            assert!(res.is_ok(), "{}", format!("Image block text {} result", index));
            let img_block_info = res.unwrap();
            assert_eq!(img_block_info.caption(), data.caption, "{}", 
                format!("Image block text {} caption", index));
            assert_eq!(img_block_info.path(), data.path, "{}", 
                format!("Image block text {} path", index));
        }
    }

    #[test]
    fn test_invalid() {
        let test_data: Vec<&str> = vec![
            INVALID_01, 
            INVALID_02, 
            INVALID_03, 
            INVALID_04,
            INVALID_05,
        ];

        for (index, text) in test_data.iter().enumerate() {
            let res = parse_image_block(text);

            let err = res
                .expect_err(
                    &format!("Expected failure for image block text {}", index)
                );

            assert_eq!(err.to_string(), ERROR_TEXT, "{}", 
                format!("Invalid image block text {}", index));
        }
    }
}