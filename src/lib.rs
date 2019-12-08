use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

use amethyst::assets::{AssetStorage, Directory, Handle, Loader, ProgressCounter, Source};
use amethyst::core::math::{Point3, Vector3};
use amethyst::ecs::World;

use amethyst::renderer::rendy::{
    hal::image::{Filter, Kind, SamplerInfo, ViewKind, WrapMode},
    texture::{pixel::Rgba8Srgb, TextureBuilder},
};
use amethyst::renderer::{
    palette::{Pixel, Srgba},
    SpriteSheet, Texture,
};
use amethyst::tiles::{FlatEncoder, MapStorage, Tile, TileMap};
use failure::Error;
use sheep::{encode, SpriteSheet as PackedSpriteSheet};
use tiled::{parse_tileset, Map, Tileset};

pub mod format;
pub mod packing;
pub mod prefab;
pub mod strategy;

use packing::AmethystOrderedFormat;

pub type TileEncoder = FlatEncoder;

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

fn collect_gid_usage(map: &Map) -> BTreeSet<u32> {
    let mut gids = BTreeSet::new();
    for layer in &map.layers {
        for row in &layer.tiles {
            for tile in row {
                gids.insert(tile.gid);
            }
        }
    }
    gids
}

fn load_sprite_sheet(packed: PackedSpriteSheet,
                    loader: &Loader,
                    progress: &mut ProgressCounter,
                    storage: &AssetStorage<Texture>,) -> SpriteSheet {
    let (width, height) = packed.dimensions;

    let mut pixel_data = Vec::new();

    for pixel in Srgba::from_raw_slice(&packed.bytes) {
        pixel_data.push(Rgba8Srgb::from(pixel.clone()));
    }


    //    let texture_builder = build_texture(tex_width, tex_height, packed.bytes);
    let texture_builder = TextureBuilder::new()
        .with_kind(Kind::D2(width, height, 1, 1))
        .with_view_kind(ViewKind::D2)
        .with_data_width(width)
        .with_data_height(height)
        .with_sampler_info(SamplerInfo::new(Filter::Nearest, WrapMode::Clamp))
        .with_data(pixel_data);

    SpriteSheet {
        texture: loader.load_from_data(texture_builder.into(), progress, storage),
        sprites: encode::<AmethystOrderedFormat>(&packed, ()),
    }
}

/// A version of load_map_inner that tries to save time and memory by skipping unused tiles when
/// packing the sprite sheet and not leaving the unused tiles stored in memory. On the other hand,
/// if most or all of the tiles are used in the map it the regular version will be faster and use a
/// similar amount of memory.
///
/// In random experimentation, this method was ~2x (23.5s -> 12.6s) as fast to load the example
pub fn load_sparse_map_inner(
    map: &Map,
    source: Arc<dyn Source>,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
    sheets: &mut AssetStorage<SpriteSheet>,
) -> Result<TileMap<TileGid, FlatEncoder>, Error> {
    let tile_usage: Vec<u32> = collect_gid_usage(map).into_iter().collect();

    let mut gid_updater = HashMap::new();

    for (new_index, old_index) in tile_usage.iter().enumerate() {
        gid_updater.insert(*old_index, new_index);
    }

    let packed = packing::pack_sparse_tileset_vec(
        &map.tilesets.iter().map(|x| x.unwrap().clone()).collect(),
        source,
        &tile_usage[..],
    )?;

    let map_size = Vector3::new(map.width, map.height, map.layers.len() as u32);
    let tile_size = Vector3::new(map.tile_width, map.tile_height, 1);
    let sheet = load_sprite_sheet(packed, loader, progress, storage);

    let mut tilemap = TileMap::new(map_size, tile_size, Some(sheets.insert(sheet)));

    for layer in &map.layers {
        for y in 0..layer.tiles.len() {
            for x in 0..layer.tiles[y].len() {
                let tile_ref = tilemap.get_mut(&Point3::new(x as u32, y as u32, layer.layer_index));
                let tile_idx = gid_updater.get(&layer.tiles[y][x].gid);

                match (tile_ref, tile_idx) {
                    (Some(tile), Some(index)) => *tile = TileGid(*index),
                    _ => unreachable!("The available tiles should not have changed since the start of the function"),
                }
            }
        }
    }

    Ok(tilemap)
}

pub fn load_map_inner(
    map: &Map,
    source: Arc<dyn Source>,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
    sheets: &mut AssetStorage<SpriteSheet>,
) -> Result<TileMap<TileGid, FlatEncoder>, Error> {
    let packed = packing::pack_tileset_vec(
        &map.tilesets.iter().map(|x| x.unwrap().clone()).collect(),
        source,
    )?;

    let map_size = Vector3::new(map.width, map.height, map.layers.len() as u32);
    let tile_size = Vector3::new(map.tile_width, map.tile_height, 1);
    let sheet = load_sprite_sheet(packed, loader, progress, storage);

    let mut tilemap = TileMap::new(map_size, tile_size, Some(sheets.insert(sheet)));

    for layer in &map.layers {
        for y in 0..layer.tiles.len() {
            for x in 0..layer.tiles[y].len() {
                match tilemap.get_mut(&Point3::new(x as u32, y as u32, layer.layer_index)) {
                    Some(v) => *v = TileGid(layer.tiles[y][x].gid as usize),
                    None => unreachable!("The map file was corrupt"),
                }
            }
        }
    }

    Ok(tilemap)
}

fn load_tileset_inner(
    _tileset: &Tileset,
    _source: Arc<dyn Source>,
    _loader: &Loader,
    _progress: &mut ProgressCounter,
    _storage: &AssetStorage<Texture>,
) -> Result<SpriteSheet, Error> {
    unimplemented!()
}

pub fn load_tileset<P: AsRef<Path>>(
    path: P,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
) -> Result<SpriteSheet, Error> {
    let tileset = parse_tileset(File::open(&path)?, 1)?;

    load_tileset_inner(
        &tileset,
        Arc::new(Directory::new(path.as_ref())),
        loader,
        progress,
        storage,
    )
}

pub fn load_cached_tileset<P: AsRef<Path>>(
    path: P,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
    sprite_sheets: &mut AssetStorage<SpriteSheet>,
    tilesets: &Tilesets,
) -> Result<Handle<SpriteSheet>, Error> {
    let tileset = parse_tileset(File::open(&path)?, 1)?;

    match tilesets.get(&tileset.name) {
        Some(handle) => Ok(handle),
        None => {
            let sheet = load_tileset_inner(
                &tileset,
                Arc::new(Directory::new(path.as_ref())),
                loader,
                progress,
                storage,
            )?;
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
