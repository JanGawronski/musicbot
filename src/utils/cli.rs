use clap::Parser;
use std::path::PathBuf;
use serenity::prelude::TypeMapKey;

#[derive(Parser, Clone)]
#[command(version, about)]
pub struct Config {
    #[arg(
        short, 
        long, 
        value_name = "EXECUTABLE_PATH", 
        help = "Path to the yt-dlp executable",
        value_parser = validate_executable_path,
    )]
    pub yt_dlp: PathBuf,

    #[arg(
        short,
        long = "local_audio",
        value_name = "DIRECTORY_PATH",
        help = "Directory containing local audio files",
        value_parser = validate_directory_path,
    )]
    pub audio_directory: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_name = "FILE_PATH",
        help = "Path to cookies file for yt-dlp authentication",
        value_parser = validate_file_path,
    )]
    pub cookies: Option<String>,
}

impl TypeMapKey for Config {
    type Value = Config;
}

fn validate_executable_path(path: &str) -> Result<PathBuf, String> {
    let pb = PathBuf::from(path);
    if !pb.exists() {
        Err(format!("'{}' does not exist", path))
    } else if !pb.is_file() {
        Err(format!("'{}' is not a file", path))
    } else {
        Ok(pb)
    }
}

fn validate_directory_path(path: &str) -> Result<PathBuf, String> {
    let pb = PathBuf::from(path);

    if !pb.exists() {
        Err(format!("'{}' does not exist", path))
    } else if !pb.is_dir() {
        Err(format!("'{}' is not a directory", path))
    } else {
        Ok(pb)
    }
}

fn validate_file_path(path: &str) -> Result<String, String> {
    let pb = PathBuf::from(path);

    if !pb.exists() {
        Err(format!("'{}' does not exist", path))
    } else if !pb.is_file() {
        Err(format!("'{}' is not a file", path))
    } else {
        Ok(pb.to_string_lossy().to_string())
    }
}