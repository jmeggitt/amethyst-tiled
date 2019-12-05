//! Module to help pack tile sets and convert them into amethyst
//! https://github.com/amethyst/sheep/blob/master/sheep/examples/simple_pack/main.rs
use amethyst::renderer::sprite::{Sprite, SpriteList, SpritePosition, Sprites, TextureCoordinates};
use amethyst::assets::Source;
use failure::{Context, Error};
use image::{DynamicImage, GenericImage, ImageError, Pixel, Rgba, RgbaImage};
use sheep::{encode, pack, AmethystFormat, InputSprite, SimplePacker, SpriteSheet, Format, SpriteAnchor};
use std::ops::Range;
use tiled::Image as TileImage;
use tiled::Tileset;
use std::sync::Arc;

struct AmethystOrderedFormat;
impl Format for AmethystOrderedFormat {
    type Data = Vec<Sprite>;
    type Options = ();

    fn encode(dimensions: (u32, u32), sprites: &[SpriteAnchor], options: Self::Options) -> Self::Data {
        // Fix ordering issues
        let mut inputs = sprites.to_vec();
        inputs.sort_by_key(|x|x.id);

//        let width = dimensions.0 as f32;
//        let height = dimensions.1 as f32;
        let (width, height) = dimensions;

        inputs.iter().map(|anchor| {
//            let (sprite_width, sprite_height) = anchor.dimensions;
//            let (x, y) = anchor.position;
//
//            let position = TextureCoordinates {
//                left: x as f32 / width,
//                right: 1.0 - (x + sprite_width) as f32 / width,
//                bottom: 1.0 - (y + sprite_height) as f32 / height,
//                top: y as f32 / height,
//            };


            // |------------|
            // |--|
            //    |-|
//            println!("{:?}\n\t{:?}", anchor, position);

//            Sprite {
//                width: anchor.dimensions.0 as f32,
//                height: anchor.dimensions.1 as f32,
//                offsets: [0.0; 2],
//                tex_coords: position,
//            }
            let (pixel_left, pixel_top) = anchor.position;
            let (sprite_w, sprite_h) = anchor.dimensions;
            Sprite::from_pixel_values(width, height, sprite_w, sprite_h, pixel_left, pixel_top, [1.0; 2], false, false)
        }).collect()
    }
}

pub fn extract_sprite_vec(sheet: &SpriteSheet) -> Vec<Sprite> {
//    encode::<Tester>(&sheet, ());
//    let formatted = encode::<AmethystFormat>(&sheet, ());
//    let mut sprites = Vec::with_capacity(formatted.sprites.len());
//
//    for sprite in formatted.sprites {
//        let position = TextureCoordinates {
//            left: sprite.x,
//            right: formatted.texture_width - sprite.x - sprite.width,
//            bottom: formatted.texture_height - sprite.y - sprite.height,
//            top: sprite.y,
//        };
//
//        let sprite = Sprite {
//            width: sprite.width,
//            height: sprite.height,
//            offsets: sprite.offsets.unwrap_or([0.0; 2]),
//            tex_coords: position,
//        };
//
////        if print > 0 {
////            println!("\tSprite: {:?}", sprite);
////            print -= 1;
////        }
//
//        sprites.push(sprite);
//    }
//
//    sprites
    encode::<AmethystOrderedFormat>(&sheet, ())
}

pub fn extract_sprites(sheet: &SpriteSheet) -> Sprites {
    let formatted = encode::<AmethystFormat>(&sheet, ());
    let mut sprites = Vec::with_capacity(formatted.sprites.len());

    for sprite in formatted.sprites {
        let sprite = SpritePosition {
            x: sprite.x as u32,
            y: sprite.y as u32,
            width: sprite.width as u32,
            height: sprite.height as u32,
            offsets: sprite.offsets,
            flip_horizontal: false,
            flip_vertical: false,
        };
        sprites.push(sprite);
    }

    Sprites::List(SpriteList {
        texture_width: formatted.texture_width as u32,
        texture_height: formatted.texture_height as u32,
        sprites,
    })
}

pub fn pack_tileset(set: &Tileset, source: Arc<dyn Source>) -> Result<SpriteSheet, Error> {
    let mut sprites = Vec::new();

    let tile_size = (set.tile_width, set.tile_height);

    for image in &set.images {
        sprites.extend(pack_image(image, source.clone(), tile_size, set.margin, set.spacing)?);
    }

    // There is guaranteed to be exactly one resulting sprite sheet
    Ok(pack::<SimplePacker>(sprites, 4, ()).remove(0))
}

pub fn pack_image(
    img: &TileImage,
    source: Arc<dyn Source>,
    tile_size: (u32, u32),
    margin: u32,
    spacing: u32,
) -> Result<Vec<InputSprite>, Error> {
    let mut image = open_image(img, source)?;
    println!("Tile size: {:?}", tile_size);

    let (width, height) = tile_size;
    let mut sprites = Vec::new();
    for y in (margin..image.height() + margin).step_by((height + spacing) as usize) {
        for x in (margin..image.width() + margin).step_by((width + spacing) as usize) {
            sprites.push(InputSprite {
                dimensions: tile_size,
                bytes: image.sub_image(x, y, width, height).to_image().into_raw(),
            })
        }
    }

    Ok(sprites)
}

/// Open the image and removes the transparent color
pub fn open_image(img: &TileImage, source: Arc<dyn Source>) -> Result<RgbaImage, Error> {
    let bytes = match source.load(&img.source) {
        Ok(v) => v,
        Err(e) => panic!("{:?}", img),
    };

    let image = match image::load_from_memory(&bytes[..]) {
        Ok(v) => v,
        Err(e) => {
            println!("Unable to open image path: {:?}", &img.source);
            return Err(e.into());
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

/// When a tilemap is loaded, the grid ids may not line up correctly. For instance, the first grid
/// id in one tileset can be an arbitrary value. To simplify things, this struct maps the original
/// id to the new compressed id starting from 0.
#[derive(Default)]
pub struct GidMapper {
    gid: Vec<usize>,
    len: Vec<usize>,
}

impl GidMapper {
    /// Checks for collisions between the requested grid and the existing ones.
    fn collisions(&self, area: Range<usize>) -> bool {
        for set in 0..self.len() {
            let gid = self.gid[set];
            let len = self.len[set];

            if gid + len > area.start && area.end > gid {
                return true;
            }
        }

        false
    }

    pub fn add_set(&mut self, first_gid: usize, len: usize) -> bool {
        if !self.collisions(first_gid..first_gid + len) {
            self.gid.push(first_gid);
            self.len.push(len);
            return true;
        }

        false
    }

    pub fn map(&self, idx: usize) -> Option<usize> {
        let mut stride = 0;

        for set in 0..self.len() {
            let gid = self.gid[set];
            let len = self.len[set];

            if idx >= gid && idx < gid + len {
                return Some(stride + idx - gid);
            } else {
                stride += len;
            }
        }

        None
    }

    pub fn len(&self) -> usize {
        self.gid.len()
    }
}

/// Pack a list of tile sets while paying attention to the first grid id
pub fn pack_tileset_vec(sets: &Vec<Tileset>, source: Arc<dyn Source>) -> Result<(SpriteSheet, GidMapper), Error> {
    println!("Beginning input extraction...");
    let mut sprites = Vec::new();
    let mut mapper = GidMapper::default();

    let tile_size = (sets[0].tile_width, sets[0].tile_height);
    sprites.push(InputSprite {
        bytes: vec![0; (tile_size.0 * tile_size.1 * 4) as usize],
        dimensions: tile_size,
    });


    for set in sets {
//        let tile_size = (set.tile_width, set.tile_height);
        let start_len = sprites.len();

        for image in &set.images {
            sprites.extend(pack_image(image, source.clone(), tile_size, set.margin, set.spacing)?);
        }

        if !mapper.add_set(set.first_gid as usize, sprites.len() - start_len) {
            return Err(Context::from("Unable to resolve first gid of tile sets in map").into());
        }
    }
    println!("Finished input extraction\nPacking...");
    // There is guaranteed to be exactly one resulting sprite sheet
    let packed = pack::<SimplePacker>(sprites, 4, ()).remove(0);

    Ok((packed, mapper))
}
