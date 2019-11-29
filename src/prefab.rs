use amethyst::assets::{AssetStorage, Handle, Loader, PrefabData, ProgressCounter};
use amethyst::ecs::{Entity, Read, ReadExpect, Write};
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::tiles::TileMap;
use amethyst::Error;
use tiled::{Map, Tileset};

use crate::TileGid;
use crate::{load_map_inner, load_tileset_inner, Tilesets};

pub enum TileSetPrefab {
    Handle(Handle<SpriteSheet>),
    TileSet(Tileset),
}

impl From<Tileset> for TileSetPrefab {
    fn from(set: Tileset) -> Self {
        Self::TileSet(set)
    }
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

        if let Self::TileSet(set) = self {
            match tilesets.get(&set.name) {
                Some(handle) => *self = Self::Handle(handle),
                None => {
                    let sheet = match load_tileset_inner(set, loader, progress, textures) {
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

pub enum TileMapPrefab {
    Handle(Handle<TileMap<TileGid>>),
    TileMap(Map),
}

impl<'a> PrefabData<'a> for TileMapPrefab {
    type SystemData = (
        Read<'a, AssetStorage<Texture>>,
        Write<'a, AssetStorage<SpriteSheet>>,
        Write<'a, AssetStorage<TileMap<TileGid>>>,
        ReadExpect<'a, Loader>,
    );

    type Result = Handle<TileMap<TileGid>>;

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
        let (textures, sheets, maps, loader) = system_data;

        if let Self::TileMap(map) = self {
            let map = match load_map_inner(&map, loader, progress, textures, sheets) {
                Ok(v) => v,
                Err(e) => return Err(Error::from_string(format!("{:}", e))),
            };
            *self = Self::Handle(maps.insert(map));
            return Ok(true);
        }

        Ok(false)
    }
}
