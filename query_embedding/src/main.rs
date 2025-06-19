use std::{fs::File, io::BufReader, path::Path};

use candle_core::{Device, Tensor};
use clap::Parser;
use embed_anything::{
    embed_query,
    embeddings::{
        embed::{EmbedData, EmbedImage, Embedder, VisionEmbedder},
        local::clip::ClipEmbedder,
    },
};
use gf_embed::embed_data::EmbedDataProto;
use protobuf::Message;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Where to read embedding data from
    #[arg(short, long, default_value = "/tmp/test_data")]
    embed_dir: String,

    #[arg(trailing_var_arg = true)]
    queries: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let embed_dir = Path::new(&args.embed_dir);
    if !embed_dir.is_dir() {
        panic!("No such dir {embed_dir:?}");
    }

    // Load precomputed embed data
    let embed_datas = WalkDir::new(embed_dir)
        .into_iter()
        .filter_map(|e| match e {
            Ok(d) => d
                .path()
                .to_string_lossy()
                .ends_with(".pb")
                .then(|| d.path().to_path_buf()),
            Err(e) => panic!("Unable to read from {embed_dir:?}: {e}"),
        })
        .map(|p| {
            let fd = File::open(p).expect("File access");
            let mut rdr = BufReader::new(fd);
            EmbedDataProto::parse_from_reader(&mut rdr)
                .expect("To load protos")
                .into()
        })
        .collect::<Vec<EmbedData>>();

    // Create a local CLIP embedder from a Hugging Face model
    let embedder = Embedder::Vision(VisionEmbedder::Clip(ClipEmbedder::default()));

    // Ref https://github.com/StarlightSearch/EmbedAnything/blob/main/rust/examples/clip.rs
    for query in args.queries.iter() {
        let query_embed_data = if query.ends_with(".png") {
            let p = Path::new(query);
            if !p.is_file() {
                eprintln!("{query} looks like an image file but doesn't exist, skipping");
                continue;
            }
            vec![embedder.embed_image(&p, None).expect("To embed image")]
        } else {
            embed_query(&[query.as_str()], &embedder, None)
                .await
                .expect("Query to execute")
        };

        // TODO we should save something closer to what we need here in make_embedding
        let n_vectors = embed_datas.len();
        let vector = embed_datas
            .iter()
            .map(|embed| embed.embedding.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|x| x.to_dense().unwrap())
            .collect::<Vec<_>>();

        let out_embeddings = Tensor::from_vec(
            vector,
            (
                n_vectors,
                embed_datas[0].embedding.to_dense().unwrap().len(),
            ),
            &Device::Cpu,
        )
        .unwrap();

        let query_embeddings = Tensor::from_vec(
            query_embed_data
                .iter()
                .map(|embed| embed.embedding.clone())
                .collect::<Vec<_>>()
                .into_iter()
                .flat_map(|x| x.to_dense().unwrap())
                .collect::<Vec<_>>(),
            (1, query_embed_data[0].embedding.to_dense().unwrap().len()),
            &Device::Cpu,
        )
        .unwrap();

        let similarities = out_embeddings
            .matmul(&query_embeddings.transpose(0, 1).unwrap())
            .unwrap()
            .detach()
            .squeeze(1)
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let mut indices: Vec<usize> = (0..similarities.len()).collect();
        indices.sort_by(|a, b| similarities[*b].partial_cmp(&similarities[*a]).unwrap());

        let top_matches = indices[0..5].to_vec();
        let top_names = top_matches
            .iter()
            .map(|i| {
                let metadata = embed_datas[*i].metadata.as_ref().unwrap();
                // Really should have used the same id field...
                metadata
                    .get("name")
                    .or_else(|| metadata.get("family_name"))
                    .unwrap_or_else(|| panic!("Should have some identifier in {metadata:?}"))
            })
            .collect::<Vec<_>>();

        println!("Best results for {query}");
        for (i, e) in top_names.iter().enumerate() {
            println!("{i}: {e}");
        }
    }
}
