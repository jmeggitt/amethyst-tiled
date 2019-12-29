//! Module to help pack tile sets and convert them into amethyst

use amethyst::assets::Source;
use amethyst::error::Error;
use amethyst::renderer::sprite::Sprite;
use image::{load_from_memory, GenericImage, Pixel, Rgba, RgbaImage};
use sheep::{
    pack, Format, InputSprite, Packer, PackerResult, SimplePacker, SpriteAnchor, SpriteData,
    SpriteSheet,
};
use std::sync::Arc;
use tiled::Image as TileImage;
use tiled::Tileset;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

pub struct AmethystOrderedFormat;

impl Format for AmethystOrderedFormat {
    type Data = Vec<Sprite>;
    type Options = ();

    fn encode(
        dimensions: (u32, u32),
        sprites: &[SpriteAnchor],
        _options: Self::Options,
    ) -> Self::Data {
        #[cfg(feature = "profiler")]
        profile_scope!("encode_amethyst_format");

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

/// A sprite packer that can save time on the packing by assuming all sprites will be the exact same
/// size. Because of this, it can pack everything in a single pass. However, it won't be as easy to
/// view and look at due to all of the sprites being put in a vertical line.
pub struct TilePacker;

impl Packer for TilePacker {
    type Options = ();

    fn pack(sprites: &[SpriteData], _options: Self::Options) -> Vec<PackerResult> {
        #[cfg(feature = "profiler")]
        profile_scope!("pack_tiled_image");

        let tile_dimensions = match sprites.get(0) {
            Some(v) => v.dimensions,
            None => {
                return vec![PackerResult {
                    dimensions: (0, 0),
                    anchors: Vec::new(),
                }]
            }
        };

        let (width, height) = tile_dimensions;

        let mut num = 0;
        let mut anchors = Vec::with_capacity(sprites.len());

        for sprite in sprites {
            anchors.push(SpriteAnchor {
                id: sprite.id,
                position: (0, num * height),
                dimensions: tile_dimensions,
            });

            num += 1;
        }

        vec![PackerResult {
            dimensions: (width, num * height),
            anchors,
        }]
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
    #[cfg(feature = "profiler")]
    profile_scope!("pack_image");

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
    #[cfg(feature = "profiler")]
    profile_scope!("pack_sparse_image");

    let mut image = open_image(img, source)?;

    let TileSpec {
        width,
        height,
        margin,
        spacing,
    } = spec;

    let grid_width = (image.width() - 2 * margin) / (width + spacing);
    let grid_height = (image.height() - 2 * margin) / (height + spacing);

    let mut sprites = Vec::new();
    let mut consumed_tiles = 0;

    for idx in usage.iter() {
        if *idx >= first_gid + grid_width * grid_height {
            break;
        }
        consumed_tiles += 1;

        let x = margin + ((idx - first_gid) % grid_width) * (width + spacing);
        let y = margin + ((idx - first_gid) / grid_width) * (height + spacing);

        sprites.push(InputSprite {
            dimensions: (width, height),
            bytes: image.sub_image(x, y, width, height).to_image().into_raw(),
        })
    }

    Ok((sprites, grid_width * grid_height, consumed_tiles))
}

/// Open the image and removes the transparent color
pub fn open_image(img: &TileImage, source: Arc<dyn Source>) -> Result<RgbaImage, Error> {
    #[cfg(feature = "profiler")]
    profile_scope!("open_image");

    let bytes = {
        #[cfg(feature = "profiler")]
        profile_scope!("load_image_source");

        match source.load(&img.source) {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::from_string(format!(
                    "Unable to open image path: {:?}",
                    &img.source
                )));
            }
        }
    };

    let mut image = {
        #[cfg(feature = "profiler")]
        profile_scope!("load_from_memory");

        // TODO: Leave images in their original formats and allow amethyst to deal with conversions
        // to save memory when possible
        load_from_memory(&bytes[..])?.to_rgba()
    };

    if let Some(color) = img.transparent_colour {
        #[cfg(feature = "profiler")]
        profile_scope!("apply_transparency");

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
    sets: &[Tileset],
    source: Arc<dyn Source>,
    usage: &[u32],
) -> Result<SpriteSheet, Error> {
    #[cfg(feature = "profiler")]
    profile_scope!("pack_sparse_tileset_vec");

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

    #[cfg(feature = "profiler")]
    profile_scope!("sheep_pack_image");

    // There is guaranteed to be exactly one resulting sprite sheet
    Ok(pack::<SimplePacker>(sprites, 4, ()).remove(0))
}

/// Pack a list of tile sets while paying attention to the first grid id
pub fn pack_tileset_vec(
    sets: &[Tileset],
    source: Arc<dyn Source>,
) -> Result<SpriteSheet, Error> {
    #[cfg(feature = "profiler")]
    profile_scope!("pack_tileset_vec");

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

    #[cfg(feature = "profiler")]
    profile_scope!("sheep_pack_image");

    // There is guaranteed to be exactly one resulting sprite sheet
    Ok(pack::<SimplePacker>(sprites, 4, ()).remove(0))
}
