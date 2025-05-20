use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use color::parse_color;
use home::home_dir;
use kurbo::{Affine, BezPath, Vec2};
use make_test_images::draw_png;
use regex::Regex;
use skrifa::FontRef;
use sleipnir::{
    icon2svg::{DrawOptions, draw_icon},
    iconid::{IconIdentifier, Icons},
    pathstyle::SvgPathStyle,
};
use tiny_skia::Pixmap;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Where to save svg files
    #[arg(short, long, default_value = "/tmp/icon_svg")]
    svg_dir: String,

    /// Where to save svg files
    #[arg(short, long, default_value = "/tmp/icon_png")]
    png_dir: String,

    /// Text color
    #[arg(short, long, default_value = "black")]
    text_color: String,

    /// Backdrop color
    #[arg(short, long, default_value = "white")]
    backdrop_color: String,

    /// The Google-style icon font to process
    #[arg(long)]
    icon_font: String,
}

fn output_file(dir: &str, icon: IconIdentifier, ext: &str) -> PathBuf {
    let mut out_file = PathBuf::from(dir);
    out_file.push(format!(
        "{}{ext}",
        match icon {
            IconIdentifier::GlyphId(gid) => format!("gid{gid}"),
            IconIdentifier::Codepoint(cp) => format!("0x{cp:04x}"),
            IconIdentifier::Name(name) => format!("name{name}"),
        }
    ));
    out_file
}

fn ensure_has_dir(dir: &str) {
    let p = Path::new(dir);
    fs::create_dir_all(p).expect("To create output dir");
}

fn main() {
    let args = Args::parse();

    let text_color = parse_color(&args.text_color).unwrap();
    let backdrop_color = parse_color(&args.backdrop_color).unwrap();

    ensure_has_dir(&args.svg_dir);
    ensure_has_dir(&args.png_dir);

    let icon_font_path = if args.icon_font.starts_with("~") {
        let mut d = home_dir().expect("Must have a home dir");
        d.push(&args.icon_font[1..]);
        d
    } else {
        PathBuf::from(&args.icon_font)
    };
    let raw_icon_font = fs::read(&icon_font_path)
        .unwrap_or_else(|e| panic!("Unable to read {icon_font_path:?}: {e}"));
    let font = FontRef::new(&raw_icon_font).expect("To parse icon font");

    let viewbox_re = Regex::new("viewBox=\"(-?\\d+) (-?\\d+) (-?\\d+) ").unwrap();
    let path_re = Regex::new("<path d=\"([^\"]+)\"").unwrap();

    for icon in font.icons().expect("Icons") {
        let id = if let Some(name) = icon.names.first() {
            IconIdentifier::Name(name.into())
        } else {
            IconIdentifier::GlyphId(icon.gid)
        };
        let draw_opts =
            DrawOptions::new(id.clone(), 128.0, Default::default(), SvgPathStyle::Compact);
        let icon_svg = draw_icon(&font, &draw_opts).expect("To draw {id:?}");

        // Grab the path and shuffle things around so they line up with the desired image
        let icon_path = path_re
            .captures_iter(&icon_svg)
            .map(|c| c.get(1).unwrap().as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let mut icon_path = BezPath::from_svg(&icon_path).unwrap();
        let (viewbox_x, viewbox_y, viewbox_dim) = viewbox_re
            .captures_iter(&icon_svg)
            .map(|c| {
                (
                    c.get(1).unwrap().as_str().parse::<f64>().unwrap(),
                    c.get(2).unwrap().as_str().parse::<f64>().unwrap(),
                    c.get(3).unwrap().as_str().parse::<f64>().unwrap(),
                )
            })
            .next()
            .unwrap();

        // move the viewed area to start at 0,0
        // then scale to 128x128
        let transform = Affine::translate(Vec2 {
            x: -viewbox_x,
            y: -viewbox_y,
        })
        .then_scale(128.0 / viewbox_dim);
        icon_path.apply_affine(transform);

        let mut pixmap =
            Pixmap::new(128, 128).unwrap_or_else(|| panic!("Failed to allocate pixmap for {id:?}"));
        let png =
            draw_png(&mut pixmap, text_color, backdrop_color, icon_path).expect("To draw png");
        let png_out = output_file(&args.png_dir, id, ".png");
        fs::write(&png_out, png).expect("To write output files");
        eprintln!("Wrote {png_out:?}");
    }
}
