mod agent;
mod game;
mod model;
mod utils;

use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use game::{Apple, ColliderVariant, GridPos, Scene, SnakeHeadBundle, SnakeOrientation};
use tch::Device;

pub const RECT_SIZE: f32 = 25.0;
pub const DEVICE: Device = Device::Cpu;

#[derive(Resource)]
pub struct GlobalAssets {
    apple_mesh_material: (Mesh2dHandle, Handle<ColorMaterial>),
    wall_mesh_material: (Mesh2dHandle, Handle<ColorMaterial>),
    snake_body_mesh_material: (Mesh2dHandle, Handle<ColorMaterial>),
    snake_head_mesh_material: (Mesh2dHandle, Handle<ColorMaterial>),
    arena: IRect,
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, (init, game::init_scene).chain())
        .add_systems(Update, update)
        .run();
}

fn init(
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

    commands.spawn(Camera2dBundle::default());
}

fn update(
    mut contexts: EguiContexts,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    assets: Res<GlobalAssets>,
    mut scene_query: Query<&mut Scene>,
    mut snake_head_query: Query<
        (&mut SnakeOrientation, &mut GridPos, &mut Transform),
        (Without<ColliderVariant>, Without<Apple>),
    >,
    mut apple_query: Query<
        (&mut GridPos, &mut Transform),
        (
            With<Apple>,
            (Without<ColliderVariant>, Without<SnakeOrientation>),
        ),
    >,
    mut collider_query: Query<&mut Transform, With<ColliderVariant>>,
) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });

    if let Some(pressed_orientation) = SnakeOrientation::pressed(&keyboard_input) {
        let mut scene = scene_query.single_mut();
        let (mut snake_orientation, mut snake_pos, mut snake_transform) =
            snake_head_query.get_mut(scene.snake_head).unwrap();
        let (mut apple_pos, mut apple_transform) = apple_query.get_mut(scene.apple).unwrap();

        if pressed_orientation.opposite() != *snake_orientation {
            SnakeHeadBundle::move_to(
                &mut snake_orientation,
                &mut snake_pos,
                &mut snake_transform,
                &mut apple_pos,
                &mut apple_transform,
                &mut commands,
                &assets,
                &mut collider_query,
                &mut scene,
                pressed_orientation,
            )
            .expect("Collided");
        }
    }
}
