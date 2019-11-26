use amethyst::assets::Format;
use amethyst::assets::Prefab;
use amethyst::assets::PrefabData;
use amethyst::assets::PrefabLoader;
use amethyst::assets::ProgressCounter;
use amethyst::ecs::Entity;
use amethyst::Error;
use serde::{Deserialize, Serialize};
use tiled::{parse, parse_tileset, Map, Tileset, TiledError};

use amethyst::renderer::SpriteSheet;
use std::path::Path;
use std::fs::File;

mod format;
mod prefab;
mod set;
mod error;

use error::LoadError;

pub fn load_tileset<P: AsRef<Path>>(path: P) -> Result<SpriteSheet, LoadError> {
    let tileset = parse_tileset(File::open(path)?, 1)?;

    let packed = set::find_then_pack(&tileset);
    unimplemented!()
}