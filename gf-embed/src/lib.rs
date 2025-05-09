use embed_anything::embeddings::embed::{EmbedData, EmbeddingResult};
use embed_data::{EmbedDataProto, FloatVecProto, MetadataProto};

pub mod embed_data;

impl From<(String, String)> for MetadataProto {
    fn from(value: (String, String)) -> Self {
        let mut proto = MetadataProto::new();
        proto.set_key(value.0);
        proto.set_value(value.1);
        proto
    }
}

impl From<Vec<f32>> for FloatVecProto {
    fn from(value: Vec<f32>) -> Self {
        let mut proto = FloatVecProto::new();
        for v in value {
            proto.value.push(v);
        }
        proto
    }
}

impl From<EmbedData> for EmbedDataProto {
    fn from(mut value: EmbedData) -> Self {
        let mut proto = EmbedDataProto::new();
        if let Some(text) = value.text.take() {
            proto.set_text(text);
        }
        if let Some(metadata) = value.metadata.take() {
            proto.metadata = metadata.into_iter().map(|kv| kv.into()).collect();
        }
        match value.embedding {
            EmbeddingResult::DenseVector(v) => proto.result.push(v.into()),
            EmbeddingResult::MultiVector(vv) => {
                for v in vv {
                    proto.result.push(v.into());
                }
            }
        }
        proto
    }
}

impl Into<EmbedData> for EmbedDataProto {
    fn into(mut self) -> EmbedData {
        let mut result = std::mem::take(&mut self.result);
        EmbedData {
            embedding: match result.len() {
                0 => panic!("This isn't supposed to happen!"),
                1 => EmbeddingResult::DenseVector(result.pop().unwrap().value),
                _ => EmbeddingResult::MultiVector(result.into_iter().map(|v| v.value).collect()),
            },
            text: if self.has_text() {
                Some(self.take_text())
            } else {
                None
            },
            metadata: if !self.metadata.is_empty() {
                Some(
                    self.metadata
                        .into_iter()
                        .map(|mut m| (m.take_key(), m.take_value()))
                        .collect(),
                )
            } else {
                None
            },
        }
    }
}
