use std::path::PathBuf;
use std::sync::Arc;

use amethyst::assets::{Format, FormatValue, Prefab, SingleFile, Source};
use amethyst::Error;
use image::{load_from_memory, DynamicImage, ImageError, RgbaImage};
use tiled::{parse, parse_tileset, TilesetRef};

use crate::prefab::TileMapPrefab;
use crate::strategy::StrategyDesc;

/// Format for loading *.tmx and *.tsx files
#[derive(Debug, Copy, Clone)]
pub struct TiledFormat;

impl<T: 'static> Format<Prefab<T>> for TiledFormat
where
    TiledFormat: Format<T>,
{
    fn name(&self) -> &'static str {
        <Self as Format<T>>::name(self)
    }

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        _: Option<Box<dyn Format<Prefab<T>>>>,
    ) -> Result<FormatValue<Prefab<T>>, Error> {
        let value = <Self as Format<T>>::import(self, name, source, None)?;
        Ok(FormatValue::data(Prefab::new_main(value.data)))
    }
}

impl<T: 'static + StrategyDesc> Format<TileMapPrefab<T>> for TiledFormat {
    fn name(&self) -> &'static str {
        "Tile Map"
    }

    fn import(
        &self,
        name: String,
        source: Arc<dyn Source>,
        create_reload: Option<Box<dyn Format<TileMapPrefab<T>>>>,
    ) -> Result<FormatValue<TileMapPrefab<T>>, Error> {
        let (b, m) = source.load_with_metadata(&name)?;

        let mut map = match parse(&b[..]) {
            Ok(v) => v,
            Err(e) => return Err(Error::new(e)),
        };

        for tileset in &mut map.tilesets {
            if let TilesetRef::Path(path, gid) = tileset {
                let mut path_buf = PathBuf::from(&name);
                path_buf.set_file_name(path);
                let source = source.load(path_buf.to_str().unwrap())?;

                let mut set = parse_tileset(&source[..], *gid)?;

                for image in &mut set.images {
                    let mut path_buf = path_buf.clone();
                    path_buf.set_file_name(&image.source);
                    image.source = path_buf.to_str().unwrap().to_owned();
                }

                *tileset = TilesetRef::TileSet(set);
            }
        }

        if let Some(boxed_format) = create_reload {
            Ok(FormatValue {
                data: TileMapPrefab::Map(map, source.clone()),
                reload: Some(Box::new(SingleFile::new(boxed_format, m, name, source))),
            })
        } else {
            Ok(FormatValue::data(TileMapPrefab::Map(map, source)))
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
            _ => Err(ImageError::FormatError("Unable to read non rgba8 images".to_owned()).into()),
        }
    }
}
