# Tiled file format support for Amethyst
This crate is here to make it easier to load Tiled tilemaps into the amethyst game engine. Its very much a work in
progress and any help/ideas are appreciated.



### TODO:
- [ ] Only pack sprites that used in the tile map to save memory and load time spent packing ignored sprites
- [ ] Support all image/pixel types (Currently only supports Rgba8)
- [ ] Use `amethyst::error::Error` everywhere when parsing for consistency
- [ ] Mark flipped tiles so they can be correctly managed by amethyst
- [ ] Support animation sequences via tiles that swap textures periodically
- [ ] Create an easy way to access layer objects stored in tile maps
- [ ] Support image layers
Please make an issue if I'm forgetting something important in this list
 
