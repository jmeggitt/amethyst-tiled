use amethyst::assets::Loader;
use amethyst::ecs::{ReadExpect, SystemData};

pub struct TiledLoader<'a> {
    loader: ReadExpect<'a, Loader>,
}
