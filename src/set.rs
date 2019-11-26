//! Module to help pack tile sets and convert them into amethyst
//! https://github.com/amethyst/sheep/blob/master/sheep/examples/simple_pack/main.rs
use failure::Error;
use image::{GenericImage, GenericImageView, ImageError, Pixel, Rgba};
use sheep::{pack, InputSprite, SimplePacker, SpriteSheet};
use tiled::Tileset;

pub fn find_then_pack(tiles: &Tileset) -> Result<SpriteSheet, Error> {
    let mut tile_bytes = Vec::with_capacity(tiles.tiles.len());

    for img in tiles.images.iter() {
        let mut img_src = image::open(&img.source)?;
        let img_src = match img_src.as_mut_rgba8() {
            Some(v) => v,
            None => {
                return Err(
                    ImageError::FormatError("Unable to read non rgba8 images".to_owned()).into(),
                )
            }
        };

        for x in (tiles.margin..img.width as u32 - tiles.margin)
            .step_by((tiles.tile_width + tiles.spacing) as usize)
        {
            for y in (tiles.margin..img.height as u32 - tiles.margin)
                .step_by((tiles.tile_height + tiles.spacing) as usize)
            {
                let mut sub_img = img_src.sub_image(x, y, tiles.tile_width, tiles.tile_height);

                let bytes: Vec<u8> = if let Some(mut color) = img.transparent_colour {
                    let color = Rgba([color.red, color.green, color.blue, 0]);

                    sub_img
                        .pixels()
                        .map(|x| match x.2.to_rgb() == color.to_rgb() {
                            true => color,
                            false => x.2,
                        })
                        .flat_map(|x| x.0.to_vec())
                        .collect()
                } else {
                    sub_img.pixels().flat_map(|x| x.2 .0.to_vec()).collect()
                };

                let sprite = InputSprite {
                    dimensions: (tiles.tile_width, tiles.tile_height),
                    bytes,
                };

                tile_bytes.push(sprite);
            }
        }
    }

    let mut packed = pack::<SimplePacker>(tile_bytes, 4, Default::default());

    // There is guaranteed to be exactly one resulting sprite sheet
    Ok(packed.remove(0))
}
