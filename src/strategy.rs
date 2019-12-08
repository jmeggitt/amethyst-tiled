use amethyst::assets::{AssetStorage, Handle, Loader, ProgressCounter, Source};
use amethyst::core::math::{Point3, Vector3};
use amethyst::ecs::{Read, ReadExpect, SystemData, Write};
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::tiles::{CoordinateEncoder, FlatEncoder, MapStorage, TileMap};
use amethyst::Error;
use tiled::Map;

use crate::packing::{pack_sparse_tileset_vec, pack_tileset_vec};
use crate::{load_sprite_sheet, TileGid};
use std::collections::{BTreeSet, HashMap};
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
#[derive(Debug, Copy, Clone, Default)]
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

        let packed = pack_tileset_vec(
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
}

/// A version of FlatLoad that tries to save time and memory by skipping unused tiles when
/// packing the sprite sheet and not leaving the unused tiles stored in memory. On the other hand,
/// if most or all of the tiles are used in the map it the regular version will be faster and use a
/// similar amount of memory.
///
/// In random experimentation, this method was ~2x (23.5s -> 12.6s) as fast as FlatLoad to load the
/// example.
#[derive(Debug, Copy, Clone, Default)]
pub struct CompressedLoad<E: CoordinateEncoder = FlatEncoder>(PhantomData<E>);

impl<E: CoordinateEncoder> StrategyDesc for CompressedLoad<E> {
    type Result = TileMap<TileGid, E>;
}

impl<'a, E: CoordinateEncoder> LoadStrategy<'a> for CompressedLoad<E> {
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

        let tile_usage: Vec<u32> = collect_gid_usage(map).into_iter().collect();

        let mut gid_updater = HashMap::new();

        for (new_index, old_index) in tile_usage.iter().enumerate() {
            gid_updater.insert(*old_index, new_index);
        }

        let packed = pack_sparse_tileset_vec(
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
                    let tile_ref =
                        tilemap.get_mut(&Point3::new(x as u32, y as u32, layer.layer_index));
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

/// Loads a tilemap into memory as a single texture. This is by far the best option for performance
/// when a map is never altered. Keep in mind that this approach will compress all of the maps
/// layers together.
#[derive(Debug, Copy, Clone, Default)]
pub struct StaticLoad;

impl StrategyDesc for StaticLoad {
    type Result = Handle<Texture>;
}

impl<'a> LoadStrategy<'a> for StaticLoad {
    type SystemData = ();

    fn load(
        _map: &Map,
        _source: Arc<dyn Source>,
        _progress: &mut ProgressCounter,
        _system_data: &mut Self::SystemData,
    ) -> Result<Self::Result, Error> {
        unimplemented!()
    }
}
