/* 04/02/2026 */

//! Parses a markdown paragraph for `**bold**`, `*italic*` and `***bold italic***`.
//! 
//! The only markdown indicator supported is `*`.
//! 
//! Also supports escaped: `\*` and `\\`, which resulted in `\u{E000}` and `\`, 
//! respectively.
//! 
//! Markdowns can be adjacent: one after the others; or nested: markdowns within 
//! a markdown.
//! 
//! Adjacent: `— *italic*, **bold**, and ***bold italic text***.`
//! Nested: `**Không đọc *sử* không đủ tư cách nói chuyện *chính trị*.**`
//! 
//! Note: `***bold italic text***` produces a bold spans over an italic.
//! 
//! This module exports the following:
//! 
//!    pub struct InlineParseResult
//!    pub fn parse_inline(markdown_text: &str) -> InlineParseResult
//!    pub fn reserve_asterisk(text: &str) -> String
//! 
//! Please the `tests` mod below how they are used.

use crate::document::Span;

/// Private Use Area. 3-byte.
const ESCAPED_ASTERISK: char = '\u{E000}'; 

/// Markdown text with escape removed:
/// `***bold*** \*` → `***bold*** ` (***bold*** ESCAPED_ASTERISK)
/// `**bold \\Úc Đại Lợi\\**"` → bold span over `bold \Úc Đại Lợi\`.
/// 
/// Escaped asterisks are propagated down to `Pango` layout as ESCAPED_ASTERISK; 
/// call reserve_asterisk() on `clean_text` to reverse ESCAPED_ASTERISKs back 
/// to asterisks.
#[allow(dead_code)]
#[derive(Debug)]
struct EscapeResult {
    /// `***bold*** `
    /// `bold \Úc Đại Lợi\`
    clean_text: String,
    /// Mapping: original byte index → clean byte index.
    /// See `preprocess_escapes()` documentation for detail.
    original_to_clean: Vec<usize>,
}

/// An opening or closing marker.
/// Recognised marker indicator: *.
/// Recognised markers: *, **, and ***.
/// Assumption: all marker indicators are single-byte.
#[derive(Debug, Clone)]
struct Marker {
    /// How many marker indicators? Note, it will only be `2` or `1`:
    /// Valid `***`, such as `***x***` will be split into `2` and `1`, 
    /// which still results in bold italic.
    count: usize,
    /// At what byte position does this marker starts. It is the 
    /// position of the first indicator.
    start_byte: usize,
}

impl Marker {
    fn new(count: usize, start_byte: usize) -> Self {
        Self { count, start_byte }
    }

    fn set_start_byte(&mut self, new_start_byte: usize) {
        self.start_byte = new_start_byte;
    }
}

#[derive(Debug)]
struct MarkerEvent {
    opening: Marker,
	closing: Marker,
}

impl MarkerEvent {
    fn new(opening: Marker, closing: Marker) -> Self {
        Self { opening, closing }
    }
}

/// An array of `MarkerEvent`.
/// Instance of this array should contain always an even number of
/// `MarkerEvent`s. That is, `MarkerEvent`s always go in pair: opening 
/// and closing.
type MarkerEvents = Vec<MarkerEvent>;

struct MarkerResult {
    escape: EscapeResult,
    marker_events: MarkerEvents,
}

impl MarkerResult {
    fn new(escape: EscapeResult, marker_events: MarkerEvents) -> Self {
        Self { escape, marker_events }
    }
}

/// The final info returned and passed to `Pango`.
/// Fields are `pub` to enable destructuring.
#[allow(dead_code)]
pub struct InlineParseResult {
    /// Clean text with all markers removed. 
    /// Escaped asterisks are represented as ESCAPED_ASTERISK; call reserve_asterisk() 
    /// on `text` to reverse ESCAPED_ASTERISKs back to asterisks.
    pub text: String,
    /// Slice indexes to `text` for different text style.
    /// Text slices can contain ESCAPED_ASTERISKs, safe to always call reserve_asterisk() 
    /// on the text slices to reverse back to asterisks.
    pub spans: Vec<Span>,
}

#[allow(dead_code)]
impl InlineParseResult {
    pub fn new(text: String, spans: Vec<Span>) -> Self {
        Self { text, spans }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }
}

#[allow(dead_code)]
pub fn reserve_asterisk(text: &str) -> String {
    text.replace(ESCAPED_ASTERISK, "*").to_owned()
}

/// Consider `text` = `\*w\*!`, length is 6 bytes. When finished:
/// `clean`: `\u{E000}w\u{E000}!` / `w!` 8 bytes. `\u{E000}` is 3 bytes.
/// `map`: 
///     map[0] = 0
///     map[1] = 0
///     map[2] = 3
///     map[3] = 4
///     map[4] = 4
///     map[5] = 7
///     map[6] = 8
/// 
/// For each byte index in the original text, `map[i]` is the byte index in 
/// the cleaned text where that original byte ended up. It is a **many‑to‑one** 
/// mapping, because escaped sequences collapse multiple original bytes into 
/// fewer cleaned bytes.
/// 
/// `map[i]` = the byte index in the cleaned text where the original byte at 
/// index `i` ended up.
/// 
/// The original text:
///     Index: 0 1 2 3 4 5
///     Bytes: \ * w \ * !
/// 
/// The value of `clean `:
///     Index: 0 3     4 7       8
///     Char : \u{E000}w\u{E000} !
/// 
/// And so:
///     map[0] = 0   // original '\' maps to cleaned byte 0 (start of placeholder)
///     map[1] = 0   // original '*' maps to cleaned byte 0 (same placeholder)
/// 
/// - Original bytes 0 and 1 (`\*`) collapse into **one** placeholder character 
///   at cleaned byte 0.
/// - So both map to the same cleaned position.
/// 
///     map[2] = 3   // original 'w' maps to cleaned byte 3
///     map[3] = 4   // original '\' maps to cleaned byte 4
///     map[4] = 4   // original '*' maps to cleaned byte 4
///     map[5] = 7   // original '!' maps to cleaned byte 7
///     map[6] = 8   // end-of-string maps to cleaned length 8
/// 
/// - Cleaned text length is 8.
fn preprocess_escapes(text: &str) -> EscapeResult {
    let mut clean = String::new();
    let mut map = vec![0; text.len() + 1];

    let mut chars = text.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            if let Some(&(j, next_c)) = chars.peek() {
                // Escape sequence: \X → literal X.
                let clean_pos = clean.len();
                clean.push(if next_c == '*' {ESCAPED_ASTERISK} else {next_c});

                // All bytes from '\' up to and including escaped char map to this clean_pos.
                for k in i..(j + next_c.len_utf8()) {
                    map[k] = clean_pos;
                }

                // Consume the escaped char.
                chars.next();
                continue;
            }
        }

        // Normal char: copy as-is.
        let clean_pos = clean.len();
        clean.push(c);

        for k in i..(i + c.len_utf8()) {
            map[k] = clean_pos;
        }
    }

    map[text.len()] = clean.len();

    EscapeResult {
        clean_text: clean,
        original_to_clean: map,
    }
}

/// Identify opening and closing marker pairs.
/// Support both adjacent markers and nested-markers.
/// 
/// Malformed unparseable markers are treated as literal strings.
/// 
/// On `j`, consider this markdown: `a, **xy, *bc*, *de***, w`, it should produce:
/// 
///     MarkerEvent { opening: Marker { count: 2, start_byte: 3 }, 
///         closing: Marker { count: 2, start_byte: 19 } }
///     MarkerEvent { opening: Marker { count: 1, start_byte: 9 }, 
///         closing: Marker { count: 1, start_byte: 12 } }
///     MarkerEvent { opening: Marker { count: 1, start_byte: 15 }, 
///         closing: Marker { count: 1, start_byte: 18 } }
/// 
/// `j` tracks the `19` and `18` bytes correctly.
/// 
/// Important: the returned vector is sorted on `MarkerEvent.opening.start_byte`.
fn get_marker_events(text: &str) -> MarkerResult {
    let mut events: MarkerEvents = Vec::new();

    let mut i = 0;
    let esc = preprocess_escapes(text);
    let bytes = esc.clean_text.as_bytes();

    let mut stack: Vec<Marker> = Vec::new();

    while i < bytes.len() {
        if bytes[i] != b'*' {
            i += 1;
            continue;
        }

        // Count the run of '*'.
        let mut count = 1;
        while i + count < bytes.len() && bytes[i + count] == b'*' {
            count += 1;
        }

        // If there is a matching opener with this exact count, don't split.
        // Treat the whole run as a single marker.
        if stack.iter().any(|m| m.count == count) {
            if let Some(pos) = stack.iter().rposition(|m| m.count == count) {
                let opening = stack.remove(pos);
                events.push(MarkerEvent::new(opening, Marker::new(count, i)));
            } else {
                stack.push(Marker::new(count, i));
            }
            i += count;
            continue;
        }

        let mut remaining = count;
        let mut j = i; // Local cursor inside this run.

        while remaining > 0 {
            // Prefer to match the top of the stack if possible.
            let preferred = stack.last().map(|m| m.count);

            let this_count = if let Some(pref) = preferred {
                if pref <= remaining { pref } else if remaining >= 2 { 2 } else { 1 }
            } else {
                if remaining >= 2 { 2 } else { 1 }
            };

            if let Some(pos) = stack.iter().rposition(|m| m.count == this_count) {
                let opening = stack.remove(pos);
                events.push(MarkerEvent::new(opening, Marker::new(this_count, j)));
            } else {
                stack.push(Marker::new(this_count, j));
            }

            j += this_count;
            remaining -= this_count;
        }

        i += count;
    }

    events.sort_by_key(|ev| ev.opening.start_byte);
    MarkerResult::new(esc, events)
}

/// Consider the nested markdown text: `**Không đọc *sử* không đủ tư cách nói chuyện 
/// *chính trị*.**`, whose length is 77 bytes.
/// 
/// which produces the following `MarkerEvent`s:
/// 
///     verify_marker(&events[0], 2, 0, 75);
///     verify_marker(&events[1], 1, 16, 21);
///     verify_marker(&events[2], 1, 60, 73);
/// 
/// For `verify_marker()` see the tests mod.
/// 
/// The above `MarkerEvent` vector produces and returns `is_marker` and `mapping`: 
/// 
/// is_marker[0] = true         mapping[0] = 0   <- out = 0
/// is_marker[1] = true         mapping[1] = 0   <- out = 0
/// is_marker[2] = false        mapping[2] = 0   <- out = 0, -> out = 1
/// is_marker[3] = false        mapping[3] = 1   <- out = 1, -> out = 2
/// is_marker[4] = false        mapping[4] = 2   <- out = 2, -> out = 3
/// is_marker[5] = false        mapping[5] = 3   <- out = 3, -> out = 4
/// is_marker[6] = false        mapping[6] = 4   <- out = 4, -> out = 5
/// is_marker[7] = false        mapping[7] = 5   <- out = 5, -> out = 6
/// is_marker[8] = false        mapping[8] = 6   <- out = 6, -> out = 7
/// is_marker[9] = false        mapping[9] = 7   <- out = 7, -> out = 8
/// is_marker[10] = false       mapping[10] = 8  <- out = 8, -> out = 9
/// is_marker[11] = false       mapping[11] = 9  <- out = 9, -> out = 10
/// is_marker[12] = false       mapping[12] = 10 <- out = 10, -> out = 11
/// is_marker[13] = false       mapping[13] = 11 <- out = 11, -> out = 12
/// is_marker[14] = false       mapping[14] = 12 <- out = 12, -> out = 13
/// is_marker[15] = false       mapping[15] = 13 <- out = 13, -> out = 14
/// is_marker[16] = true        mapping[16] = 14 <- out = 14, -> out = 14
/// is_marker[17] = false       mapping[17] = 14 <- out = 14, -> out = 15
/// is_marker[18] = false       mapping[18] = 15 <- out = 15, -> out = 16
/// is_marker[19] = false       mapping[19] = 16 <- out = 16, -> out = 17
/// is_marker[20] = false       mapping[20] = 17 <- out = 17, -> out = 18
/// is_marker[21] = true        mapping[21] = 18 <- out = 18, -> out = 18
/// is_marker[22] = false       mapping[22] = 18 <- out = 18, -> out = 19
/// is_marker[23] = false       mapping[23] = 19 <- out = 19, -> out = 20
/// is_marker[24] = false       mapping[24] = 20 <- out = 20, -> out = 21
/// is_marker[25] = false       mapping[25] = 21 <- out = 21, -> out = 22
/// is_marker[26] = false       mapping[26] = 22 <- out = 22, -> out = 23
/// is_marker[27] = false       mapping[27] = 23 <- out = 23, -> out = 24
/// is_marker[28] = false       mapping[28] = 24 <- out = 24, -> out = 25
/// is_marker[29] = false       mapping[29] = 25 <- out = 25, -> out = 26
/// is_marker[30] = false       mapping[30] = 26 <- out = 26, -> out = 27
/// is_marker[31] = false       mapping[31] = 27 <- out = 27, -> out = 28
/// is_marker[32] = false       mapping[32] = 28 <- out = 28, -> out = 29
/// is_marker[33] = false       mapping[33] = 29 <- out = 29, -> out = 30
/// is_marker[34] = false       mapping[34] = 30 <- out = 30, -> out = 31
/// is_marker[35] = false       mapping[35] = 31 <- out = 31, -> out = 32
/// is_marker[36] = false       mapping[36] = 32 <- out = 32, -> out = 33
/// is_marker[37] = false       mapping[37] = 33 <- out = 33, -> out = 34
/// is_marker[38] = false       mapping[38] = 34 <- out = 34, -> out = 35
/// is_marker[39] = false       mapping[39] = 35 <- out = 35, -> out = 36
/// is_marker[40] = false       mapping[40] = 36 <- out = 36, -> out = 37
/// is_marker[41] = false       mapping[41] = 37 <- out = 37, -> out = 38
/// is_marker[42] = false       mapping[42] = 38 <- out = 38, -> out = 39
/// is_marker[43] = false       mapping[43] = 39 <- out = 39, -> out = 40
/// is_marker[44] = false       mapping[44] = 40 <- out = 40, -> out = 41
/// is_marker[45] = false       mapping[45] = 41 <- out = 41, -> out = 42
/// is_marker[46] = false       mapping[46] = 42 <- out = 42, -> out = 43
/// is_marker[47] = false       mapping[47] = 43 <- out = 43, -> out = 44
/// is_marker[48] = false       mapping[48] = 44 <- out = 44, -> out = 45
/// is_marker[49] = false       mapping[49] = 45 <- out = 45, -> out = 46
/// is_marker[50] = false       mapping[50] = 46 <- out = 46, -> out = 47
/// is_marker[51] = false       mapping[51] = 47 <- out = 47, -> out = 48
/// is_marker[52] = false       mapping[52] = 48 <- out = 48, -> out = 49
/// is_marker[53] = false       mapping[53] = 49 <- out = 49, -> out = 50
/// is_marker[54] = false       mapping[54] = 50 <- out = 50, -> out = 51
/// is_marker[55] = false       mapping[55] = 51 <- out = 51, -> out = 52
/// is_marker[56] = false       mapping[56] = 52 <- out = 52, -> out = 53
/// is_marker[57] = false       mapping[57] = 53 <- out = 53, -> out = 54
/// is_marker[58] = false       mapping[58] = 54 <- out = 54, -> out = 55
/// is_marker[59] = false       mapping[59] = 55 <- out = 55, -> out = 56
/// is_marker[60] = true        mapping[60] = 56 <- out = 56, -> out = 56
/// is_marker[61] = false       mapping[61] = 56 <- out = 56, -> out = 57
/// is_marker[62] = false       mapping[62] = 57 <- out = 57, -> out = 58
/// is_marker[63] = false       mapping[63] = 58 <- out = 58, -> out = 59
/// is_marker[64] = false       mapping[64] = 59 <- out = 59, -> out = 60
/// is_marker[65] = false       mapping[65] = 60 <- out = 60, -> out = 61
/// is_marker[66] = false       mapping[66] = 61 <- out = 61, -> out = 62
/// is_marker[67] = false       mapping[67] = 62 <- out = 62, -> out = 63
/// is_marker[68] = false       mapping[68] = 63 <- out = 63, -> out = 64
/// is_marker[69] = false       mapping[69] = 64 <- out = 64, -> out = 65
/// is_marker[70] = false       mapping[70] = 65 <- out = 65, -> out = 66
/// is_marker[71] = false       mapping[71] = 66 <- out = 66, -> out = 67
/// is_marker[72] = false       mapping[72] = 67 <- out = 67, -> out = 68
/// is_marker[73] = true        mapping[73] = 68 <- out = 68, -> out = 68
/// is_marker[74] = false       mapping[74] = 68 <- out = 68, -> out = 69
/// is_marker[75] = true        mapping[75] = 69 <- out = 69, -> out = 69
/// is_marker[76] = true        mapping[76] = 69 <- out = 69, -> out = 69
///                             mapping[77] = 69
/// 
/// Callers use the returned `is_marker` and `mapping` vectors to make adjustments 
/// to `MarkerEvent` vector after removing the marker indicators. After adjustments:
/// 
///     verify_marker(&events[0], 2, 0, 69);
///     verify_marker(&events[1], 1, 14, 18);
///     verify_marker(&events[2], 1, 56, 68);
fn get_markers_global_mapping(marker_result: &MarkerResult) -> (Vec<bool>, Vec<usize>) {
    let clean_text = &marker_result.escape.clean_text;
    let events = &marker_result.marker_events;

    // Mark all marker bytes.
    let mut is_marker = vec![false; clean_text.len()];

    for e in events {
        let o = e.opening.start_byte;
        let c = e.closing.start_byte;
        let count = e.opening.count;

        for i in o .. o + count {
            is_marker[i] = true;
        }
        for i in c .. c + count {
            is_marker[i] = true;
        }
    }

    // Build the mapping.
    let mut mapping = vec![0; clean_text.len() + 1];
    let mut out = 0;

    for i in 0..clean_text.len() {
        mapping[i] = out;
        if !is_marker[i] {
            out += 1;
        }
    }

    mapping[clean_text.len()] = out;

    (is_marker, mapping)

}

/// Removed markers from clean / escaped text, and returned the new text as string.
/// The original markdown text: `***bold*** \*` returned as `bold ` (i.e. 
/// `bold ESCAPED_ASTERISK`).
fn get_markers_removed_clean_text(marker_result: &MarkerResult, 
    is_marker: &[bool]
) -> String {
    // For markdown text: `***bold*** \*`, `clean_text` is now: `***bold*** `,
    // or `***bold*** ESCAPED_ASTERISK`.
    let clean_text = &marker_result.escape.clean_text;

    // Final clean text.
    let bytes = clean_text.as_bytes();
    let mut final_text = String::new();
    let mut i = 0;

    while i < bytes.len() {
        if !is_marker[i] {
            // Copy this byte as part of UTF-8 char.
            let ch = clean_text[i..].chars().next().unwrap();
            final_text.push(ch);
            i += ch.len_utf8();
        } else {
            // Skip marker bytes.
            i += 1;
        }
    }

    final_text
}

/// Markers have been removed from the text, now adjusted the opening and closing 
/// `Marker::start_byte`.
/// See `get_markers_global_mapping()` for detailed explanation on parameter 
/// `adjusted_mapping`.
fn adjust_markers_start_byte(marker_result: &mut MarkerResult, adjusted_mapping: &[usize]) {
    let events = &mut marker_result.marker_events;

    for e in events.iter_mut() {
        // Adjust opening.
        let o = e.opening.start_byte;
        e.opening.set_start_byte(adjusted_mapping[o]);

        // Adjust closing.
        let c = e.closing.start_byte;
        e.closing.set_start_byte(adjusted_mapping[c]);
    }
}

/// The input markdown has all markers removed and all opening and closing `Marker`s
/// have their `start_byte` adjusted accordingly. 
/// 
/// Note: in `final_text`, escaped `*` is still represented by ``, i.e. `ESCAPED_ASTERISK`.
/// Original markdown text: `***bold*** \*`; `final_text`: `bold `.
/// 
/// Based on opening and closing `Marker`s' `start_byte` and `count`, generates `Span`s 
/// text slices index into `final_text`.
fn generate_spans(marker_result: &MarkerResult, final_text: &str) -> Vec<Span> {
    let events = &marker_result.marker_events;

    let mut styled_spans = Vec::new();

    for e in events.iter() {
        styled_spans.push( Span::new(e.opening.start_byte, 
            e.closing.start_byte, e.opening.count as u8) );
    }
    styled_spans.sort_by_key(|s| s.start());

    let mut all_spans = Vec::new();
    let mut cursor = 0;

    for s in styled_spans {
        if cursor < s.start() {
            // Gap before this styled span → Normal span.
            all_spans.push(Span::normal(cursor, s.start()));
        }

        all_spans.push(s.clone());
        // Nested markdown: never let cursor move backwards.
        cursor = cursor.max(s.end());
    }

    // Tail after last styled span.
    if cursor < final_text.len() {
        all_spans.push(Span::normal(cursor, final_text.len()));
    }

    all_spans
}

pub fn parse_inline(markdown_text: &str) -> InlineParseResult {
    let mut res = get_marker_events(markdown_text);

    let (is_marker, mapping) = get_markers_global_mapping(&res);

    adjust_markers_start_byte(&mut res, &mapping);

    let final_text = get_markers_removed_clean_text(&res, &is_marker);

    let all_spans = generate_spans(&res, &final_text);
        
    InlineParseResult::new(final_text, all_spans)
}

// To run test for this module only: 
// 
//     * cargo test inline_parser::tests
//
//     * cargo test inline_parser::tests::test_get_marker_events_valid_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_valid_02 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_uneven_markers_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_valid_03 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_valid_04 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_escaped_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_escaped_02 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_escaped_03 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_escaped_04 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_escaped_05 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_nested_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_nested_02 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_bug_fix_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_bug_fix_02 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_get_marker_events_bug_fix_03 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_valid_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_valid_02 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_uneven_markers_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_valid_03 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_valid_04 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_escaped_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_escaped_02 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_escaped_03 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_escaped_04 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_escaped_05 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_nested_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_nested_02 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_bug_fix_01 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_bug_fix_02 -- --exact [--nocapture]
//     * cargo test inline_parser::tests::test_parse_inline_bug_fix_03 -- --exact [--nocapture]
#[cfg(test)]
/// Note: `let start = event.count + event.start_byte;` is based on the 
/// assumption that all marker indicators are single-byte.
mod tests {
    use super::*;
    use crate::document::SpanStyle;
    use constcat::concat;

    // Valid
    const MARKDOWN_01: &str = "— **Tưởng Vĩnh Kính**, Hồ Chí Minh Tại *Trung Quốc*, \
        Thượng Huyền dịch, ***trang 339***.";
    const CLEAN_01: &str = "— Tưởng Vĩnh Kính, Hồ Chí Minh Tại Trung Quốc, \
        Thượng Huyền dịch, trang 339.";

    const MARKDOWN_02: &str = "— **Tưởng Vĩnh Kính**, Hồ Chí Minh Tại *Trung Quốc*, \
        Thượng Huyền dịch, ***trang 339***.*";
    const CLEAN_02: &str = "— Tưởng Vĩnh Kính, Hồ Chí Minh Tại Trung Quốc, \
        Thượng Huyền dịch, trang 339.*";

    // MARKDOWN_03: `Tưởng Vĩnh Kính` in bold, followed by `*`.
    // MARKDOWN_04: `*Tưởng Vĩnh Kính` in bold.
    // MARKDOWN_05: `**` follows by `Tưởng Vĩnh Kính` in italic.
    const MARKDOWN_03: &str = "**Tưởng Vĩnh Kính***";
    const CLEAN_03: &str = "Tưởng Vĩnh Kính*";
    const MARKDOWN_04: &str = "***Tưởng Vĩnh Kính**";
    const CLEAN_04: &str = "*Tưởng Vĩnh Kính";
    const MARKDOWN_05: &str = "***Tưởng Vĩnh Kính*";
    const CLEAN_05: &str = "**Tưởng Vĩnh Kính";

    // Valid
    const MARKDOWN_06: &str = "Tưởng Vĩnh Kính (*)";
    // Valid adjacent markers
    const MARKDOWN_07: &str = "***bold***text**more**";
    const CLEAN_07: &str = "boldtextmore";

    // Valid escape
    const MARKDOWN_08: &str = r"***bold*** \*";
    const CLEAN_08: &str = "bold *";
    const MARKDOWN_09: &str = r"\*not bold\*";
    const CLEAN_09: &str = "*not bold*";
    const MARKDOWN_10: &str = r"**bold \*inside\***";
    const CLEAN_10: &str = "bold *inside*";
    const MARKDOWN_11: &str = r"\\Úc Đại Lợi\\";
    const CLEAN_11: &str = r"\Úc Đại Lợi\";
    const MARKDOWN_12: &str = r"**bold \\Úc Đại Lợi\\**";
    const CLEAN_12: &str = r"bold \Úc Đại Lợi\";

    // Valid nested
    const MARKDOWN_13: &str = "**bold *italic inside bold* bold**";
    const CLEAN_13: &str = "bold italic inside bold bold";
    const MARKDOWN_14: &str = "**Không đọc *sử* không đủ tư cách nói chuyện *chính trị*.**";
    const CLEAN_14: &str = "Không đọc sử không đủ tư cách nói chuyện chính trị.";

    const FINAL_MARKDOWN: &str = concat!(MARKDOWN_02, " ", MARKDOWN_03);

    // `MARKDOWN_02 + " " + MARKDOWN_03` introduces additional markdown, which makes 
    // the final text not equal to `CLEAN_02 + " " + CLEAN_03`!
    const FINAL_CLEAN: &str = concat!("— Tưởng Vĩnh Kính, Hồ Chí Minh Tại Trung Quốc, \
        Thượng Huyền dịch, trang 339. Tưởng Vĩnh Kính**");

    const MARKDOWN_BUG_01: &str = "( **Chính Ðạo, *Việt Nam Niên Biểu*, *Tập 1A***, trang 347 )";
    const CLEAN_BUG_01: &str = "( Chính Ðạo, Việt Nam Niên Biểu, Tập 1A, trang 347 )";

    const MARKDOWN_BUG_02: &str = "***bold***";
    const CLEAN_BUG_02: &str = "bold";
    const MARKDOWN_BUG_03: &str = "**xy, *bc*, *de***";
    const CLEAN_BUG_03: &str = "xy, bc, de";
    const MARKDOWN_BUG_04: &str = "***xy* z**";
    const CLEAN_BUG_04: &str = "xy z";

    // 0: . | 1: * | 2:  | 3: * | 4: * | 5: T | 6: Æ | 7: ° | 8: á | 9: » |
    // 10:  | 11: n | 12: g | 13:  | 14: V | 15: Ä | 16: © |  17: n | 18: h |
    // 19:  | 20: K | 21: Ã | 22: ­| 23: n | 24: h | 25: * | 26: * | 27: * 
    // This is an oddity, if this was a real intention, it should have been 
    // written more explicitly. These cases are not guaranteed to produce 
    // what the users expect. 
    //
    // A normal `.`, followed by an italic ` `, finally `. Tưởng Vĩnh Kính**` in space.
    const MARKDOWN_BUG_05: &str = ".* **Tưởng Vĩnh Kính***";
    const CLEAN_BUG_05: &str = ". Tưởng Vĩnh Kính**";

    fn verify_marker(marker: &MarkerEvent, count: usize, 
        opening_start_byte: usize, closing_start_byte: usize) {
        assert_eq!(marker.opening.count, count, "opening count");
        assert_eq!(marker.closing.count, count, "closing count");
        assert_eq!(marker.opening.start_byte, opening_start_byte, "opening start byte");
        assert_eq!(marker.closing.start_byte, closing_start_byte, "closing start byte");
    }

    #[test]
    fn test_get_marker_events_valid_01() {
        let res = get_marker_events(MARKDOWN_01);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 4, "number of events");

        verify_marker(&events[0], 2, 4, 26);
        verify_marker(&events[1], 1, 51, 64);
        verify_marker(&events[2], 2, 93, 106);
        verify_marker(&events[3], 1, 95, 105);

        let event = &events[0];
        assert_eq!(&clean_text[0..event.opening.start_byte], "— ", "span 1");

        // Marker indicators are single-byte.
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(start, 6, "start 1");

        assert_eq!(&clean_text[start..event.closing.start_byte], "Tưởng Vĩnh Kính", "span 2");

        let start = event.opening.count + event.closing.start_byte;
        assert_eq!(start, 28, "start 2");

        let event = &events[1];
        assert_eq!(&clean_text[start..event.opening.start_byte], ", Hồ Chí Minh Tại ", "span 3");

        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(start, 52, "start 3");

        assert_eq!(&clean_text[start..event.closing.start_byte], "Trung Quốc", "span 4");
        let start = event.opening.count + event.closing.start_byte;
        assert_eq!(start, 65, "start 4");

        let event = &events[2];
        assert_eq!(&clean_text[start..event.opening.start_byte], ", Thượng Huyền dịch, ", "span 5");

        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(start, 95, "start 5 bold");

        assert_eq!(&clean_text[start..event.closing.start_byte], "*trang 339*", "span 6 bold");

        let event = &events[3];

        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(start, 96, "start 5 italic");

        assert_eq!(&clean_text[start..event.closing.start_byte], "trang 339", "span 6 italic");

        let start = event.opening.count + event.closing.start_byte;
        assert_eq!(start, 106, "start 6");

        // "**." appears unintuitive -- at the final stage, the marker indicators
        //     get removed, `start_byte`s get adjusted. Perhaps it makes more sense 
        //     then.
        assert_eq!(&clean_text[start..], "**.", "span 7");
    }

    #[test]
    /// An extension of `test_get_marker_events_valid_01()`: the text just has has 
    /// an `*` appended to the end. The last span text is `.*` instead of `.`.
    fn test_get_marker_events_valid_02() {
        let res = get_marker_events(MARKDOWN_02);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 4, "number of events");

        let event = &events[3];

        let start = event.opening.count + event.closing.start_byte;
        assert_eq!(start, 106, "start 6");

        // "**." appears unintuitive -- at the final stage, the marker indicators
        //     get removed, `start_byte`s get adjusted. Perhaps it makes more sense 
        //     then.
        assert_eq!(&clean_text[start..], "**.*", "span 7");
    }

    #[test]
    fn test_get_marker_events_uneven_markers_01() {
        // `Tưởng Vĩnh Kính` in bold, followed by `*`.
        let res = get_marker_events(MARKDOWN_03);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(res.marker_events.len(), 1, "MARKDOWN_03 number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "Tưởng Vĩnh Kính", "MARKDOWN_03 span 1");

        // `*Tưởng Vĩnh Kính` in bold.
        let res = get_marker_events(MARKDOWN_04);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(res.marker_events.len(), 1, "MARKDOWN_04 number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "*Tưởng Vĩnh Kính", "MARKDOWN_04 span 1");

        // `**` follows by `Tưởng Vĩnh Kính` in italic.
        let res = get_marker_events(MARKDOWN_05);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(res.marker_events.len(), 1, "MARKDOWN_05 number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "Tưởng Vĩnh Kính", "MARKDOWN_05 span 1");
    }

    #[test]
    fn test_get_marker_events_valid_03() {
        let res = get_marker_events(MARKDOWN_06);
        assert_eq!(res.marker_events.len(), 0, "number of events");
        assert_eq!(&res.escape.clean_text, MARKDOWN_06, "text");
    }

    #[test]
    /// Handle adjacent markers.
    fn test_get_marker_events_valid_04() {
        let res = get_marker_events(MARKDOWN_07);
        let events = &res.marker_events;

        assert_eq!(events.len(), 3, "number of events");

        verify_marker(&events[0], 2, 0, 8);
        verify_marker(&events[1], 1, 2, 7);
        verify_marker(&events[2], 2, 14, 20);
    }

    #[test]
    /// Escape supporting: `\*` is treated as literal string `*`.
    fn test_get_marker_events_escaped_01() {
        let res = get_marker_events(MARKDOWN_08);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 2, "number of events");

        verify_marker(&events[0], 2, 0, 8);
        verify_marker(&events[1], 1, 2, 7);

        let event = &events[0];
        // Marker indicators are single-byte.
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "*bold*", "span 1 bold");

        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "bold", "span 1 italic");

        let start = event.opening.count + event.closing.start_byte;
        let span_2 = &clean_text[start..];
        // "** *" appears unintuitive -- at the final stage, the marker indicators
        //     get removed, `start_byte`s get adjusted. Perhaps it makes more sense 
        //     then.
        assert_eq!(span_2, "** ", "span 2");
    }

    #[test]
    /// Escape supporting: `\*` is treated as literal string `*`.
    /// `r"\*not bold\*"` → `*not bold*`, no markers.
    fn test_get_marker_events_escaped_02() {
        let res = get_marker_events(MARKDOWN_09);
        let events = &res.marker_events;
        // Note: must call reserve_asterisk()!
        let clean_text = reserve_asterisk(&res.escape.clean_text);

        assert_eq!(events.len(), 0, "number of events");
        assert_eq!(clean_text, CLEAN_09, "clean text");
    }

    #[test]
    /// `r"**bold \*inside\***"` → bold span over `bold *inside*`.
    /// Gets turned into: `**bold inside**` which is 21 bytes.
    /// Then `**bold inside**` is fed into the parser.
    fn test_get_marker_events_escaped_03() {
        let res = get_marker_events(MARKDOWN_10);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 1, "number of events");

        verify_marker(&events[0], 2, 0, 19);

        let event = &events[0];
        // Marker indicators are single-byte.
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "bold inside", "span 1");
    }

    #[test]
    /// `r"\\Úc Đại Lợi\\"` → `\Úc Đại Lợi\`, no markers.
    fn test_get_marker_events_escaped_04() {
        let res = get_marker_events(MARKDOWN_11);
        let events = &res.marker_events;
        // Note: must call reserve_asterisk()!
        let clean_text = reserve_asterisk(&res.escape.clean_text);

        assert_eq!(events.len(), 0, "number of events");
        assert_eq!(clean_text, CLEAN_11, "clean text");
    }

    #[test]
    /// `r"**bold \\Úc Đại Lợi\\**"` → bold span over `bold \Úc Đại Lợi\`.
    /// Gets turned into: `**bold \Úc Đại Lợi\**` which is 27 bytes.
    fn test_get_marker_events_escaped_05() {
        let res = get_marker_events(MARKDOWN_12);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 1, "number of events");

        verify_marker(&events[0], 2, 0, 25);

        let event = &events[0];
        // Marker indicators are single-byte.
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], CLEAN_12, "span 1");
    }

    #[test]
    /// Nested markdown results in overlapped spans.
    fn test_get_marker_events_nested_01() {
        let res = get_marker_events(MARKDOWN_13);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 2, "number of events");

        verify_marker(&events[0], 2, 0, 32);
        verify_marker(&events[1], 1, 7, 26);

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "bold *italic inside bold* bold", "span 1");

        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "italic inside bold", "span 2");
    }

    #[test]
    /// Nested markdown results in overlapped spans.
    fn test_get_marker_events_nested_02() {
        let res = get_marker_events(MARKDOWN_14);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 3, "number of events");

        verify_marker(&events[0], 2, 0, 75);
        verify_marker(&events[1], 1, 16, 21);
        verify_marker(&events[2], 1, 60, 73);

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "Không đọc *sử* không đủ tư cách nói chuyện *chính trị*.", "span 1");

        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "sử", "span 2");

        let event = &events[2];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "chính trị", "span 3");
    }

    #[test]
    fn test_get_marker_events_bug_fix_01() {
        /*
        let res = get_marker_events("a, **xy, *bc*, *de***, w");
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 3, "number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "xy, *bc*, *de*", "span 1");
            
        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "bc", "span 2");

        let event = &events[2];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "de", "span 3");
        */

        let res = get_marker_events(MARKDOWN_BUG_01);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 3, "number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "Chính Ðạo, *Việt Nam Niên Biểu*, *Tập 1A*", "span 1");
            
        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "Việt Nam Niên Biểu", "span 2");

        let event = &events[2];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], "Tập 1A", "span 3");
    }

    #[test]
    fn test_get_marker_events_bug_fix_02() {
        let res = get_marker_events(MARKDOWN_BUG_02);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 2, "MARKDOWN_BUG_02 number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "*bold*", "MARKDOWN_BUG_02 span 1 bold");
            
        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "bold", "MARKDOWN_BUG_02 span 1 italic");

        //
        let res = get_marker_events(MARKDOWN_BUG_03);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 3, "MARKDOWN_BUG_03 number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "xy, *bc*, *de*", "MARKDOWN_BUG_03 span 1 bold");
            
        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "bc", "MARKDOWN_BUG_03 span 2 italic");

        let event = &events[2];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "de", "MARKDOWN_BUG_03 span 3 italic");

        //
        let res = get_marker_events(MARKDOWN_BUG_04);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 2, "MARKDOWN_BUG_04 number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "*xy* z", "MARKDOWN_BUG_04 span 1 bold");
            
        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "xy", "MARKDOWN_BUG_04 span 2 italic");

    }

    #[test]
    // This is an oddity, if this was a real intention, it should have been 
    // written more explicitly. These cases are not guaranteed to produce 
    // what the users expect.     
    fn test_get_marker_events_bug_fix_03() {
        let res = get_marker_events(MARKDOWN_BUG_05);
        let events = &res.marker_events;
        let clean_text = &res.escape.clean_text;

        assert_eq!(events.len(), 2, "MARKDOWN_BUG_05 number of events");

        let event = &events[0];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            " ", "MARKDOWN_BUG_05 span 1 italic");
            
        let event = &events[1];
        let start = event.opening.count + event.opening.start_byte;
        assert_eq!(&clean_text[start..event.closing.start_byte], 
            "Tưởng Vĩnh Kính", "MARKDOWN_BUG_05 span 2 italic");
    }

    fn verify_span(span: &Span, final_text: &str, text: &str, style: SpanStyle, label: &str) {
        assert_eq!(final_text[span.start()..span.end()], *text, 
            "{}", format!("{}: text", label));
        assert_eq!(*span.style(), style, "{}", format!("{}: style", label));
    }

    #[test]
    fn test_parse_inline_valid_01() {
        let res = parse_inline(MARKDOWN_01);

        let text = res.text();

        assert_eq!(text, CLEAN_01, "text");
        
        let spans = res.spans();
        assert_eq!(spans.len(), 8, "span length");

        verify_span(&spans[0], text, "— ", SpanStyle::Normal, "span 1");
        verify_span(&spans[1], text, "Tưởng Vĩnh Kính", SpanStyle::Bold, "span 2");
        verify_span(&spans[2], text, ", Hồ Chí Minh Tại ", SpanStyle::Normal, "span 3");
        verify_span(&spans[3], text, "Trung Quốc", SpanStyle::Italic, "span 4");
        verify_span(&spans[4], text, ", Thượng Huyền dịch, ", SpanStyle::Normal, "span 5");
        verify_span(&spans[5], text, "trang 339", SpanStyle::Bold, "span 6");
        verify_span(&spans[6], text, "trang 339", SpanStyle::Italic, "span 7");
        verify_span(&spans[7], text, ".", SpanStyle::Normal, "span 8");
    }

    #[test]
    /// An extension of `test_parse_inline_valid_01()`: the text just has has 
    /// an `*` appended to the end. The last span text is `.*` instead of `.`.
    fn test_parse_inline_valid_02() {
        let res = parse_inline(MARKDOWN_02);

        let text = res.text();

        assert_eq!(text, CLEAN_02, "text");
        
        let spans = res.spans();
        assert_eq!(spans.len(), 8, "span length");

        verify_span(&spans[7], text, ".*", SpanStyle::Normal, "span 7");
    }

    #[test]
    fn test_parse_inline_uneven_markers_01() {
        let res = parse_inline(MARKDOWN_03);

        let text = res.text();
        assert_eq!(text, CLEAN_03, "CLEAN_03 text");
        let spans = res.spans();
        assert_eq!(spans.len(), 2, "MARKDOWN_03 span length");
        verify_span(&spans[0], text, "Tưởng Vĩnh Kính", SpanStyle::Bold, "MARKDOWN_03 span 1");
        verify_span(&spans[1], text, "*", SpanStyle::Normal, "MARKDOWN_03 span 2");

        //
        let res = parse_inline(MARKDOWN_04);

        let text = res.text();
        assert_eq!(text, CLEAN_04, "CLEAN_04 text");
        let spans = res.spans();
        assert_eq!(spans.len(), 1, "MARKDOWN_04 span length");
        verify_span(&spans[0], text, "*Tưởng Vĩnh Kính", SpanStyle::Bold, "MARKDOWN_04 span 1");

        //
        let res = parse_inline(MARKDOWN_05);

        let text = res.text();
        assert_eq!(text, CLEAN_05, "CLEAN_05 text");
        let spans = res.spans();
        assert_eq!(spans.len(), 2, "MARKDOWN_05 span length");
        verify_span(&spans[0], text, "**", SpanStyle::Normal, "MARKDOWN_05 span 1");
        verify_span(&spans[1], text, "Tưởng Vĩnh Kính", SpanStyle::Italic, "MARKDOWN_05 span 2");
    }    

    #[test]
    fn test_parse_inline_valid_03() {
        let res = parse_inline(MARKDOWN_06);

        let text = res.text();

        assert_eq!(text, MARKDOWN_06, "text");

        let spans = res.spans();
        assert_eq!(spans.len(), 1, "span length");

        verify_span(&spans[0], text, MARKDOWN_06, SpanStyle::Normal, "span 1");
    }

    #[test]
    /// Handle adjacent markers.
    fn test_parse_inline_valid_04() {
        let res = parse_inline(MARKDOWN_07);
        
        let text = res.text();

        assert_eq!(text, CLEAN_07, "text");
        
        let spans = res.spans();
        assert_eq!(spans.len(), 4, "span length");

        verify_span(&spans[0], text, "bold", SpanStyle::Bold, "span 1 bold");
        verify_span(&spans[1], text, "bold", SpanStyle::Italic, "span 1 italic");
        verify_span(&spans[2], text, "text", SpanStyle::Normal, "span 2");
        verify_span(&spans[3], text, "more", SpanStyle::Bold, "span 3");
    }

    #[test]
    /// Escape supporting: `\*` is treated as literal string `*`.
    fn test_parse_inline_escaped_01() {
        let res = parse_inline(MARKDOWN_08);

        let text = res.text();

        // Note reserve_asterisk() call here and in verify_span().
        assert_eq!(reserve_asterisk(text), CLEAN_08, "text");
        
        let spans = res.spans();
        assert_eq!(spans.len(), 3, "span length");

        verify_span(&spans[0], text, "bold", SpanStyle::Bold, "span 1 bold");
        verify_span(&spans[1], text, "bold", SpanStyle::Italic, "span 1 italic");
        verify_span(&spans[2], text, " ", SpanStyle::Normal, "span 2");
    }

    #[test]    
    /// Escape supporting: `\*` is treated as literal string `*`.
    /// `r"\*not bold\*"` → `*not bold*`, no markers.
    fn test_parse_inline_escaped_02() {
        let res = parse_inline(MARKDOWN_09);

        let text = res.text();

        // Note reserve_asterisk() call here and in verify_span().
        assert_eq!(reserve_asterisk(text), CLEAN_09, "text");

        let spans = res.spans();
        assert_eq!(spans.len(), 1, "span length");

        verify_span(&spans[0], text, "not bold", SpanStyle::Normal, "span 1");
    }

    #[test]
    /// Escape supporting: `\*` is treated as literal string `*`.
    /// `r"**bold \*inside\***"` → bold span over `bold *inside*`.
    fn test_parse_inline_escaped_03() {
        let res = parse_inline(MARKDOWN_10);

        let text = res.text();

        // Note reserve_asterisk() call here and in verify_span().
        assert_eq!(reserve_asterisk(text), CLEAN_10, "text");

        let spans = res.spans();
        assert_eq!(spans.len(), 1, "span length");

        verify_span(&spans[0], text, "bold inside", SpanStyle::Bold, "span 1");
    }

    #[test]
    /// Escape supporting: `\\` is treated as literal string `\`.
    /// `r"\\Úc Đại Lợi\\"` → `\Úc Đại Lợi\`, no markers.
    fn test_parse_inline_escaped_04() {
        let res = parse_inline(MARKDOWN_11);

        let text = res.text();

        // Note reserve_asterisk() call here and in verify_span().
        assert_eq!(reserve_asterisk(text), CLEAN_11, "text");

        let spans = res.spans();
        assert_eq!(spans.len(), 1, "span length");

        verify_span(&spans[0], text, r"\Úc Đại Lợi\", SpanStyle::Normal, "span 1");
    }

    #[test]
    /// Escape supporting: `\\` is treated as literal string `\`.
    /// `r"**bold \\Úc Đại Lợi\\**"` → bold span over `bold \Úc Đại Lợi\`.
    fn test_parse_inline_escaped_05() {
        let res = parse_inline(MARKDOWN_12);

        let text = res.text();

        // Note reserve_asterisk() call here and in verify_span().
        assert_eq!(reserve_asterisk(text), CLEAN_12, "text");

        let spans = res.spans();
        assert_eq!(spans.len(), 1, "span length");

        verify_span(&spans[0], text, r"bold \Úc Đại Lợi\", SpanStyle::Bold, "span 1");
    }

    #[test]
    /// Nested markdown results in overlapped spans.
    fn test_parse_inline_nested_01() {
        let res = parse_inline(MARKDOWN_13);

        let text = res.text();

        // Note reserve_asterisk() call here and in verify_span().
        assert_eq!(reserve_asterisk(text), CLEAN_13, "text");

        let spans = res.spans();
        assert_eq!(spans.len(), 2, "span length");

        verify_span(&spans[0], text, CLEAN_13, SpanStyle::Bold, "span 1");
        verify_span(&spans[1], text, "italic inside bold", SpanStyle::Italic, "span 2");
    }

    #[test]
    /// Nested markdown results in overlapped spans.
    fn test_parse_inline_nested_02() {
        let res = parse_inline(MARKDOWN_14);

        let text = res.text();

        // Note reserve_asterisk() call here and in verify_span().
        assert_eq!(reserve_asterisk(text), CLEAN_14, "text");

        let spans = res.spans();
        assert_eq!(spans.len(), 3, "span length");

        verify_span(&spans[0], text, CLEAN_14, SpanStyle::Bold, "span 1");
        verify_span(&spans[1], text, "sử", SpanStyle::Italic, "span 2");
        verify_span(&spans[2], text, "chính trị", SpanStyle::Italic, "span 3");
    }

    #[test]
    fn test_parse_inline() {
        let res = parse_inline(FINAL_MARKDOWN);

        let text = res.text();

        assert_eq!(reserve_asterisk(text), FINAL_CLEAN, "text");
        
        let spans = res.spans();
        // MARKDOWN_02 → 8; MARKDOWN_03 → 11; 
        assert_eq!(spans.len(), 11, "span length");

        // MARKDOWN_02.
        verify_span(&spans[0], text, "— ", SpanStyle::Normal, "span 1");
        verify_span(&spans[1], text, "Tưởng Vĩnh Kính", SpanStyle::Bold, "span 2");
        verify_span(&spans[2], text, ", Hồ Chí Minh Tại ", SpanStyle::Normal, "span 3");
        verify_span(&spans[3], text, "Trung Quốc", SpanStyle::Italic, "span 4");
        verify_span(&spans[4], text, ", Thượng Huyền dịch, ", SpanStyle::Normal, "span 5");
        verify_span(&spans[5], text, "trang 339", SpanStyle::Bold, "span 6 bold");
        verify_span(&spans[6], text, "trang 339", SpanStyle::Italic, "span 6 italic");
        verify_span(&spans[7], text,".", SpanStyle::Normal, "span 7 normal");

        // MARKDOWN_03.
        verify_span(&spans[8], text," ", SpanStyle::Italic, "span 8 normal");
        verify_span(&spans[9], text, "Tưởng Vĩnh Kính", SpanStyle::Italic, "span 9 italic");
        verify_span(&spans[10], text, "**", SpanStyle::Normal, "span 10 normal");
    }

    #[test]
    fn test_parse_inline_bug_fix_01() {
        let res = parse_inline(MARKDOWN_BUG_01);

        let text = res.text();

        assert_eq!(reserve_asterisk(text), CLEAN_BUG_01, "text");

        let spans = res.spans();
        assert_eq!(spans.len(), 5, "span length");

        verify_span(&spans[0], text, "( ", SpanStyle::Normal, "span 1");
        verify_span(&spans[1], text, "Chính Ðạo, Việt Nam Niên Biểu, Tập 1A", SpanStyle::Bold, "span 2");
        verify_span(&spans[2], text, "Việt Nam Niên Biểu", SpanStyle::Italic, "span 3");
        verify_span(&spans[3], text, "Tập 1A", SpanStyle::Italic, "span 4");
        verify_span(&spans[4], text, ", trang 347 )", SpanStyle::Normal, "span 5");
    }

    #[test]
    fn test_parse_inline_bug_fix_02() {
        let res = parse_inline(MARKDOWN_BUG_02);

        let text = res.text();
        assert_eq!(reserve_asterisk(text), CLEAN_BUG_02, "CLEAN_BUG_02 text");

        let spans = res.spans();
        assert_eq!(spans.len(), 2, "MARKDOWN_BUG_02 span length");

        verify_span(&spans[0], text, "bold", SpanStyle::Bold, "MARKDOWN_BUG_02 span 1 bold");
        verify_span(&spans[1], text, "bold", SpanStyle::Italic, "MARKDOWN_BUG_02 span 1 italic");

        //
        let res = parse_inline(MARKDOWN_BUG_03);

        let text = res.text();
        assert_eq!(reserve_asterisk(text), CLEAN_BUG_03, "CLEAN_BUG_03 text");

        let spans = res.spans();
        assert_eq!(spans.len(), 3, "MARKDOWN_BUG_03 span length");

        verify_span(&spans[0], text, "xy, bc, de", SpanStyle::Bold, "MARKDOWN_BUG_03 span 1 bold");
        verify_span(&spans[1], text, "bc", SpanStyle::Italic, "MARKDOWN_BUG_03 span 2 italic");
        verify_span(&spans[2], text, "de", SpanStyle::Italic, "MARKDOWN_BUG_03 span 3 italic");

        //
        let res = parse_inline(MARKDOWN_BUG_04);

        let text = res.text();
        assert_eq!(reserve_asterisk(text), CLEAN_BUG_04, "CLEAN_BUG_04 text");

        let spans = res.spans();
        assert_eq!(spans.len(), 2, "MARKDOWN_BUG_04 span length");

        verify_span(&spans[0], text, "xy z", SpanStyle::Bold, "MARKDOWN_BUG_04 span 1 bold");
        verify_span(&spans[1], text, "xy", SpanStyle::Italic, "MARKDOWN_BUG_04 span 2 italic");
    }

    #[test]
    // This is an oddity, if this was a real intention, it should have been 
    // written more explicitly. These cases are not guaranteed to produce 
    // what the users expect.
    fn test_parse_inline_bug_fix_03() {
        let res = parse_inline(MARKDOWN_BUG_05);

        let text = res.text();
        assert_eq!(reserve_asterisk(text), CLEAN_BUG_05, "MARKDOWN_BUG_05 text");

        let spans = res.spans();
        assert_eq!(spans.len(), 4, "MARKDOWN_BUG_05 span length");

        verify_span(&spans[0], text, ".", SpanStyle::Normal, "MARKDOWN_BUG_05 span 1 normal");
        verify_span(&spans[1], text, " ", SpanStyle::Italic, "MARKDOWN_BUG_05 span 2 italic");
        verify_span(&spans[2], text, "Tưởng Vĩnh Kính", SpanStyle::Italic, "MARKDOWN_BUG_05 span 3 italic");
        verify_span(&spans[3], text, "**", SpanStyle::Normal, "MARKDOWN_BUG_05 span 4 normal");
    }
}