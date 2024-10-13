mod agent;
mod game;
mod model;
mod utils;

use std::sync::Mutex;

use agent::Agent;
use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_egui::{
    egui::{self, Id},
    EguiContexts, EguiPlugin,
};
use game::{
    init_scene, AppleMarker, ColliderMarker, PlayerStepAction, PlayerStepResult, Scene,
    SnakeHeadMarker, SnakeOrientation,
};
use model::{Snapshot, ACTION_SIZE};
use tch::Device;

pub const RECT_SIZE: f32 = 10.0;
pub const DEVICE: Device = Device::Cpu;
pub const ARENA: IRect = IRect {
    min: IVec2 { x: -20, y: -20 },
    max: IVec2 { x: 20, y: 20 },
};

pub type MaterialMesh = (Mesh2dHandle, Handle<ColorMaterial>);
pub type DType = f32;

#[derive(Resource)]
pub struct GlobalAssets {
    apple_mesh_material: MaterialMesh,
    wall_mesh_material: MaterialMesh,
    snake_body_mesh_material: MaterialMesh,
    snake_head_mesh_material: MaterialMesh,
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

#[derive(Component)]
struct AiController {
    plot_scores: Vec<usize>,
    plot_mean_scores: Vec<f64>,
    total_score: usize,
    record: usize,
    agent: Mutex<Agent>,
}
impl AiController {
    fn action(raw: &[DType; ACTION_SIZE]) -> PlayerStepAction {
        if raw[0] == 1.0 {
            PlayerStepAction::Forward
        } else if raw[1] == 1.0 {
            PlayerStepAction::Left
        } else {
            PlayerStepAction::Right
        }
    }
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, (init_assets, init_ai).chain())
        .add_systems(Update, ai_update)
        .add_systems(Update, ui_info_update)
        .run();
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let generic_rect = Rectangle::new(RECT_SIZE * 0.8, RECT_SIZE * 0.8);

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
    });
}

fn ui_info_update(
    mut ctx: EguiContexts,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    scene_query: Query<(&Scene, &GlobalTransform)>,
) {
    let mut area_id: u64 = 1;
    let (camera, camera_transform) = camera_query.single();
    for (scene, scene_transform) in scene_query.iter() {
        if let Some(point) =
            camera.world_to_viewport(camera_transform, scene_transform.translation())
        {
            area_id += 1;
            egui::Area::new(Id::new(area_id))
                .fixed_pos([
                    point.x + ARENA.min.x as f32 * RECT_SIZE,
                    point.y + ARENA.min.y as f32 * RECT_SIZE,
                ])
                .show(ctx.ctx_mut(), |ui| {
                    let frame = egui::Frame::none();

                    frame.show(ui, |ui| {
                        ui.set_min_width(200.0);
                        ui.label(format!("Score: {}", scene.punctuation));
                        ui.label(format!("Frame: {}", scene.frame_iteration));
                    });
                });
        }
    }
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

fn init_ai(mut commands: Commands, assets: Res<GlobalAssets>) {
    commands.spawn(Camera2dBundle::default());

    let scene_id = init_scene(&mut commands, &assets);
    commands.entity(scene_id).insert(AiController {
        plot_scores: Vec::new(),
        plot_mean_scores: Vec::new(),
        total_score: 0,
        record: 0,
        agent: Mutex::new(Agent::load_if_exists("model.ot")),
    });
}

fn ai_update(
    mut commands: Commands,
    assets: Res<GlobalAssets>,
    mut scene_query: Query<(&mut Scene, &mut AiController)>,
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
    if true {
        let (mut scene, mut controller) = scene_query.single_mut();
        let mut snake_head_transform = snake_head_query.get_mut(scene.snake_head.ge.id).unwrap();
        let mut apple_transform = apple_query.get_mut(scene.apple.ge.id).unwrap();

        let mut agent = controller.agent.lock().unwrap();

        let state_old = Agent::get_state(&scene);

        let final_move = agent.get_action(&state_old);

        let (reward, done, score) = match scene.play_step(
            &mut commands,
            &assets,
            &mut collider_query,
            &mut snake_head_transform,
            &mut apple_transform,
            AiController::action(&final_move),
        ) {
            PlayerStepResult::Nothing => {
                if scene.frame_iteration > 100 * scene.snake_len() {
                    (-10.0, true, scene.punctuation)
                } else {
                    (0.0, false, scene.punctuation)
                }
            }
            PlayerStepResult::AppleEaten => (10.0, false, scene.punctuation),
            PlayerStepResult::Collision => {
                scene.frame_iteration += 1;
                (-10.0, true, scene.punctuation)
            }
        };

        let state_new = Agent::get_state(&scene);

        let snapshot = Snapshot {
            state: state_old,
            action: final_move,
            reward,
            next_state: state_new,
            done,
        };

        agent.train_short_memory(&snapshot);

        agent.remember(snapshot);

        if done {
            scene.reset(
                &mut commands,
                &assets,
                &mut snake_head_transform,
                &mut apple_transform,
            );
            agent.n_games += 1;
            agent.train_long_memory();

            let n_games = agent.n_games;
            if score > controller.record {
                agent.save("model.ot").unwrap();
                drop(agent);
                controller.record = score;
            } else {
                drop(agent);
            }

            println!(
                "Game: {:?}, Score: {:?}, Record: {:?}",
                n_games, score, controller.record
            );

            controller.plot_scores.push(score);
            controller.total_score += score;
            let mean_score = controller.total_score as f64 / n_games as f64;
            controller.plot_mean_scores.push(mean_score);
        }
    }
}
