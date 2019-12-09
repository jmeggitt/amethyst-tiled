use amethyst::assets::{Asset, AssetStorage, Handle, Loader, PrefabData, ProgressCounter, Source};
use amethyst::ecs::{Component, Entity, Read, ReadExpect, Write, WriteStorage};
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::Error;
use tiled::{Map, Tileset};

use crate::strategy::{CompressedLoad, LoadStrategy, StrategyDesc};
use crate::{load_tileset_inner, Tilesets};
use std::sync::Arc;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

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

pub enum TileMapPrefab<S: StrategyDesc = CompressedLoad> {
    Result(S::Result),
    Map(Map, Arc<dyn Source>),
}

impl<'a, T: LoadStrategy<'a>> PrefabData<'a> for TileMapPrefab<T>
where
    T::Result: Clone + Component + Asset,
{
    type SystemData = (T::SystemData, WriteStorage<'a, <T as StrategyDesc>::Result>);

    // Don't use a result due to the requirement of cloning the tilemap extra times
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<(), Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("add_tilemap_to_entity");

        let (_, storage) = system_data;

        match self {
            TileMapPrefab::Result(v) => {
                storage.insert(entity, v.clone())?;
                Ok(())
            }
            _ => unreachable!("load_sub_assets should be called before add_to_entity"),
        }
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("load_tilemap_assets");
        match self {
            TileMapPrefab::Map(map, source) => {
                *self = Self::Result(T::load(map, source.clone(), progress, &mut system_data.0)?);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
