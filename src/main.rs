mod game;

use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use game::{ColliderVariant, GridPos, Scene, SnakeHeadBundle, SnakeOrientation};

pub const RECT_SIZE: f32 = 25.0;

#[derive(Resource)]
pub struct GlobalAssets {
    apple_mesh_material: (Mesh2dHandle, Handle<ColorMaterial>),
    wall_mesh_material: (Mesh2dHandle, Handle<ColorMaterial>),
    snake_body_mesh_material: (Mesh2dHandle, Handle<ColorMaterial>),
    snake_head_mesh_material: (Mesh2dHandle, Handle<ColorMaterial>),
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

    commands.insert_resource(GlobalAssets {
        apple_mesh_material: (
            Mesh2dHandle(meshes.add(generic_rect)),
            materials.add(Color::srgb(1.0, 0.0, 1.0)),
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
    });

    commands.spawn(Camera2dBundle::default());
}

fn update(
    mut contexts: EguiContexts,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scene_query: Query<&mut Scene>,
    mut snake_head_query: Query<
        (&mut SnakeOrientation, &mut GridPos, &mut Transform),
        Without<ColliderVariant>,
    >,
    mut collider_query: Query<&mut Transform, With<ColliderVariant>>,
) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });

    if let Some(pressed_orientation) = SnakeOrientation::pressed(&keyboard_input) {
        let mut scene = scene_query.single_mut();
        let (mut orientation, mut pos, mut transform) =
            snake_head_query.get_mut(scene.snake_head).unwrap();

        if pressed_orientation.opposite() != *orientation {
            SnakeHeadBundle::move_to(
                &mut orientation,
                &mut pos,
                &mut transform,
                &mut collider_query,
                &mut scene,
                pressed_orientation,
            )
            .unwrap();
        }
    }
}
