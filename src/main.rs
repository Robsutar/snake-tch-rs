mod agent;
mod game;
mod model;
mod utils;

use bevy::{prelude::*, sprite::Mesh2dHandle};
use game::{
    init_scene, AppleMarker, ColliderMarker, Scene, SnakeHead, SnakeHeadMarker, SnakeOrientation,
};
use tch::Device;

pub const RECT_SIZE: f32 = 10.0;
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
struct HumanController {
    up_command: KeyCode,
    down_command: KeyCode,
    left_command: KeyCode,
    right_command: KeyCode,
}
impl HumanController {
    pub fn orientation_pressed(
        &self,
        keyboard_input: &Res<ButtonInput<KeyCode>>,
    ) -> Option<SnakeOrientation> {
        if keyboard_input.just_pressed(self.up_command) {
            Some(SnakeOrientation::Up)
        } else if keyboard_input.just_pressed(self.down_command) {
            Some(SnakeOrientation::Down)
        } else if keyboard_input.just_pressed(self.left_command) {
            Some(SnakeOrientation::Left)
        } else if keyboard_input.just_pressed(self.right_command) {
            Some(SnakeOrientation::Right)
        } else {
            None
        }
    }
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (init_assets, init_human).chain())
        .add_systems(Update, human_update)
        .run();
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let generic_rect = Rectangle::new(RECT_SIZE * 0.8, RECT_SIZE * 0.8);

    let arena_len = 20;
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

fn init_human(mut commands: Commands, assets: Res<GlobalAssets>) {
    commands.spawn(Camera2dBundle::default());

    let scene_id = init_scene(&mut commands, &assets);
    commands.entity(scene_id).insert(HumanController {
        up_command: KeyCode::ArrowUp,
        down_command: KeyCode::ArrowDown,
        left_command: KeyCode::ArrowLeft,
        right_command: KeyCode::ArrowRight,
    });
}

fn human_update(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    assets: Res<GlobalAssets>,
    mut scene_query: Query<(&mut Scene, &HumanController)>,
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
    let (mut scene, controller) = scene_query.single_mut();
    if let Some(pressed_orientation) = controller.orientation_pressed(&keyboard_input) {
        let mut snake_head_transform = snake_head_query.get_mut(scene.snake_head.ge.id).unwrap();
        let mut apple_transform = apple_query.get_mut(scene.apple.ge.id).unwrap();

        if let Some(action) = scene
            .snake_head
            .orientation
            .apply_to_action(&pressed_orientation)
        {
            match scene.play_step(
                &mut commands,
                &assets,
                &mut collider_query,
                &mut snake_head_transform,
                &mut apple_transform,
                action,
            ) {
                PlayerStepResult::Nothing => {}
                PlayerStepResult::AppleEaten => {
                    println!("Apple eaten! Punctuation: {:?}", scene.punctuation)
                }
                PlayerStepResult::Collision => {
                    println!("Collision! Game reset");
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
}
