use amethyst::assets::{AssetStorage, Handle, Loader, PrefabData, ProgressCounter, Source};
use amethyst::core::math::{Point3, Vector3};
use amethyst::ecs::{Component, Entity, Read, ReadExpect, SystemData, Write, WriteStorage};
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::tiles::{CoordinateEncoder, FlatEncoder, MapStorage, TileMap};
use amethyst::Error;
use tiled::{Map, Tileset};

use crate::packing::pack_tileset_vec;
use crate::{load_sparse_map_inner, load_sprite_sheet, TileGid};
use crate::{load_tileset_inner, Tilesets};
use std::marker::PhantomData;
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
pub struct FlatLoad<E: CoordinateEncoder = FlatEncoder>(PhantomData<E>);

impl<E: CoordinateEncoder> StrategyDesc for FlatLoad<E> {
    type Result = TileMap<TileGid, E>;
}

impl<'a, E: CoordinateEncoder> LoadStrategy<'a> for FlatLoad<E> {
    type SystemData = (
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Texture>>,
        Write<'a, AssetStorage<SpriteSheet>>,
    );

    fn load(
        map: &Map,
        source: Arc<dyn Source>,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<Self::Result, Error> {
        let (loader, storage, sheets) = system_data;

        let packed = match pack_tileset_vec(
            &map.tilesets.iter().map(|x| x.unwrap().clone()).collect(),
            source,
        ) {
            Ok(v) => v,
            Err(e) => return Err(Error::from_string(format!("{:?}", e))),
        };

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
}

/// A load strategy that makes sure that unused tiles are not loaded to the gpu. This eliminates
/// possible memory leaks, but also means that the grid ids of tiles will not line up with the ones
/// used in Tiled.
pub struct CompressedLoad<E: CoordinateEncoder = FlatEncoder>(PhantomData<E>);

impl<E: CoordinateEncoder> StrategyDesc for CompressedLoad<E> {
    type Result = TileMap<TileGid, E>;
}
/// Loads a tilemap into memory as a single texture. This is by far the best option for performance
/// when a map is never altered. Keep in mind that this approach will compress all of the maps
/// layers together.
pub struct StaticLoad;
