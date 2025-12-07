// 30/10/2025

use std::collections::BTreeMap;
use ttf_parser::Face;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use lopdf::StringFormat;

use super::{
    A4, 
    a4_default_content_height, 
    A4_DEFAULT_MARGINS, 
    LINE_SPACING_FACTOR,
    PdfTextContent, 
};

struct PdfDocument {
    pub doc: Document,
    pub pages_id: (u32, u16),
    pub fd_id: (u32, u16),
}

// Copy and Paste mapping.
fn tounicode_mapping(cmap: &mut String, 
    pdf_text: &PdfTextContent,
    face: &Face
) {
    // Chunked bfchar
    const CHUNK: usize = 100;

    for chunk in pdf_text.copy_paste_unicodes().chunks(CHUNK) {
        cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
        for c in chunk {
            if let Some(ch) = char::from_u32(*c as u32) {
                if let Some(gid) = face.glyph_index(ch) {
                    // Known glyph → map CID → Unicode normally
                    cmap.push_str(&format!("<{:04X}> <{:04X}>\n", gid.0, c));
                } else {
                    // Missing glyph → map CID 0 to U+FFFD
                    cmap.push_str("<0000> <FFFD>\n");
                }
            } else {
                // Invalid UTF-16 code unit → also fallback
                cmap.push_str("<0000> <FFFD>\n");
            }
        }
        cmap.push_str("endbfchar\n");
    }
}

/// Create a minimal ToUnicode CMap for mapping CID -> Unicode
/// 
/// ToUnicode CMap stream (map cids -> unicode code points)
/// For simplicity we built it from the original unicode characters
/// so it maps the CID values we will emit (which are glyph IDs) to the same codepoints.
/// This makes copy/paste produce the original Unicode text. 
fn make_to_unicode_cmap(pdf_text: &PdfTextContent, face: &Face) -> Stream {
    let mut cmap = String::new();
    cmap.push_str("/CIDInit /ProcSet findresource begin\n");
    cmap.push_str("12 dict begin\n");
    cmap.push_str("begincmap\n");
    cmap.push_str("/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> def\n");
    cmap.push_str("/CMapName /Identity-H def\n");
    // /CMapType is set to 2 in “ToUnicode” mapping files.
    cmap.push_str("/CMapType 2 def\n");
    // Because a “ToUnicode” mapping file is used to convert from CIDs (which begin at decimal 0,
    // which is expressed as 0x0000 in hexadecimal notation) to Unicode code points, the following
    // “codespacerange” definition, without exception, shall always be used:
    cmap.push_str("1 begincodespacerange\n");
    cmap.push_str("<0000> <FFFF>\n");
    cmap.push_str("endcodespacerange\n");

    // Route unassigned codes to `.notdef`
    cmap.push_str("1 beginnotdefrange\n");
    cmap.push_str("<0000> <FFFF> 0\n");
    cmap.push_str("endnotdefrange\n");

    // Map CID 0 → U+FFFD
    cmap.push_str("1 beginbfchar\n");
    cmap.push_str("<0000> <FFFD>\n");
    cmap.push_str("endbfchar\n");

    tounicode_mapping(&mut cmap, pdf_text, face);

    cmap.push_str("endcmap\n");
    cmap.push_str("CMapName currentdict /CMap defineresource pop\n");
    cmap.push_str("end\n");
    cmap.push_str("end\n");

    Stream::new(dictionary! {}, cmap.into_bytes())
}

fn create_font_stream(doc: &mut Document, font_data: &[u8]) -> (u32, u16) {
    // Create an indirect stream object for the font program (FontFile2)
    let font_stream_id = {
        let s = Stream::new(dictionary! {}, font_data.to_vec());
        doc.add_object(s)
    };
    
    font_stream_id
}

fn create_font_descriptor(doc: &mut Document, 
    font_name: &str, 
    face: &Face, 
    font_stream_id: (u32, u16)
) -> (u32, u16) {
        let bbox = face.global_bounding_box();

        let italic_angle = face.italic_angle();
        // 32: nonsymbolic
        let flags = if italic_angle != 0.0 { 64 } else { 32 };
        // 400: regular weight
        let weight = face.weight().to_number(); // ttf-parser supports this

        // Create FontDescriptor (reference FontFile2 indirectly)
        let fd_id = doc.add_object(dictionary! {
            "Type" => "FontDescriptor",
            "FontName" => Object::Name(font_name.as_bytes().to_vec()),   // name object
            "FontBBox" => vec![
                bbox.x_min.into(),
                bbox.y_min.into(),
                bbox.x_max.into(),
                bbox.y_max.into(),
            ],
            // minimal useful metrics: Ascent/Descent/Flags/FontBBox etc are helpful but optional for some viewers.
            // We'll add a couple of numeric keys where possible (approximate) using ttf-parser metrics:
            "Ascent" => face.ascender() as i64,
            "Descent" => face.descender() as i64,
            "CapHeight" => face.ascender() as i64,
            "ItalicAngle" => 0,
            "StemV" => 80,
            "Flags" => flags,
            "FontWeight" => weight,
            // Embed the font program as an indirect reference
            "FontFile2" => font_stream_id,
        });

        fd_id
}

fn get_width_maps(used_cids: &Vec<u16>, face: &Face, units_per_em: f32) -> BTreeMap<u16, i64> {
    // Build a compact W array: PDF expects something like [firstCID [w0 w1 ...]]
    // We'll build a single contiguous range if possible. If not contiguous, we will group contiguous ranges.
    // We'll compute widths from ttf-parser (glyph horizontal advance), scaled to DW=1000 em.
    let mut widths_map: BTreeMap<u16, i64> = BTreeMap::new();
    for &cid in used_cids {
        let gid = ttf_parser::GlyphId(cid);
        // Try to get horizontal advance from font; fallback to units_per_em if missing
        let adv = face.glyph_hor_advance(gid).unwrap_or(face.units_per_em());
        // Convert to the PDF width units for DW=1000 em. We'll choose DW=1000.
        let width1000 = ((adv as f32 / units_per_em) * 1000.0).round() as i64;
        widths_map.insert(cid, width1000.max(0));
    }

    widths_map
}

fn build_w_array(widths_map: BTreeMap<u16, i64>) -> Vec<Object> {
    // Build W array for CIDFont: group contiguous CIDs into ranges
    // Format: [firstCid arrayOfWidths ...] where arrayOfWidths are integers
    let mut w_array: Vec<Object> = Vec::new();
    let mut iter = widths_map.into_iter().peekable();
    while let Some((start_cid, _)) = iter.peek().cloned() {
        // collect contiguous run starting at start_cid
        let mut run = Vec::new();
        let mut current = start_cid;
        while let Some((cid, width)) = iter.peek().cloned() {
            if cid == current {
                run.push(width);
                current = current.wrapping_add(1);
                iter.next();
            } else {
                break;
            }
        }
        // append start cid and array of widths
        w_array.push(Object::Integer(start_cid as i64));
        let arr = run.into_iter().map(|w| Object::Integer(w)).collect::<Vec<_>>();
        w_array.push(Object::Array(arr));
    }
    w_array
}

fn create_cid_font_type2(doc: &mut Document, 
    font_name: &str, 
    fd_id: (u32, u16), 
    w_array: Vec<Object>
) -> (u32, u16) {
    // Create CIDFontType2 (descendant). Use CIDToGIDMap /Identity and DW=1000
    let cidfont_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "CIDFontType2",
        "BaseFont" => Object::Name(font_name.as_bytes().to_vec()),
        "CIDSystemInfo" => dictionary! {
            "Registry" => Object::string_literal("Adobe"),
            "Ordering" => Object::string_literal("Identity"),
            "Supplement" => 0,
        },
        "FontDescriptor" => fd_id,
        "DW" => 1000,
        // CIDToGIDMap should be a name /Identity
        "CIDToGIDMap" => Object::Name(b"Identity".to_vec()),
        // W array (widths) - supply the compact array we built
        "W" => Object::Array(w_array),
    });

    cidfont_id
}

fn create_font_referencing_descendant(doc: &mut Document, 
    font_name: &str, 
    cidfont_id: (u32, u16), 
    tounicode_id: (u32, u16)
) -> (u32, u16) {
    // Type0 font referencing descendant
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type0",
        "BaseFont" => Object::Name(font_name.as_bytes().to_vec()),
        "Encoding" => Object::Name(b"Identity-H".to_vec()),
        "DescendantFonts" => vec![cidfont_id.into()],
        "ToUnicode" => tounicode_id,
    });

    font_id
}

fn create_font_resources_id(doc: &mut Document, font_id: (u32, u16)) -> (u32, u16) {
    // Resources
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        }
    });

    resources_id
}

fn prepare_pdf_doc(font_data: &[u8], 
    font_name: &str, 
    face: &Face
) -> PdfDocument {
    // Create lopdf Document
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_stream_id = create_font_stream(&mut doc, font_data);

    let fd_id = create_font_descriptor(&mut doc, font_name, face, font_stream_id);

    PdfDocument { 
        doc, 
        pages_id, 
        fd_id
    }
}

fn new_page(font_size_pt: f32) -> Vec<Operation> {
    vec![
        Operation::new("BT", vec![]),
        // Set font F1 and size 12
        Operation::new("Tf", vec!["F1".into(), font_size_pt.into()]),
        Operation::new("Td", vec![A4_DEFAULT_MARGINS.left.into(), 
            a4_default_content_height().into()]), // start position
    ]
}

fn finalise_page(pdf_doc: &mut PdfDocument, 
    operations: &mut Vec<Operation>, 
    content_ids: &mut Vec<(u32, u16)>) 
{
    operations.push(Operation::new("ET", vec![]));

    // Optional: can add BOM at start if you want but not necessary for CID stream
    // Build content stream with hex string for Tj
    let content = Content { operations };

    // content_id
    content_ids.push(
        pdf_doc.doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()))
    );
}

fn prepare_page_content(pdf_doc: &mut PdfDocument, 
    pdf_text: &PdfTextContent
) -> Vec<(u32, u16)> {
    let line_height_pt = pdf_text.font_size_pt() * LINE_SPACING_FACTOR;

    let mut result: Vec<(u32, u16)> = Vec::new();
    let mut current_y = a4_default_content_height();
    
    let mut operations: Vec<Operation> = new_page(*pdf_text.font_size_pt());

    for glyph_bytes in pdf_text.lines_glyph_bytes() {
        if current_y - line_height_pt <= A4_DEFAULT_MARGINS.bottom {
            finalise_page(pdf_doc, &mut operations, &mut result);

            operations = new_page(*pdf_text.font_size_pt());
            current_y = a4_default_content_height();
        } else {
            operations.push(
                Operation::new("Tj", vec![
                    Object::String(glyph_bytes.to_vec(), StringFormat::Hexadecimal)])
            );

            operations.push(
                Operation::new("Td", vec![0.into(), (-line_height_pt).into()]) // move down
            );

            current_y -= line_height_pt;
        }
    }

    // The remaining unflushed page.
    finalise_page(pdf_doc, &mut operations, &mut result);

    result
}

fn prepare_shared_font(pdf_doc: &mut PdfDocument, 
    pdf_text: &PdfTextContent, 
    face: &Face, 
    units_per_em: f32
) -> (u32, u16) {
    let widths_map = get_width_maps(pdf_text.used_cids(), &face, units_per_em);
    let w_array = build_w_array(widths_map);

    let cidfont_id = create_cid_font_type2(&mut pdf_doc.doc, 
        pdf_text.font_info().embedded_name(), pdf_doc.fd_id, w_array);

    let tounicode_id = pdf_doc.doc.add_object(make_to_unicode_cmap(pdf_text, &face));

    let font_id = create_font_referencing_descendant(&mut pdf_doc.doc, 
        pdf_text.font_info().embedded_name(), cidfont_id, tounicode_id);
    
    let resources_id = create_font_resources_id(&mut pdf_doc.doc, font_id);

    resources_id
}

pub fn generate_pdf(pdf_text: &PdfTextContent, pdf_file_name: &str) -> Result<(), String> {
    // Font metric
    let face = Face::parse(&pdf_text.font_subset(), 0).expect("TTF parse");
    // Parse with ttf-parser for glyph indices + metrics
    let units_per_em = face.units_per_em() as f32;

    let mut pdf_doc = prepare_pdf_doc(&pdf_text.font_subset(), 
        pdf_text.font_info().embedded_name(), &face);

    let resources_id = prepare_shared_font(&mut pdf_doc, 
        pdf_text, &face, units_per_em);

    let mut page_ids = vec![];

    let content_ids = prepare_page_content(&mut pdf_doc, pdf_text);
    // Page object
    for content_id in content_ids {
        page_ids.push( pdf_doc.doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pdf_doc.pages_id,
            "Contents" => content_id,
            "MediaBox" => vec![0.into(), 0.into(), A4.width.into(), A4.height.into()],
        }).into() );
    }

    // Pages root
    let pages = dictionary! {
        "Type" => "Pages",
        "Count" => page_ids.len() as u32,
        "Kids" => page_ids,
        "Resources" => resources_id,   // ✅ shared across all pages
    };

    pdf_doc.doc.objects.insert(pdf_doc.pages_id, Object::Dictionary(pages));

    // Catalog
    let catalog_id = pdf_doc.doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pdf_doc.pages_id,
    });
    pdf_doc.doc.trailer.set("Root", catalog_id);

    pdf_doc.doc.compress();
    match pdf_doc.doc.save(pdf_file_name) {
        Ok(_) => {
            println!("PDF document written to {pdf_file_name}");
            return Ok(());
        },
        Err(err) => Err(err.to_string()),
    }
}