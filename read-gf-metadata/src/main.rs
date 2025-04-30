use home::home_dir;
use read_gf_metadata::{iter_families, iter_languages};


fn main() {
    let home = home_dir().expect("Must have a home dir");
    let mut fonts = home.clone();
    fonts.push("oss/fonts");

    let mut metadata_success = 0;
    let mut metadata_fail = 0;
    let mut lang_success = 0;
    let mut lang_fail = 0;

    for entry in iter_families(&fonts) {
        if let Err(e) = entry {
            eprintln!("Family read error {e:?}");
            metadata_fail += 1;
            continue;
        }
        metadata_success += 1;        
    }

    for entry in iter_languages(&fonts) {
        if let Err(e) = entry {
            eprintln!("Language pread error {e:?}");
            lang_fail += 1;
            continue;
        }
        lang_success += 1;
    }

    eprintln!(
        "Read {}/{} METADATA.pb files successfully",
        metadata_success,
        metadata_success + metadata_fail
    );
    eprintln!(
        "Read {}/{} language files successfully",
        lang_success,
        lang_success + lang_fail
    );
}
