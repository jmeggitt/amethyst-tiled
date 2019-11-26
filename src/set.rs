//! Module to help pack tile sets and convert them into amethyst
//! https://github.com/amethyst/sheep/blob/master/sheep/examples/simple_pack/main.rs
use failure::Error;
use image::{DynamicImage, GenericImage, GenericImageView, ImageError, Pixel, Rgba, RgbaImage};
use sheep::{InputSprite, pack, SimplePacker, SpriteSheet};
use tiled::Image as TileImage;
use tiled::Tileset;


pub fn pack_tileset(set: &Tileset) -> Result<Vec<InputSprite>, Error> {
    let mut sprites = Vec::new();

    let tile_size = (set.tile_width, set.tile_height);

    for image in &set.images {
        sprites.extend(pack_image(image, tile_size, set.margin, set.spacing)?);
    }

    Ok(sprites)
}

pub fn pack_image(img: &TileImage, tile_size: (u32, u32), margin: u32, spacing: u32) -> Result<Vec<InputSprite>, Error> {
    let mut image = open_image(img)?;

    let (width, height) = tile_size;
    let mut sprites = Vec::new();

    for x in (margin..image.width() + margin).step_by((width + spacing) as usize) {
        for y in (margin..image.height() + margin).step_by((height + spacing) as usize) {
            sprites.push(InputSprite {
                dimensions: tile_size,
                bytes: sub_image_bytes(&mut image, x, y, width, height),
            })
        }
    }

    Ok(sprites)
}

/// Open the image and removes the transparent color
pub fn open_image(img: &TileImage) -> Result<RgbaImage, Error> {
    let mut image = match image::open(&img.source)? {
        DynamicImage::ImageRgba8(v) => v,
        _ => return Err(ImageError::FormatError("Unable to read non rgba8 images".to_owned()).into())
    };

    if let Some(color) = img.transparent_colour {
        let color = Rgba([color.red, color.green, color.blue, 0xFF]);

        for pixel in image.pixels_mut() {
            if pixel.to_rgb() == color.to_rgb() {
                *pixel = color;
            }
        }
    }

    Ok(image)
}

// Gets the bytes from a portion of an image
pub fn sub_image_bytes(img: &mut RgbaImage, x: u32, y: u32, width: u32, height: u32) -> Vec<u8> {
    img.sub_image(x, y, width, height).to_image().into_raw()
}

pub fn find_then_pack(tiles: &Tileset) -> Result<SpriteSheet, Error> {
    let mut tile_bytes = Vec::with_capacity(tiles.tiles.len());

    for img in tiles.images.iter() {
        let mut img_src = image::open(&img.source)?;
        let img_src = match img_src.as_mut_rgba8() {
            Some(v) => v,
            None => {
                return Err(
                    ImageError::FormatError("Unable to read non rgba8 images".to_owned()).into(),
                );
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
