use std::{fs, path::PathBuf};

use gf_metadata::{GoogleFonts, exemplar};
use home::home_dir;
use kurbo::Shape;
use make_test_images::draw::draw_sample_svg;

fn main() {
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
        let sample = draw_sample_svg(&gf, exemplar);
        let bbox = sample.bounding_box();
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"{} {} {} {}\">\n",
            bbox.min_x(),
            bbox.min_y(),
            bbox.width(),
            bbox.height()
        );
        svg.push_str("  <path d=\"");
        svg.push_str(&sample.to_svg());
        svg.push_str("\" />\n");
        svg += "</svg>";

        let mut out_file = PathBuf::from("/tmp");
        out_file.push(format!("{}.svg", exemplar.filename()));
        fs::write(&out_file, svg).expect("To write output files");
        eprintln!("Wrote {out_file:?}");
    }
}
