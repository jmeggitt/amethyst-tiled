use std::collections::HashMap;
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
use amethyst::renderer::{palette::Srgba, SpriteSheet, Texture};
use amethyst::tiles::{FlatEncoder, MapStorage, Tile, TileMap};
use failure::Error;
use sheep::encode;
use tiled::{parse_tileset, Map, Tileset};

pub mod format;
pub mod packing;
pub mod prefab;

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

pub fn load_map_inner(
    map: &Map,
    source: Arc<dyn Source>,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
    sheets: &mut AssetStorage<SpriteSheet>,
) -> Result<TileMap<TileGid, crate::TileEncoder>, Error> {
    let (packed, _mapper) = packing::pack_tileset_vec(
        &map.tilesets.iter().map(|x| x.unwrap().clone()).collect(),
        source,
    )?;

    let mut pixel_data = Vec::new();

    for idx in (0..packed.bytes.len()).step_by(4) {
        pixel_data.push(Rgba8Srgb::from(Srgba::new(
            packed.bytes[idx],
            packed.bytes[idx + 1],
            packed.bytes[idx + 2],
            packed.bytes[idx + 3],
        )));
    }

    let texture_builder = TextureBuilder::new()
        .with_kind(Kind::D2(packed.dimensions.0, packed.dimensions.1, 1, 1))
        .with_view_kind(ViewKind::D2)
        .with_data_width(packed.dimensions.0)
        .with_data_height(packed.dimensions.1)
        .with_sampler_info(SamplerInfo::new(Filter::Nearest, WrapMode::Clamp))
        .with_data(pixel_data);

    let sheet = SpriteSheet {
        texture: loader.load_from_data(texture_builder.into(), progress, storage),
        sprites: encode::<AmethystOrderedFormat>(&packed, ()),
    };

    let sprite_sheet = sheets.insert(sheet);

    let map_size = Vector3::new(map.width, map.height, map.layers.len() as u32);
    let tile_size = Vector3::new(map.tile_width, map.tile_height, 1);

    let mut tilemap = TileMap::new(map_size, tile_size, Some(sprite_sheet));

    for layer in &map.layers {
        for y in 0..layer.tiles.len() {
            for x in 0..layer.tiles[y].len() {
                match tilemap.get_mut(&Point3::new(x as u32, y as u32, layer.layer_index)) {
                    Some(v) => *v = TileGid(layer.tiles[y][x].gid as usize),
                    //                    Some(v) => *v = TileGid(mapper.map(layer.tiles[x][y].gid as usize).unwrap()),
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
