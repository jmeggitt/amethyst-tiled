use amethyst::assets::AssetStorage;
use amethyst::assets::Loader;
use amethyst::assets::ProgressCounter;
use amethyst::renderer::rendy::texture::image::{load_from_image, ImageTextureConfig};
use amethyst::renderer::types::TextureData;
use amethyst::renderer::{Sprite, SpriteSheet, Texture};
use failure::Error;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use tiled::parse_tileset;

use sheep::SpriteSheet as PackedSpriteSheet;
use sheep::{encode, AmethystFormat};

mod set;

pub fn load_tileset<P: AsRef<Path>>(
    path: P,
    loader: Loader,
    progress: &mut ProgressCounter,
    storage: &AssetStorage<Texture>,
) -> Result<PackedSpriteSheet, Error> {
    let tileset = parse_tileset(File::open(path)?, 1)?;
    let tile_width = tileset.tile_width;
    let tile_height = tileset.tile_height;

    let mut packed = set::find_then_pack(&tileset)?;
    let (img_width, img_height) = packed.dimensions;

    let formatted = encode::<AmethystFormat>(&packed, ());
    let reader = BufReader::new(Cursor::new(packed.bytes));

    let texture_builder = load_from_image(reader, ImageTextureConfig::default())?;

    let texture_handle = loader.load_from_data(TextureData(texture_builder), progress, storage);

    let sprites: Vec<Sprite> = formatted
        .sprites
        .iter()
        .map(|x| {
            Sprite::from_pixel_values(
                img_width,
                img_height,
                tile_width,
                tile_height,
                x.x as u32,
                x.y as u32,
                [0.0; 2],
                false,
                false,
            )
        })
        .collect();

    unimplemented!()
}
