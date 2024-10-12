mod agent;
mod game;
mod model;
mod utils;

use bevy::{prelude::*, sprite::Mesh2dHandle};
use game::{
    init_scene, AppleMarker, ColliderMarker, Scene, SnakeHead, SnakeHeadMarker, SnakeOrientation,
};
use tch::Device;

pub const RECT_SIZE: f32 = 25.0;
pub const DEVICE: Device = Device::Cpu;

pub type MaterialMesh = (Mesh2dHandle, Handle<ColorMaterial>);

#[derive(Resource)]
pub struct GlobalAssets {
    apple_mesh_material: MaterialMesh,
    wall_mesh_material: MaterialMesh,
    snake_body_mesh_material: MaterialMesh,
    snake_head_mesh_material: MaterialMesh,
    arena: IRect,
}

#[derive(Component)]
struct HumanController;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (init_assets, init).chain())
        .add_systems(Update, human_update)
        .run();
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let generic_rect = Rectangle::new(RECT_SIZE * 0.8, RECT_SIZE * 0.8);

    let arena_len = 10;
    commands.insert_resource(GlobalAssets {
        apple_mesh_material: (
            Mesh2dHandle(meshes.add(generic_rect)),
            materials.add(Color::srgb(1.0, 0.0, 0.0)),
        ),
        wall_mesh_material: (
            Mesh2dHandle(meshes.add(generic_rect)),
            materials.add(Color::srgb(0.0, 0.0, 1.0)),
        ),
        snake_body_mesh_material: (
            Mesh2dHandle(meshes.add(generic_rect)),
            materials.add(Color::srgb(0.0, 1.0, 1.0)),
        ),
        snake_head_mesh_material: (
            Mesh2dHandle(meshes.add(generic_rect)),
            materials.add(Color::srgb(0.0, 1.0, 0.0)),
        ),
        arena: IRect::new(-arena_len, -arena_len, arena_len, arena_len),
    });
}

fn init(mut commands: Commands, assets: Res<GlobalAssets>) {
    commands.spawn(Camera2dBundle::default());

    let human_scene_id = init_scene(&mut commands, &assets);
    commands.entity(human_scene_id).insert(HumanController);
}

fn human_update(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    assets: Res<GlobalAssets>,
    mut scene_query: Query<&mut Scene, With<HumanController>>,
    mut snake_head_query: Query<
        &mut Transform,
        (
            With<SnakeHeadMarker>,
            Without<AppleMarker>,
            Without<ColliderMarker>,
        ),
    >,
    mut apple_query: Query<
        &mut Transform,
        (
            With<AppleMarker>,
            Without<SnakeHeadMarker>,
            Without<ColliderMarker>,
        ),
    >,
    mut collider_query: Query<&mut Transform, With<ColliderMarker>>,
) {
    if let Some(pressed_orientation) = SnakeOrientation::pressed(&keyboard_input) {
        let mut scene = scene_query.single_mut();
        let mut snake_head_transform = snake_head_query.get_mut(scene.snake_head.ge.id).unwrap();
        let mut apple_transform = apple_query.get_mut(scene.apple.ge.id).unwrap();

        if pressed_orientation.opposite() != scene.snake_head.orientation {
            if SnakeHead::move_to(
                &mut snake_head_transform,
                &mut apple_transform,
                &mut commands,
                &assets,
                &mut collider_query,
                &mut scene,
                pressed_orientation,
            )
            .is_err()
            {
                scene.reset(
                    &mut commands,
                    &assets,
                    &mut snake_head_transform,
                    &mut apple_transform,
                );
            }
        }
    }
}
