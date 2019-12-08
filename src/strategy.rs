use amethyst::assets::{AssetStorage, Handle, Loader, PrefabData, ProgressCounter, Source};
use amethyst::ecs::{Entity, Read, ReadExpect, Write, WriteStorage, SystemData, Component};
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::tiles::{FlatEncoder, TileMap};
use amethyst::Error;
use tiled::{Map, Tileset};

use crate::{load_tileset_inner, Tilesets};
use crate::{load_sparse_map_inner, TileGid};
use std::sync::Arc;


pub trait StrategyDesc {
    /// The type of output this strategy will produce
    type Result;
}

pub trait LoadStrategy<'a>: StrategyDesc {
    /// The data to request when loading a map
    type SystemData: SystemData<'a>;

    // Preform the load operation using a given map and source location
    fn load(
        map: &Map,
        source: Arc<dyn Source>,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<<Self as StrategyDesc>::Result, Error>;
}

/// A load strategy that does not take steps to compress the tile sets. This is the most efficient
/// approach for maps that use all or near all of the tiles in a tileset.
pub struct FlatLoad;

/// A load strategy that makes sure that unused tiles are not loaded to the gpu. This eliminates
/// possible memory leaks, but also means that the grid ids of tiles will not line up with the ones
/// used in Tiled.
pub struct CompressedLoad;

/// Loads a tilemap into memory as a single texture. This is by far the best option for performance
/// when a map is never altered. Keep in mind that this approach will compress all of the maps
/// layers together.
pub struct StaticLoad;



