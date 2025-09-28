use std::{
    fs::read_dir,
    collections::HashMap,
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
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    let metadata = Metadata {
                        title: Some(filename.to_string()),
                        uploader: None,
                        track: None,
                        artist: None,
                        duration: None,
                        thumbnail: None,
                        webpage_url: None,
                        url: Some(path.to_string_lossy().to_string()),

                    };
                    return Some((filename.to_string(), metadata));
                }
            } 
        }
        None
    }).collect()
} 