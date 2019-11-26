use amethyst::assets::Format;
use amethyst::assets::Prefab;
use amethyst::assets::PrefabData;
use amethyst::assets::PrefabLoader;
use amethyst::assets::ProgressCounter;
use amethyst::ecs::Entity;
use amethyst::Error;
use serde::{Deserialize, Serialize};
use tiled::{parse, parse_tileset, Map, Tileset};

use amethyst::renderer::SpriteSheet;

mod format;
mod prefab;
mod set;

pub use format::*;

#[derive(Serialize, Deserialize)]
pub enum TileSetPrefab {
    #[serde(skip)]
    TileSet(Tileset),
    /// The name of the file
    Set(String),
}

impl<'a> PrefabData<'a> for TileSetPrefab {
    type SystemData = ();
    type Result = ();

    fn add_to_entity(
        &self,
        _entity: Entity,
        _system_data: &mut Self::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<Self::Result, Error> {
        unimplemented!()
    }

    fn load_sub_assets(
        &mut self,
        _progress: &mut ProgressCounter,
        _system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        unimplemented!()
    }
}

pub struct TileSheetLoadedSet {}
