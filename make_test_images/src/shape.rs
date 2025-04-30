//! Simple wrapper around shaping

use harfruzz::{FontRef, GlyphBuffer, ShaperFont};

// Simplified version of <https://github.com/harfbuzz/harfruzz/blob/006472176ab87e3a84e799e74e0ac19fbe943dd7/tests/shaping/main.rs#L107>
// Will have to update if/when that API updates
pub fn shape(text: &str, font: &FontRef) -> GlyphBuffer {

    let shaper_font = ShaperFont::new(&font);
    let face = shaper_font.shaper(&font, &[]);

    let mut buffer = harfruzz::UnicodeBuffer::new();
    buffer.push_str(text);
        
    harfruzz::shape(&face, &[], buffer)
}