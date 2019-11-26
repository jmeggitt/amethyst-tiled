//! Module to help pack tile sets and convert them into amethyst
//! https://github.com/amethyst/sheep/blob/master/sheep/examples/simple_pack/main.rs
use failure::Error;
use image::{DynamicImage, GenericImage, GenericImageView, ImageError, Pixel, Rgba, RgbaImage};
use sheep::{InputSprite, pack, SimplePacker, SpriteSheet};
use tiled::Image as TileImage;
use tiled::Tileset;


pub fn pack_tileset(set: &Tileset) -> Result<SpriteSheet, Error> {
    let mut sprites = Vec::new();

    let tile_size = (set.tile_width, set.tile_height);

    for image in &set.images {
        sprites.extend(pack_image(image, tile_size, set.margin, set.spacing)?);
    }

    // There is guaranteed to be exactly one resulting sprite sheet
    Ok(pack::<SimplePacker>(sprites, 4, ()).remove(0))

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
        let color = Rgba([color.red, color.green, color.blue, 0]);

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
