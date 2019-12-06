//! Module to help pack tile sets and convert them into amethyst
//! https://github.com/amethyst/sheep/blob/master/sheep/examples/simple_pack/main.rs
use amethyst::assets::Source;
use amethyst::renderer::sprite::Sprite;
use failure::{Context, Error};
use image::{DynamicImage, GenericImage, ImageError, Pixel, Rgba, RgbaImage};
use sheep::{pack, Format, InputSprite, SimplePacker, SpriteAnchor, SpriteSheet};
use std::sync::Arc;
use tiled::Image as TileImage;
use tiled::Tileset;

pub struct AmethystOrderedFormat;

impl Format for AmethystOrderedFormat {
    type Data = Vec<Sprite>;
    type Options = ();

    fn encode(
        dimensions: (u32, u32),
        sprites: &[SpriteAnchor],
        _options: Self::Options,
    ) -> Self::Data {
        // Fix ordering issues
        let mut inputs = sprites.to_vec();
        inputs.sort_by_key(|x| x.id);

        let (width, height) = dimensions;

        inputs
            .iter()
            .map(|anchor| {
                let (pixel_left, pixel_top) = anchor.position;
                let (sprite_w, sprite_h) = anchor.dimensions;
                Sprite::from_pixel_values(
                    width, height, sprite_w, sprite_h, pixel_left, pixel_top, [1.0; 2], false,
                    false,
                )
            })
            .collect()
    }
}

pub fn pack_tileset(set: &Tileset, source: Arc<dyn Source>) -> Result<SpriteSheet, Error> {
    let mut sprites = Vec::new();

    for image in &set.images {
        sprites.extend(pack_image(
            image,
            source.clone(),
            TileSpec {
                width: set.tile_width,
                height: set.tile_height,
                margin: set.margin,
                spacing: set.spacing,
            },
        )?);
    }

    // There is guaranteed to be exactly one resulting sprite sheet
    Ok(pack::<SimplePacker>(sprites, 4, ()).remove(0))
}

pub struct TileSpec {
    pub width: u32,
    pub height: u32,
    pub margin: u32,
    pub spacing: u32,
}

pub fn pack_image(
    img: &TileImage,
    source: Arc<dyn Source>,
    spec: TileSpec,
) -> Result<Vec<InputSprite>, Error> {
    let mut image = open_image(img, source)?;

    let TileSpec {
        width,
        height,
        margin,
        spacing,
    } = spec;
    let mut sprites = Vec::new();
    for y in (margin..image.height() + margin).step_by((height + spacing) as usize) {
        for x in (margin..image.width() + margin).step_by((width + spacing) as usize) {
            sprites.push(InputSprite {
                dimensions: (width, height),
                bytes: image.sub_image(x, y, width, height).to_image().into_raw(),
            })
        }
    }

    Ok(sprites)
}

/// Returns the necessary import sprites, the number of tiles within this image, and the number of
/// gids filled by this call.
pub fn pack_sparse_image(
    img: &TileImage,
    source: Arc<dyn Source>,
    spec: TileSpec,
    first_gid: u32,
    usage: &[u32],
) -> Result<(Vec<InputSprite>, u32, usize), Error> {
    let mut image = open_image(img, source)?;

    let TileSpec {
        width,
        height,
        margin,
        spacing,
    } = spec;

    let grid_width = (image.width() - 2 * margin) / (width + spacing);
    let grid_height = (image.width() - 2 * margin) / (width + spacing);

    let mut sprites = Vec::new();
    let mut consumed_tiles = 0;

    for idx in usage.iter() {
        if *idx >= first_gid + grid_width * grid_height {
            break;
        }
        consumed_tiles += 1;

        let x = margin + ((idx - first_gid) % grid_width) * (height + spacing);
        let y = margin + ((idx - first_gid) / grid_height) * (width + spacing);

        sprites.push(InputSprite {
            dimensions: (width, height),
            bytes: image.sub_image(x, y, width, height).to_image().into_raw(),
        })
    }

    Ok((sprites, grid_width * grid_height, consumed_tiles))
}

/// Open the image and removes the transparent color
pub fn open_image(img: &TileImage, source: Arc<dyn Source>) -> Result<RgbaImage, Error> {
    let bytes = match source.load(&img.source) {
        Ok(v) => v,
        Err(_) => panic!("Unable to find image: {:?}", img),
    };

    let image = match image::load_from_memory(&bytes[..]) {
        Ok(v) => v,
        Err(_) => {
            return Err(
                Context::from(format!("Unable to open image path: {:?}", &img.source)).into(),
            );
        }
    };
    let mut image = match image {
        DynamicImage::ImageRgba8(v) => v,
        _ => {
            return Err(
                ImageError::FormatError("Unable to read non rgba8 images".to_owned()).into(),
            )
        }
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

pub fn pack_sparse_tileset_vec(
    sets: &Vec<Tileset>,
    source: Arc<dyn Source>,
    usage: &[u32],
) -> Result<SpriteSheet, Error> {
    let mut sprites = Vec::new();
    let tile_size = (sets[0].tile_width, sets[0].tile_height);

    // Add the see through placeholder.
    sprites.push(InputSprite {
        bytes: vec![0; (tile_size.0 * tile_size.1 * 4) as usize],
        dimensions: tile_size,
    });

    // Don't load GID 0
    let mut tile_index = 1;

    for set in sets {
        let mut first_gid = set.first_gid;

        for image in &set.images {
            let (input_sprites, len, consumed) = pack_sparse_image(
                image,
                source.clone(),
                TileSpec {
                    width: set.tile_width,
                    height: set.tile_height,
                    margin: set.margin,
                    spacing: set.spacing,
                },
                first_gid,
                &usage[tile_index..],
            )?;

            first_gid += len;
            tile_index += consumed;

            sprites.extend(input_sprites);
        }
    }

    // There is guaranteed to be exactly one resulting sprite sheet
    Ok(pack::<SimplePacker>(sprites, 4, ()).remove(0))
}

/// Pack a list of tile sets while paying attention to the first grid id
pub fn pack_tileset_vec(
    sets: &Vec<Tileset>,
    source: Arc<dyn Source>,
) -> Result<SpriteSheet, Error> {
    let mut sprites = Vec::new();
    let tile_size = (sets[0].tile_width, sets[0].tile_height);

    // Add the see through placeholder.
    sprites.push(InputSprite {
        bytes: vec![0; (tile_size.0 * tile_size.1 * 4) as usize],
        dimensions: tile_size,
    });

    for set in sets {
        for image in &set.images {
            sprites.extend(pack_image(
                image,
                source.clone(),
                TileSpec {
                    width: set.tile_width,
                    height: set.tile_height,
                    margin: set.margin,
                    spacing: set.spacing,
                },
            )?);
        }
    }

    // There is guaranteed to be exactly one resulting sprite sheet
    Ok(pack::<SimplePacker>(sprites, 4, ()).remove(0))
}
