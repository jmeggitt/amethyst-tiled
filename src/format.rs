use std::path::PathBuf;
use std::sync::Arc;

use amethyst::assets::{Format, FormatValue, Prefab, SingleFile, Source};
use amethyst::Error;
use image::{load_from_memory, DynamicImage, ImageError, RgbaImage};
use tiled::{parse, parse_tileset, TilesetRef};

use crate::prefab::TileMapPrefab;
use crate::strategy::StrategyDesc;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

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
        #[cfg(feature = "profiler")]
        profile_scope!("import_tiled_format");

        let (b, m) = source.load_with_metadata(&name)?;

        let mut map = match parse(&b[..]) {
            Ok(v) => v,
            Err(e) => return Err(Error::new(e)),
        };

        for tileset in &mut map.tilesets {
            if let TilesetRef::Path(path, gid) = tileset {
                let file = shift_path(&name, path);
                let source = source.load(&file)?;

                let mut set = parse_tileset(&source[..], *gid)?;

                for image in &mut set.images {
                    image.source = shift_path(&file, &image.source);
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
            _ => Err(Error::from_string("Unable to read non rgba8 images".to_owned())),
        }
    }
}

/// Get an adjusted path based on a reference
fn shift_path(reference: &str, path: &str) -> String {
    let mut path_buf = PathBuf::from(reference);
    path_buf.set_file_name(path);
    path_buf.to_str().unwrap().to_owned()
}
