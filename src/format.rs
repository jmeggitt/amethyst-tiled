use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use amethyst::assets::{self, Format, FormatValue, Prefab, SingleFile, Source};
use amethyst::Error;
use image::{DynamicImage, ImageError, load_from_memory, RgbaImage};
use sheep::InputSprite;
use tiled::{Map, parse, parse_file, parse_tileset, parse_with_path, Tileset, TilesetRef};

use crate::prefab::{TileMapPrefab, TileSetPrefab};

/// Format for loading *.tmx and *.tsx files
#[derive(Debug, Copy, Clone)]
pub struct TiledFormat;

impl<T: 'static> Format<Prefab<T>> for TiledFormat
    where TiledFormat: Format<T> {
    fn name(&self) -> &'static str {
        <Self as Format<T>>::name(self)
    }

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        create_reload: Option<Box<dyn Format<Prefab<T>>>>,
    ) -> Result<FormatValue<Prefab<T>>, Error> {
        let value =  <Self as Format<T>>::import(self, name, source, None)?;
        Ok(FormatValue::data(Prefab::new_main(value.data)))
    }
}

impl Format<TileSetPrefab> for TiledFormat {
    fn name(&self) -> &'static str {
        "Tile Set"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<TileSetPrefab, Error> {
        let set = parse_tileset(&bytes[..], 1).map_err(Error::new)?;
        Ok(TileSetPrefab::TileSet(set))
    }
}

impl Format<TileMapPrefab> for TiledFormat {
    fn name(&self) -> &'static str {
        "Tile Map"
    }

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        create_reload: Option<Box<dyn Format<TileMapPrefab>>>,
    ) -> Result<FormatValue<TileMapPrefab>, Error> {
        let (b, m) = source
            .load_with_metadata(&name)?;

        println!("Loading with correct method!");
        let mut map = match parse(&b[..]) {
            Ok(v) => v,
            Err(e) => return Err(Error::new(e)),
        };

        for tileset in &mut map.tilesets {
            if let TilesetRef::Path(path, gid) = tileset {
                let source = source.load(path)?;
                *tileset = TilesetRef::TileSet(parse_tileset(&source[..], *gid)?);
                println!("Fixed tileset!");
            }
        }

        if let Some(boxed_format) = create_reload {
            Ok(FormatValue {
                data: TileMapPrefab::TileMap(map),
                reload: Some(Box::new(SingleFile::new(boxed_format, m, name, source))),
            })
        } else {
            Ok(FormatValue::data(TileMapPrefab::TileMap(map)))
        }
    }
}

impl Format<RgbaImage> for TiledFormat {
    fn name(&self) -> &'static str {
        "Rgba Image"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<RgbaImage, Error> {
        match load_from_memory(&bytes[..])? {
            DynamicImage::ImageRgba8(v) => Ok(v),
            _ => Err(ImageError::FormatError("Unable to read non rgba8 images".to_owned()).into())
        }
    }
}




