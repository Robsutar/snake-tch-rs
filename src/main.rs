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
use egui_plot::{AxisHints, Legend, Line, Plot, PlotPoints};
use game::{
    init_scene, AppleMarker, ColliderMarker, PlayerStepAction, PlayerStepResult, Scene,
    SnakeHeadMarker, SnakeOrientation,
};
use model::{Snapshot, ACTION_SIZE};
use tch::Device;

pub const RECT_SIZE: f32 = 5.0;
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
    plot_scores: Vec<[f64; 2]>,
    plot_mean_scores: Vec<[f64; 2]>,
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

#[derive(Component)]
struct AiControllerDependent;

fn main() {
    let use_human_controller = false;

    let mut app = App::default();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(EguiPlugin);
    app.add_systems(Update, ui_info_update);
    if use_human_controller {
        app.add_systems(Startup, (init_assets, init_human).chain());
        app.add_systems(Update, human_update);
    } else {
        app.add_systems(Startup, (init_assets, init_ai).chain());
        app.add_systems(Update, ai_update);
    }
    app.run();
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

    let scene_id = init_scene(&mut commands, &assets, Transform::default());
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

    commands.spawn(AiController {
        plot_scores: Vec::new(),
        plot_mean_scores: Vec::new(),
        total_score: 0,
        record: 0,
        agent: Mutex::new(Agent::load_if_exists("model.ot")),
    });

    let margin = 1.1;
    let x_count = 5;
    let y_count = 3;

    let mut x_index = -x_count as f32 / 2.0 - 0.5;
    for _ in 0..x_count {
        x_index += 1.0;

        let mut y_index = -y_count as f32 / 2.0 - 0.5;
        for _ in 0..y_count {
            y_index += 1.0;

            let scene_id = init_scene(
                &mut commands,
                &assets,
                Transform::from_xyz(
                    RECT_SIZE * ARENA.width() as f32 * x_index * margin,
                    RECT_SIZE * ARENA.height() as f32 * y_index * margin,
                    0.0,
                ),
            );
            commands.entity(scene_id).insert(AiControllerDependent);
        }
    }
}

fn ai_update(
    mut commands: Commands,
    mut ctx: EguiContexts,
    assets: Res<GlobalAssets>,
    mut controller_query: Query<&mut AiController>,
    mut scene_query: Query<&mut Scene, With<AiControllerDependent>>,
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
    let mut controller = controller_query.single_mut();
    let mut agent = controller.agent.lock().unwrap();

    let mut done_scores = Vec::new();
    let mut best_score = 0;

    let scene_query = scene_query.iter_mut();
    let snapshots_stored = scene_query.len();
    for mut scene in scene_query {
        let mut snake_head_transform = snake_head_query.get_mut(scene.snake_head.ge.id).unwrap();
        let mut apple_transform = apple_query.get_mut(scene.apple.ge.id).unwrap();

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

        agent.remember(snapshot);

        if done {
            scene.reset(
                &mut commands,
                &assets,
                &mut snake_head_transform,
                &mut apple_transform,
            );

            done_scores.push(score);
            if score > best_score {
                best_score = score;
            }
        }
    }

    agent.train_with_last(snapshots_stored);

    let old_n_games = agent.n_games;
    agent.n_games += done_scores.len();

    if best_score != 0 {
        agent.train_long_memory();

        if best_score > controller.record {
            agent.save("model.ot").unwrap();
        }
    }
    drop(agent);

    if best_score > controller.record {
        controller.record = best_score;
    }

    for (i, score) in done_scores.into_iter().enumerate() {
        let game_number = (old_n_games + i + 1) as f64;

        controller.total_score += score;
        let mean_score = controller.total_score as f64 / game_number;

        controller.plot_scores.push([game_number, score as f64]);
        controller.plot_mean_scores.push([game_number, mean_score]);
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx.ctx_mut(), |ui| {
            Plot::new("main_plot")
                .show_background(false)
                .show_grid(false)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false)
                .show_x(false)
                .show_y(false)
                .legend(Legend::default().position(egui_plot::Corner::LeftTop))
                .custom_x_axes(vec![AxisHints::new_x().label("Number of Games")])
                .show(ui, |plot_ui| {
                    plot_ui.line(
                        Line::new(PlotPoints::new(controller.plot_scores.clone())).name("Scores"),
                    );
                    plot_ui.line(
                        Line::new(PlotPoints::new(controller.plot_mean_scores.clone()))
                            .name("Mean Scores"),
                    );
                });
        });
}
