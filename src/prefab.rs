use amethyst::assets::AssetStorage;
use amethyst::assets::Loader;
use amethyst::assets::Prefab;
use amethyst::assets::PrefabData;
use amethyst::assets::PrefabLoader;
use amethyst::assets::ProgressCounter;
use amethyst::assets::Source;
use amethyst::ecs::{Entity, Read, ReadExpect};
use amethyst::renderer::sprite::prefab::SpriteSheetLoadedSet;
use amethyst::renderer::TexturePrefab;
use amethyst::Error;
use serde::{Deserialize, Serialize};
use tiled::{parse, parse_tileset, parse_with_path, Map, Tileset};

use amethyst::assets::Handle;
use amethyst::renderer::SpriteSheet;

use crate::format::TiledFormat;
use std::fs::read;

//pub fn tester(loader: &mut PrefabLoader<TileSetPrefab>, progress: &mut ProgressCounter) {
//    let a: Handle<SpriteSheet> = loader.load("", TiledFormat, progress);
//}

#[derive(Serialize, Deserialize)]
pub enum TileSetPrefab {
    #[serde(skip)]
    Handle(Handle<SpriteSheet>),
    #[serde(skip)]
    TileSet(Tileset),
}

impl From<Tileset> for TileSetPrefab {
    fn from(set: Tileset) -> Self {
        Self::TileSet(set)
    }
}

impl<'a> PrefabData<'a> for TileSetPrefab {
    type SystemData = (
        <TexturePrefab as PrefabData<'a>>::SystemData,
        Read<'a, SpriteSheetLoadedSet>,
        Read<'a, AssetStorage<SpriteSheet>>,
        ReadExpect<'a, Loader>,
    );
    type Result = Handle<SpriteSheet>;

    fn add_to_entity(
        &self,
        _entity: Entity,
        system_data: &mut Self::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<Self::Result, Error> {
        let (ref mut tex_data, ref mut loaded_set, storage, loader) = system_data;

        //        match self {
        //            Self::Set(path) => loader.load(path, TileSetFormat::default(), storage)
        //        }

        unimplemented!()
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (ref mut tex_data, ref mut loaded_set, storage, loader) = system_data;

        Ok(false)
    }
}
