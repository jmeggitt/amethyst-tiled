# Tiled file format support for Amethyst
This crate adds prefab loading functionality for Tiled tilemaps into the amethyst game engine.

## Usage
### Initialization
When creating your game, initialize the the prefab loader and include the tile map rendering pass `amethyst_tiles::RenderTiles2D`.
```rust
use tiled_support::{TileGid, TileMapPrefab};
use amethyst::tiles::{RenderTiles2D, FlatEncoder};

let game_data = GameDataBuilder::default()
    .with_system_desc(PrefabLoaderSystemDesc::<TileMapPrefab>::default(), "", &[])
    .with_bundle(
        RenderingBundle::<DefaultBackend>::new()
            .with_plugin(RenderTiles2D::<TileGid, FlatEncoder>::default()),
    )?;
```

> **Note:** `FlatEncoder` is mentioned explicitly due to a bug in the default encoder `MortonEncoder2D`. This bug will likely be fixed very soon.

### Loading a tile map
A tile map can be loaded like any other prefab and added to a texture.
```rust
use tiled_support::{TiledFormat, TileMapPrefab};

let prefab_handle =
    world.exec(|loader: PrefabLoader<'_, TileMapPrefab>| {
        loader.load("prefab/example_map.tmx", TiledFormat, ())
    });

let _map_entity = world
    .create_entity()
    .with(prefab_handle)
    .with(Transform::default())
    .build();
```



## Features to add:
A list of features I would like to add in the future, but havent had time to do yet.
- [x] Only pack sprites that used in the tile map to save memory and load time spent packing ignored sprites
- [x] Support all image/pixel types (Currently only supports Rgba8) ***Currently all images are converted to Rgba8***
- [x] Use `amethyst::error::Error` everywhere when parsing for consistency
- [ ] Mark flipped tiles so they can be correctly managed by amethyst
- [ ] Support animation sequences via tiles that swap textures periodically
- [ ] Create an easy way to access layer objects stored in tile maps
- [ ] Support image layers

Please make an issue if I'm forgetting something important in this list
 
