use std::{collections::HashMap, fs, path::Path};

use clap::Parser;
use embed_anything::embeddings::{
    embed::{EmbedImage, Embedder, VisionEmbedder},
    local::clip::ClipEmbedder,
};
use gf_embed::embed_data::EmbedDataProto;
use protobuf::Message;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Where to load png files from
    #[arg(short, long, default_value = "/tmp/icon_png")]
    image_dir: String,

    /// Where to write embedding data to
    #[arg(short, long, default_value = "/tmp/icon_data")]
    embed_dir: String,
}

fn main() {
    let args = Args::parse();

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

    eprintln!(
        "Found {} icon images, creating embeddings...",
        image_files.len(),
    );

    // Create a local CLIP embedder from a Hugging Face model
    let embedder = Embedder::Vision(VisionEmbedder::Clip(ClipEmbedder::default()));

    for image_file in image_files {
        let image_file_name = image_file.file_stem().unwrap().to_str().unwrap();
        let mut out = embed_dir.to_path_buf();
        out.push(&image_file_name);
        out.set_extension("pb");
        let metadata = HashMap::from([("name".to_string(), image_file_name.to_string())]);
        match embedder.embed_image(&image_file, Some(metadata)) {
            Ok(data) => {
                let proto: EmbedDataProto = data.into();
                let raw = proto.write_to_bytes().expect("To convert proto to bytes");
                fs::write(&out, &raw).expect("To write output file");

                println!("Embedded {} and wrote {out:?}", image_file_name,);
            }
            Err(e) => eprintln!("Failed to embed {}: {e:?}", image_file_name),
        }
    }
}
