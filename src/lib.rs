use tiled::parse_tileset;
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::renderer::rendy::texture::image::{load_from_image, ImageTextureConfig};
use amethyst::assets::Loader;
use amethyst::assets::AssetStorage;
use amethyst::assets::ProgressCounter;
use amethyst::renderer::types::TextureData;
use std::io::{Cursor, BufReader};
use std::fs::File;
use std::path::Path;
use failure::Error;

use sheep::SpriteSheet as PackedSpriteSheet;

mod set;


pub fn load_tileset<P: AsRef<Path>>(path: P, loader: Loader, progress: &mut ProgressCounter, storage: &AssetStorage<Texture>) -> Result<PackedSpriteSheet, Error> {
    let tileset = parse_tileset(File::open(path)?, 1)?;

    let packed = set::find_then_pack(&tileset)?;


    let reader = BufReader::new(Cursor::new(packed.bytes));

    let texture_builder = load_from_image(reader, ImageTextureConfig::default())?;

    let texture_handle = loader.load_from_data(TextureData(texture_builder), progress, storage);


    unimplemented!()
}
