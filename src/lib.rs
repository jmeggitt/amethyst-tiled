use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Mutex;

use amethyst::assets::{AssetStorage, Handle, Loader, ProgressCounter};
use amethyst::renderer::rendy::texture::image::{load_from_image, ImageTextureConfig};
use amethyst::renderer::types::TextureData;
use amethyst::renderer::{SpriteSheet, Texture};
use failure::Error;
use tiled::{parse_tileset, Tileset};

pub mod format;
pub mod loader;
pub mod packing;
pub mod prefab;
pub mod system;

fn load_tileset_inner(
    tileset: &Tileset,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
) -> Result<SpriteSheet, Error> {
    let packed = packing::pack_tileset(tileset)?;
    let reader = BufReader::new(Cursor::new(&packed.bytes));

    let texture_builder = load_from_image(reader, ImageTextureConfig::default())?;

    let sheet = SpriteSheet {
        texture: loader.load_from_data(TextureData(texture_builder), progress, storage),
        sprites: packing::extract_sprite_vec(&packed),
    };

    Ok(sheet)
}

pub fn load_tileset<P: AsRef<Path>>(
    path: P,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
) -> Result<SpriteSheet, Error> {
    let tileset = parse_tileset(File::open(path)?, 1)?;

    load_tileset_inner(&tileset, loader, progress, storage)
}

pub fn load_cached_tileset<P: AsRef<Path>>(
    path: P,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
    sprite_sheets: &mut AssetStorage<SpriteSheet>,
    tilesets: &Tilesets,
) -> Result<Handle<SpriteSheet>, Error> {
    let tileset = parse_tileset(File::open(path)?, 1)?;

    match tilesets.get(&tileset.name) {
        Some(handle) => Ok(handle),
        None => {
            let sheet = load_tileset_inner(&tileset, loader, progress, storage)?;
            let handle = sprite_sheets.insert(sheet);
            tilesets.push(tileset.name.to_owned(), handle.clone());

            Ok(handle)
        }
    }
}

#[derive(Default)]
pub struct Tilesets(Mutex<HashMap<String, Handle<SpriteSheet>>>);

impl Tilesets {
    pub fn push(&self, set_name: String, handle: Handle<SpriteSheet>) {
        self.0.lock().unwrap().insert(set_name, handle);
    }

    pub fn get(&self, set_name: &str) -> Option<Handle<SpriteSheet>> {
        self.0.lock().unwrap().get(set_name).cloned()
    }
}
