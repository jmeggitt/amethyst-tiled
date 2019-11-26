use amethyst::assets::AssetStorage;
use amethyst::assets::Loader;
use amethyst::assets::ProgressCounter;
use amethyst::renderer::rendy::texture::image::{load_from_image, ImageTextureConfig};
use amethyst::renderer::types::TextureData;
use amethyst::renderer::{SpriteSheet, Texture};
use failure::Error;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use tiled::parse_tileset;

pub mod packing;

pub fn load_tileset<P: AsRef<Path>>(
    path: P,
    loader: Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
) -> Result<SpriteSheet, Error> {
    let tileset = parse_tileset(File::open(path)?, 1)?;

    let packed = packing::pack_tileset(&tileset)?;
    let reader = BufReader::new(Cursor::new(&packed.bytes));

    let texture_builder = load_from_image(reader, ImageTextureConfig::default())?;

    let sheet = SpriteSheet {
        texture: loader.load_from_data(TextureData(texture_builder), progress, storage),
        sprites: packing::extract_sprites(&packed)
    };

    Ok(sheet)
}
