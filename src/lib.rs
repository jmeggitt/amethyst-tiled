use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Mutex;

use amethyst::assets::{AssetStorage, Handle, Loader, ProgressCounter};
use amethyst::tiles::{TileMap, Tile, MapStorage};
use amethyst::renderer::rendy::texture::image::{load_from_image, ImageTextureConfig};
use amethyst::renderer::types::TextureData;
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::core::math::{Point3, Vector3};
use amethyst::ecs::World;
use failure::Error;
use tiled::{parse_tileset, Tileset, Map};

pub mod format;
pub mod loader;
pub mod packing;
pub mod prefab;
pub mod system;




/// The grid id of a tile
#[repr(transparent)]
#[derive(Copy, Clone, Hash, Default)]
pub struct TileGid(usize);

impl From<usize> for TileGid {
    fn from(idx: usize) -> Self {
        Self(idx)
    }
}

impl Tile for TileGid {
    fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
        Some(self.0)
    }
}

pub fn load_map_inner(
    map: &Map,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
    sheets: &mut AssetStorage<SpriteSheet>,
) -> Result<TileMap<TileGid>, Error> {

    let (packed, mapper) = packing::pack_tileset_vec(&map.tilesets)?;
    let reader = BufReader::new(Cursor::new(&packed.bytes));
    let texture_builder = load_from_image(reader, ImageTextureConfig::default())?;

    let sheet = SpriteSheet {
        texture: loader.load_from_data(TextureData(texture_builder), progress, storage),
        sprites: packing::extract_sprite_vec(&packed),
    };

    let sprite_sheet = sheets.insert(sheet);

    let map_size = Vector3::new(map.width, map.height, map.layers.len() as u32);
    let tile_size = Vector3::new(map.tile_width, map.tile_height, 1);

    let mut tilemap = TileMap::new(map_size, tile_size, Some(sprite_sheet));

    for layer in &map.layers {
        for x in 0..layer.tiles.len() {
            for y in 0..layer.tiles[x].len() {
                match tilemap.get_mut(&Point3::new(x as u32, y as u32, layer.layer_index)) {
                    Some(v) => *v = TileGid(mapper.map(layer.tiles[x][y] as usize).unwrap()),
                    None => unreachable!("The map file was corrupt"),
                }
            }
        }
    }

    Ok(tilemap)
}


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
