//! Helpers for accessing Google Fonts metadata

use std::{fs, path::Path};

use gf_metadata::{read_family, read_language, FamilyProto, LanguageProto};
use protobuf::text_format::ParseError;
use walkdir::WalkDir;

pub fn iter_families(p: &Path) -> impl Iterator<Item=Result<FamilyProto, ParseError>> {
    WalkDir::new(p).into_iter()
        .filter_map(|d| d.ok())
        .filter(|d| d.file_name() == "METADATA.pb")
        .map(|d| {
            
            read_family(&fs::read_to_string(d.path()).expect("To read files!"))
        })
}

pub fn iter_languages(p: &Path) -> impl Iterator<Item=Result<LanguageProto, ParseError>> {
    WalkDir::new(p).into_iter()
        .filter_map(|d| d.ok())
        .filter(|d| d.path()
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .contains("gflanguages/data/languages")
        && d.file_name().to_string_lossy().ends_with(".textproto"))
        .map(|d| {
            
            read_language(&fs::read_to_string(d.path()).expect("To read files!"))
        })
}