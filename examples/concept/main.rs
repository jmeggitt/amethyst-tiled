use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystemDesc},
    core::{math::Vector3, Transform, TransformBundle},
    ecs::{Entities, Join, Read, ReadStorage, System, WriteStorage},
    input::{InputBundle, InputHandler, StringBindings},
    prelude::*,
    renderer::{
        camera::{ActiveCamera, Camera},
        types::DefaultBackend,
        RenderToWindow, RenderingBundle,
    },
    tiles::{FlatEncoder, RenderTiles2D},
    utils::application_root_dir,
    window::ScreenDimensions,
};

use amethyst_tiled::{TileGid, TileMapPrefab, TiledFormat};

#[derive(Default)]
pub struct CameraMovementSystem;
impl<'s> System<'s> for CameraMovementSystem {
    type SystemData = (
        Read<'s, ActiveCamera>,
        Entities<'s>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (active_camera, entities, cameras, mut transforms, input): Self::SystemData) {
        let x_move = input.axis_value("camera_x").unwrap();
        let y_move = input.axis_value("camera_y").unwrap();
        let z_move = input.axis_value("camera_z").unwrap();
        let z_move_scale = input.axis_value("camera_scale").unwrap();

        if x_move != 0.0 || y_move != 0.0 || z_move != 0.0 || z_move_scale != 0.0 {
            let mut camera_join = (&cameras, &mut transforms).join();
            if let Some((_, camera_transform)) = active_camera
                .entity
                .and_then(|a| camera_join.get(a, &entities))
                .or_else(|| camera_join.next())
            {
                camera_transform.prepend_translation_x(x_move * 5.0);
                camera_transform.prepend_translation_y(y_move * 5.0);
                camera_transform.prepend_translation_z(z_move);

                let z_scale = 0.01 * z_move_scale;
                let scale = camera_transform.scale();
                let scale = Vector3::new(scale.x + z_scale, scale.y + z_scale, scale.z + z_scale);
                camera_transform.set_scale(scale);
            }
        }
    }
}

struct Example;
impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        // Init camera
        world
            .create_entity()
            .with(Transform::from(Vector3::new(0.0, 0.0, 5.0)))
            .with(Camera::standard_2d(width, height))
            .build();

        // Use a prefab loader to get the tiled .tmx file loaded
        let prefab_handle = world.exec(|loader: PrefabLoader<'_, TileMapPrefab>| {
            loader.load("prefab/example_map.tmx", TiledFormat, ())
        });

        let _map_entity = world
            .create_entity()
            .with(prefab_handle)
            .with(Transform::default())
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(Default::default())
        .level_for("amethyst_tiles", log::LevelFilter::Warn)
        .start();

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("examples/assets");
    let display_config_path = app_root.join("examples/concept/resources/display_config.ron");

    let game_data = GameDataBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<TileMapPrefab>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<StringBindings>::new()
                .with_bindings_from_file("examples/concept/resources/input.ron")?,
        )?
        .with(CameraMovementSystem::default(), "movement", &[])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear([1.0; 4]),
                )
                .with_plugin(RenderTiles2D::<TileGid, FlatEncoder>::default()),
        )?;

    let mut game = Application::build(assets_directory, Example)?.build(game_data)?;
    game.run();
    Ok(())
}
