use std::{fs, path::PathBuf};

use color::parse_color;
use gf_metadata::{FontProto, GoogleFonts, exemplar};
use home::home_dir;
use kurbo::{Affine, BezPath, Rect, Shape, Vec2};
use make_test_images::{draw::path_for_sampletext, draw_png};
use tiny_skia::Pixmap;

fn svg(sample: &BezPath, viewbox: Rect) -> String {
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"{} {} {} {}\">\n",
        viewbox.min_x(),
        viewbox.min_y(),
        viewbox.width(),
        viewbox.height()
    );
    svg.push_str("  <path d=\"");
    svg.push_str(&sample.to_svg());
    svg.push_str("\" />\n");
    svg += "</svg>";
    svg
}

fn output_file(exemplar: &FontProto, ext: &str) -> PathBuf {
    let mut out_file = PathBuf::from("/tmp");
    out_file.push(format!("{}{ext}", exemplar.filename()));
    out_file
}

fn with_margin(rect: Rect, multiplier: f64) -> Rect {
    let margin = rect.width().min(rect.height()) * multiplier;
    rect.inflate(margin, margin)
}

fn main() {
    let text_color = parse_color("black").unwrap();
    let backdrop_color = parse_color("white").unwrap();

    let mut d = home_dir().expect("Must have a home dir");
    d.push("oss/fonts");
    let gf = GoogleFonts::new(d);

    let mut metadatas = Vec::new();
    let mut metadata_fail = 0;

    for (path, entry) in gf.families() {
        match entry {
            Ok(m) => metadatas.push(m),
            Err(e) => {
                eprintln!("Family read error {e:?} at {path:?}");
                metadata_fail += 1;
            }
        }
    }

    eprintln!(
        "Read {}/{} METADATA.pb files successfully",
        metadatas.len(),
        metadatas.len() + metadata_fail
    );

    for metadata in &metadatas {
        let Some(exemplar) = exemplar(metadata) else {
            eprintln!("Unable to identify an exemplar for {}", metadata.name());
            continue;
        };
        let path = path_for_sampletext(&gf, exemplar);

        // Add a 3% of smallest dimension as margin
        let sample_bbox = with_margin(path.bounding_box(), 0.03);

        if sample_bbox.area() == 0.0 {
            eprintln!("Nothing drawn (area 0) for {}", metadata.name());
            continue;
        }

        // Draw an svg
        let svg = svg(&path, sample_bbox);
        let svg_out = output_file(exemplar, ".svg");
        fs::write(&svg_out, svg).expect("To write output files");
        eprintln!("Wrote {svg_out:?}");

        // Draw a png normalized to fit within 128 vertical pixels
        let mut scaled_path = path.clone();
        // Move the bbox so minx/y are both 0 and scale so height is 128
        // TODO: this ignores font choice of vertical height, perhaps we should scale everything the same
        // e.g. apply the scale that makes the largest height fit
        let transform = Affine::translate(Vec2 {
            x: -sample_bbox.min_x(),
            y: -sample_bbox.min_y(),
        })
        .then_scale(128.0 / path.bounding_box().height());
        scaled_path.apply_affine(transform);
        let scaled_bbox = with_margin(scaled_path.bounding_box(), 0.03);
        let mut pixmap = Pixmap::new(
            scaled_bbox.width().ceil() as u32,
            scaled_bbox.height().ceil() as u32,
        )
        .unwrap_or_else(|| {
            panic!(
                "Failed to allocate {scaled_bbox:?} pixmap for {}",
                metadata.name()
            )
        });
        let png =
            draw_png(&mut pixmap, text_color, backdrop_color, scaled_path).expect("To draw png");
        let png_out = output_file(exemplar, ".png");
        fs::write(&png_out, png).expect("To write output files");
        eprintln!("Wrote {png_out:?}");
    }
}
