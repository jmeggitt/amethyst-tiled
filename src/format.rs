use crate::prefab::{TileMapPrefab, TileSetPrefab};
use amethyst::assets::Format;
use amethyst::Error;
use tiled::{parse, parse_tileset, Map, Tileset};

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

impl Format<Tileset> for TiledFormat {
    fn name(&self) -> &'static str {
        "Tile Set"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<Tileset, Error> {
        parse_tileset(&bytes[..], 1).map_err(Error::new)
    }
}

impl Format<TileSetPrefab> for TiledFormat {
    fn name(&self) -> &'static str {
        "Tile Set Prefab"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<TileSetPrefab, Error> {
        let set = parse_tileset(&bytes[..], 1).map_err(Error::new)?;
        Ok(TileSetPrefab::TileSet(set))
    }
}

impl Format<TileMapPrefab> for TiledFormat {
    fn name(&self) -> &'static str {
        "Tile Map Prefab"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<TileMapPrefab, Error> {
        let map = parse(&bytes[..]).map_err(Error::new)?;
        Ok(TileMapPrefab::TileMap(map))
    }
}
