use gf_metadata::{GoogleFonts, exemplar};
use home::home_dir;

fn main() {
    let mut d = home_dir().expect("Must have a home dir");
    d.push("oss/fonts");
    //d.push("ofl/robotomono");
    let gf = GoogleFonts::new(d);

    let mut metadata_success = 0;
    let mut metadata_fail = 0;
    let mut lang_success = 0;
    let mut lang_fail = 0;

    for (path, entry) in gf.families() {
        let Ok(family) = entry else {
            eprintln!("Family read error {entry:?} at {path:?}");
            metadata_fail += 1;
            continue;
        };
        let Some(exemplar) = exemplar(&family) else {
            eprintln!("No exemplar for {} from {path:?}", family.name());
            metadata_fail += 1;
            continue;
        };
        if gf.find_font_binary(&exemplar).is_none() {
            eprintln!("No font binary for {exemplar:?} from {path:?}");
            metadata_fail += 1;
            continue;
        }
        metadata_success += 1;
    }

    for entry in gf.languages() {
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
