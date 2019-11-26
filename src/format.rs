/// It is impossible to return a sprite sheet from this context to is may be better to just use a prefab
use amethyst::assets::Format;
use amethyst::assets::FormatValue;
use amethyst::assets::Prefab;
use amethyst::assets::Source;
use amethyst::renderer::sprite::prefab::SpriteSheetPrefab;
use amethyst::renderer::SpriteSheet;
use amethyst::renderer::TexturePrefab;
use amethyst::Error;
use tiled::{parse, parse_tileset, Map, Tileset};

use crate::prefab::TileSetPrefab;
use std::sync::Arc;

/// Format for loading *.tmx and *.tsx files
#[derive(Debug, Copy, Clone)]
pub struct TiledFormat;

impl Format<Map> for TiledFormat {
    fn name(&self) -> &'static str {
        "Tile Map"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<Map, Error> {
        parse(&bytes[..]).map_err(Error::new)
    }
}

impl Format<Prefab<TileSetPrefab>> for TiledFormat {
    fn name(&self) -> &'static str {
        "Tile Map"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<Prefab<TileSetPrefab>, Error> {
        match parse_tileset(&bytes[..], 1) {
            Ok(v) => Ok(Prefab::new_main(v.into())),
            Err(e) => Err(Error::new(e)),
        }
    }
}
