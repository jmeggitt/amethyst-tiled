use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

use amethyst::assets::{AssetStorage, Directory, Handle, Loader, ProgressCounter, Source};
use amethyst::core::math::Point3;
use amethyst::ecs::World;

use amethyst::error::Error;
use amethyst::renderer::rendy::{
    hal::image::{Filter, Kind, SamplerInfo, ViewKind, WrapMode},
    texture::{pixel::AsPixel, pixel::Rgba8Srgb, TextureBuilder},
};
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::tiles::Tile;
use sheep::{encode, SpriteSheet as PackedSpriteSheet};
use tiled::{parse_tileset, Tileset};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

mod format;
pub mod packing;
mod prefab;
pub mod strategy;

use packing::AmethystOrderedFormat;

pub use format::TiledFormat;
pub use prefab::*;
pub use strategy::{CompressedLoad, FlatLoad, StaticLoad};

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

fn load_sprite_sheet(
    packed: PackedSpriteSheet,
    loader: &Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
) -> SpriteSheet {
    #[cfg(feature = "profiler")]
    profile_scope!("load_sprite_sheet");

    let sprites = encode::<AmethystOrderedFormat>(&packed, ());

    let (width, height) = packed.dimensions;

    let texture_builder = TextureBuilder::new()
        .with_kind(Kind::D2(width, height, 1, 1))
        .with_view_kind(ViewKind::D2)
        .with_data_width(width)
        .with_data_height(height)
        .with_sampler_info(SamplerInfo::new(Filter::Nearest, WrapMode::Clamp))
        .with_raw_data(packed.bytes, Rgba8Srgb::FORMAT);

    SpriteSheet {
        texture: loader.load_from_data(texture_builder.into(), progress, storage),
        sprites,
    }
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
