use std::fs::File;

use gf_metadata::{FontProto, GoogleFonts};
use harfruzz::{GlyphBuffer, ShaperFont};
use kurbo::{Affine, BezPath, Point, Vec2};
use memmap::{Mmap, MmapOptions};
use skrifa::{
    MetadataProvider,
    outline::{DrawSettings, OutlinePen},
    prelude::{LocationRef, Size},
};

/// Draws sample text in the specified font.
///
/// Baseline is at y=0.
pub fn path_for_sampletext(gf: &GoogleFonts, font: &FontProto) -> BezPath {
    // Figure out what string to draw
    let Some((_, family)) = gf.family(font) else {
        panic!("No family available for {font:?}?!");
    };

    let lang = gf.primary_language(family);
    let sample_text = lang.sample_text.styles();

    // Load the font and shape the sample string
    let Some(font_file) = gf.find_font_binary(font) else {
        panic!("Unable to locate {font:?}");
    };

    let fd = File::open(&font_file).expect("To read fonts!");
    let mmap: Mmap = unsafe { MmapOptions::new().map(&fd).expect("To map files!") };
    let harf_font_ref = harfruzz::FontRef::new(&mmap).expect("For font files to be font files!");
    let skrifa_font_ref = skrifa::FontRef::new(&mmap).expect("Fonts to be fonts");

    // Draw an SVG of it
    let outlines = skrifa_font_ref.outline_glyphs();
    let mut pen = PathPen::default();

    let glyphs = shape(&sample_text, &harf_font_ref);

    for (glyph_info, pos) in glyphs.glyph_infos().iter().zip(glyphs.glyph_positions()) {
        let glyph = outlines
            .get(glyph_info.glyph_id.into())
            .expect("Glyphs to exist!");
        glyph
            .draw(
                DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                &mut pen,
            )
            .expect("To draw!");

        pen.transform = pen.transform.then_translate(Vec2 {
            x: pos.x_advance.into(),
            y: pos.y_advance.into(),
        });
    }

    pen.path
}

// Simplified version of <https://github.com/harfbuzz/harfruzz/blob/006472176ab87e3a84e799e74e0ac19fbe943dd7/tests/shaping/main.rs#L107>
// Will have to update if/when that API updates
fn shape(text: &str, font: &harfruzz::FontRef) -> GlyphBuffer {
    let shaper_font = ShaperFont::new(font);
    let face = shaper_font.shaper(font, &[]);

    let mut buffer = harfruzz::UnicodeBuffer::new();
    buffer.push_str(text);

    harfruzz::shape(&face, &[], buffer)
}

struct PathPen {
    transform: Affine,
    path: BezPath,
}

impl Default for PathPen {
    fn default() -> Self {
        // flip y because fonts are y-up and svg is y-down
        Self {
            transform: Affine::FLIP_Y,
            path: Default::default(),
        }
    }
}

impl OutlinePen for PathPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(
            self.transform
                * Point {
                    x: x.into(),
                    y: y.into(),
                },
        );
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(
            self.transform
                * Point {
                    x: x.into(),
                    y: y.into(),
                },
        );
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.path.quad_to(
            self.transform
                * Point {
                    x: cx0.into(),
                    y: cy0.into(),
                },
            self.transform
                * Point {
                    x: x.into(),
                    y: y.into(),
                },
        );
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.path.curve_to(
            self.transform
                * Point {
                    x: cx0.into(),
                    y: cy0.into(),
                },
            self.transform
                * Point {
                    x: cx1.into(),
                    y: cy1.into(),
                },
            self.transform
                * Point {
                    x: x.into(),
                    y: y.into(),
                },
        );
    }

    fn close(&mut self) {
        self.path.close_path();
    }
}
