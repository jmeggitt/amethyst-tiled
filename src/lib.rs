use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Mutex;

use amethyst::assets::AssetStorage;
use amethyst::assets::Handle;
use amethyst::assets::Loader;
use amethyst::assets::PrefabData;
use amethyst::assets::ProgressCounter;
use amethyst::ecs::{Read, World, Write, WriteExpect};
use amethyst::renderer::{SpriteSheet, Texture, TexturePrefab};
use amethyst::renderer::rendy::texture::image::{ImageTextureConfig, load_from_image};
use amethyst::renderer::sprite::prefab::SpriteSheetPrefab;
use amethyst::renderer::types::TextureData;
use failure::Error;
use tiled::{parse_file, parse_tileset, Tileset};
use std::collections::HashMap;

pub mod packing;

fn load_tileset_inner(
    tileset: &Tileset,
    loader: Loader,
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
    loader: Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
) -> Result<SpriteSheet, Error> {
    let tileset = parse_tileset(File::open(path)?, 1)?;

    load_tileset_inner(&tileset, loader, progress, storage)
}

pub fn load_cached_tileset<P: AsRef<Path>>(
    path: P,
    loader: Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
    sprite_sheets: &mut AssetStorage<SpriteSheet>,
    tilesets: &TileSets,
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

pub struct TileSets(Mutex<HashMap<String, Handle<SpriteSheet>>>);

impl TileSets {

    pub fn push(&self, set_name: String, handle: Handle<SpriteSheet>) {
        self.0.lock().unwrap().insert(set_name, handle);
    }

    pub fn get(&self, set_name: &String) -> Option<Handle<SpriteSheet>> {
        self.0.lock().unwrap().get(set_name).map(|x| x.clone())
    }

}


//pub fn register_sprite_sheet(texture: Handle<Texture>, sprites: Vec<Sprite>, name: String) -> Handle<SpriteSheet> {
//    let mut sheet = SpriteSheetPrefab::Sheet {
//        texture: TexturePrefab::Handle(texture),
//        sprites,
//        name: Some(name)
//    };
//
//    unimplemented!()
//}

//pub fn load_map<P: AsRef<Path>>(
//    path: P,
//    loader: Loader,
//    progress: &mut ProgressCounter,
//    storage: &AssetStorage<Texture>,
//) -> Result<SpriteSheet, Error> {
//    let tile_map = parse_file(path.as_ref())?;
//
//    let packed = packing::pack_tileset(&tileset)?;
//    let reader = BufReader::new(Cursor::new(&packed.bytes));
//
//    let texture_builder = load_from_image(reader, ImageTextureConfig::default())?;
//
//    let sheet = SpriteSheet {
//        texture: loader.load_from_data(TextureData(texture_builder), progress, storage),
//        sprites: packing::extract_sprites(&packed)
//    };
//
//    Ok(sheet)
//}
