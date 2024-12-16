use std::{collections::HashMap, io::Read};

use eyre::Result;
use sha1::{Digest, Sha1};

#[derive(Debug, Clone)]
pub struct AssetPaths {
    pub lookup: HashMap<String, AssetPath>,
    pub paths: mmo_common::client_config::AssetPaths,
}

#[derive(Debug, Clone)]
pub struct AssetPath {
    pub request_filename: String,
    pub request_path: String,
    pub local_path: String,
}

struct LocalAssetPaths {
    tileset: &'static str,
    charset: &'static str,
    font: &'static str,
    font_meta: &'static str,
}

const ASSETS: LocalAssetPaths = LocalAssetPaths {
    tileset: "tileset.png",
    charset: "charset.png",
    font: "notosans.png",
    font_meta: "notosans.json",
};

pub fn load_assets() -> Result<AssetPaths> {
    let tileset = load_asset(ASSETS.tileset)?;
    let charset = load_asset(ASSETS.charset)?;
    let font = load_asset(ASSETS.font)?;
    let font_meta = load_asset(ASSETS.font_meta)?;

    let paths = mmo_common::client_config::AssetPaths {
        tileset: tileset.request_path.clone(),
        charset: charset.request_path.clone(),
        font: font.request_path.clone(),
        font_meta: font_meta.request_path.clone(),
    };

    let lookup = [
        (tileset.request_filename.clone(), tileset),
        (charset.request_filename.clone(), charset),
        (font.request_filename.clone(), font),
        (font_meta.request_filename.clone(), font_meta),
    ]
    .into();

    Ok(AssetPaths { lookup, paths })
}

fn load_asset(local_filename: &str) -> Result<AssetPath> {
    let local_path = format!("assets/{}", local_filename);
    let hash = hash_file_content(&local_path)?;
    let request_filename = generate_filename(local_filename, &hash);
    let request_path = format!("/assets/{}", request_filename);

    Ok(AssetPath { request_filename, request_path, local_path })
}

fn hash_file_content(path: &str) -> Result<String> {
    let mut hasher = Sha1::new();
    {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = [0; 4096];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
    }
    let hash_bytes = hasher.finalize();

    let mut hash = String::new();
    for byte in hash_bytes.iter() {
        hash.push_str(&format!("{:02x}", byte));
    }

    Ok(hash)
}

fn generate_filename(local_filename: &str, hash: &str) -> String {
    if let Some((before_ext, ext)) = local_filename.split_once('.') {
        format!("{}.{}.{}", before_ext, hash, ext)
    } else {
        format!("{}.{}", local_filename, hash)
    }
}
