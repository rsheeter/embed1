use std::{collections::HashMap, fs, path::Path};

use clap::Parser;
use embed_anything::embeddings::{
    embed::{EmbedImage, Embedder, VisionEmbedder},
    local::clip::ClipEmbedder,
};
use gf_metadata::{GoogleFonts, exemplar};
use home::home_dir;
use itertools::Itertools;
use make_embedding::embed_data::EmbedDataProto;
use protobuf::Message;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Where to load png files from
    #[arg(short, long, default_value = "/tmp/test_png")]
    image_dir: String,

    /// Family path filter, retain only paths that contain this regex.
    #[arg(long)]
    family_filter: Option<String>,

    /// Where to write embedding data to
    #[arg(short, long, default_value = "/tmp/test_data")]
    embed_dir: String,
}

fn main() {
    let args = Args::parse();

    let family_filter = args
        .family_filter
        .map(|f| Regex::new(&f).expect("A valid filter regex"));

    let image_dir = Path::new(&args.image_dir);
    if !image_dir.is_dir() {
        panic!("Input dir {} doesn't exist", args.image_dir);
    }
    let image_files = WalkDir::new(image_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".png"))
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();

    if image_files.is_empty() {
        panic!("No images, this will be dull");
    }

    let embed_dir = Path::new(&args.embed_dir);
    if !embed_dir.is_dir() {
        fs::create_dir_all(embed_dir).expect("To create output dir");
    }
    println!("Clearing {:?}", args.embed_dir);
    for entry in WalkDir::new(&embed_dir).into_iter() {
        let entry = entry.expect("To walk output dir");
        if !entry.file_name().to_string_lossy().ends_with(".pb") {
            continue;
        }
        fs::remove_file(entry.path()).expect("To delete files");
    }

    let mut d = home_dir().expect("Must have a home dir");
    d.push("oss/fonts");
    let gf = GoogleFonts::new(d, family_filter);

    let mut tags = gf.tags().expect("Tags").iter().collect::<Vec<_>>();
    tags.sort_by_key(|t| (&t.family, &t.tag, (100.0 * t.value) as i32));
    let mut tags_by_family = HashMap::new();
    for (key, chunk) in &tags.iter().chunk_by(|t| &t.family) {
        tags_by_family.insert(key.as_str(), chunk.collect::<Vec<_>>());
    }

    eprintln!(
        "Found {} tagged families, {} tags total, creating embeddings...",
        tags_by_family.len(),
        tags.len()
    );

    // Create a local CLIP embedder from a Hugging Face model
    let embedder = Embedder::Vision(VisionEmbedder::Clip(ClipEmbedder::default()));

    for (_, family) in gf.families().iter().filter_map(|e| match &e {
        (p, Ok(f)) => Some((p, f)),
        (_, Err(..)) => None,
    }) {
        let mut out = embed_dir.to_path_buf();
        out.push(format!("{}.pb", family.name().replace(" ", "_")));

        let Some(exemplar) = exemplar(family) else {
            eprintln!("Unable to identify an exemplar for {}", family.name());
            continue;
        };
        let tags = tags_by_family.get(family.name());
        let mut image_file = image_dir.to_path_buf();
        image_file.push(exemplar.filename().to_string() + ".png");
        if !image_file.is_file() {
            eprintln!("Missing {image_file:?}");
            continue;
        }
        let metadata = tags.map(|tags| tags).map(|tags| {
            tags.iter()
                .map(|t| (t.tag.to_string(), format!("{}", t.value)))
                .collect()
        });
        match embedder.embed_image(&image_file, metadata) {
            Ok(data) => {
                let proto: EmbedDataProto = data.into();
                let raw = proto.write_to_bytes().expect("To convert proto to bytes");
                fs::write(&out, &raw).expect("To write output file");

                println!(
                    "Embedded {} with {} tags and wrote {out:?}",
                    family.name(),
                    tags.map(|tags| tags.len()).unwrap_or_default()
                );
            }
            Err(e) => eprintln!("Failed to embed {}: {e:?}", family.name()),
        }
    }
}
