use amethyst::assets::{AssetStorage, Handle, Loader, PrefabData, ProgressCounter, Source};
use amethyst::ecs::{Component, Entity, Read, ReadExpect, SystemData, Write, WriteStorage};
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::tiles::{FlatEncoder, TileMap};
use amethyst::Error;
use tiled::{Map, Tileset};

use crate::strategy::{LoadStrategy, StrategyDesc};
use crate::{load_sparse_map_inner, TileGid};
use crate::{load_tileset_inner, Tilesets};
use std::sync::Arc;

pub enum TileSetPrefab {
    Handle(Handle<SpriteSheet>),
    TileSet(Tileset, Arc<dyn Source>),
}

impl<'a> PrefabData<'a> for TileSetPrefab {
    type SystemData = (
        Write<'a, Tilesets>,
        Read<'a, AssetStorage<Texture>>,
        Write<'a, AssetStorage<SpriteSheet>>,
        ReadExpect<'a, Loader>,
    );

    type Result = Handle<SpriteSheet>;

    fn add_to_entity(
        &self,
        _entity: Entity,
        _system_data: &mut Self::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<Self::Result, Error> {
        match self {
            Self::Handle(handle) => Ok(handle.clone()),
            _ => unreachable!("load_sub_assets should be called before add_to_entity"),
        }
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (tilesets, textures, sheets, loader) = system_data;

        if let Self::TileSet(set, source) = self {
            match tilesets.get(&set.name) {
                Some(handle) => *self = Self::Handle(handle),
                None => {
                    let sheet =
                        match load_tileset_inner(set, source.clone(), loader, progress, textures) {
                            Ok(v) => v,
                            Err(e) => return Err(Error::from_string(format!("{:}", e))),
                        };
                    let handle = sheets.insert(sheet);
                    tilesets.push(set.name.to_owned(), handle.clone());

                    *self = Self::Handle(handle);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

pub enum MapPrefab<S: StrategyDesc> {
    Result(S::Result),
    Map(Map, Arc<dyn Source>),
}

impl<'a, T: LoadStrategy<'a>> PrefabData<'a> for MapPrefab<T>
where
    T::Result: Clone + Component,
{
    type SystemData = (T::SystemData, WriteStorage<'a, <T as StrategyDesc>::Result>);
    type Result = T::Result;

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<T::Result, Error> {
        let (_, storage) = system_data;

        match self {
            Self::Result(v) => {
                storage.insert(entity, v.clone())?;
                Ok(v.clone())
            }
            _ => unreachable!("load_sub_assets should be called before add_to_entity"),
        }
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        match self {
            Self::Map(map, source) => {
                *self = Self::Result(T::load(map, source.clone(), progress, &mut system_data.0)?);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub enum TileMapPrefab {
    Handle(Handle<TileMap<TileGid, FlatEncoder>>),
    TileMap(Map, Arc<dyn Source>),
}

impl<'a> PrefabData<'a> for TileMapPrefab {
    type SystemData = (
        Read<'a, AssetStorage<Texture>>,
        Write<'a, AssetStorage<SpriteSheet>>,
        Write<'a, AssetStorage<TileMap<TileGid, FlatEncoder>>>,
        WriteStorage<'a, TileMap<TileGid, FlatEncoder>>,
        ReadExpect<'a, Loader>,
    );

    type Result = Handle<TileMap<TileGid, FlatEncoder>>;

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<Self::Result, Error> {
        match self {
            Self::Handle(handle) => {
                system_data
                    .3
                    .insert(entity, system_data.2.get(handle).unwrap().clone())
                    .ok();
                Ok(handle.clone())
            }
            _ => unreachable!("load_sub_assets should be called before add_to_entity"),
        }
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (textures, sheets, maps, _, loader) = system_data;
        if let Self::TileMap(map, source) = self {
            let map = match load_sparse_map_inner(
                &map,
                source.clone(),
                loader,
                progress,
                textures,
                sheets,
            ) {
                Ok(v) => v,
                Err(e) => return Err(Error::from_string(format!("{:?}", e))),
            };

            *self = Self::Handle(maps.insert(map));

            return Ok(true);
        }
        Ok(false)
    }
}
