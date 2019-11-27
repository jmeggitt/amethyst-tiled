use amethyst::ecs::{SystemData, ReadExpect};
use amethyst::assets::Loader;


pub struct TiledLoader<'a> {
    loader: ReadExpect<'a, Loader>,
}


