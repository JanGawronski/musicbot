use std::{
    fs::read_dir,
    path::PathBuf,
    collections::HashMap,
};

use lofty::{
    probe::Probe,
    tag::Accessor,
    file::{
        TaggedFileExt,
        TaggedFile,
        AudioFile,
    },
};

use super::audio::Metadata;

pub fn get_audio_files() -> HashMap<String, Metadata> {
    let paths = match read_dir("audio/") {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("Failed to read audio directory: {err:?}. If you want to use local files, create an 'audio' directory and add some audio files to it.");
            return HashMap::new();
        }
    };

    paths.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().and_then(|n| n.to_str())?.to_string();
            let tagged_file = get_tagged_file(&path);
            let tag = tagged_file.as_ref().and_then(|f| f.primary_tag())
                .or(tagged_file.as_ref().and_then(|f| f.first_tag()));

            let title = tag.and_then(|t| t.title().map(|s| s.to_string()));
            let artist = tag.and_then(|t| t.artist().map(|s| s.to_string()));

            let duration = tagged_file.as_ref()
                .and_then(|f| f.properties().duration().as_secs().try_into().ok());

            Some((title.clone().unwrap_or(filename.clone()), Metadata {
                title: Some(filename),
                uploader: None,
                track: title,
                artist: artist,
                duration: duration,
                thumbnail: None,
                webpage_url: None,
                url: path.to_str().map(|s| s.to_string()),
                }))
        } else {
            None
        }
    }).collect()
}

fn get_tagged_file(path: &PathBuf) -> Option<TaggedFile> {
    Probe::open(path)
        .ok()?
        .guess_file_type()
        .ok()?
        .read()
        .ok()
}