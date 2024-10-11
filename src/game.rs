use std::collections::HashMap;

use crate::RECT_SIZE;

use super::GlobalAssets;
use bevy::{
    prelude::*,
    sprite::{ColorMaterial, MaterialMesh2dBundle},
};
use rand::{thread_rng, Rng};

#[derive(Component, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct GridPos {
    x: i32,
    y: i32,
}

impl GridPos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    fn as_rect_translation(&self) -> Vec3 {
        Vec3::new(RECT_SIZE * self.x as f32, RECT_SIZE * self.y as f32, 0.0)
    }
}

#[derive(Component, PartialEq, Eq)]
pub enum SnakeOrientation {
    Up,
    Down,
    Left,
    Right,
}
impl SnakeOrientation {
    fn next(&self, pos: &GridPos) -> GridPos {
        match &self {
            SnakeOrientation::Up => GridPos::new(pos.x, pos.y + 1),
            SnakeOrientation::Down => GridPos::new(pos.x, pos.y - 1),
            SnakeOrientation::Left => GridPos::new(pos.x - 1, pos.y),
            SnakeOrientation::Right => GridPos::new(pos.x + 1, pos.y),
        }
    }

    pub fn pressed(keyboard_input: &Res<ButtonInput<KeyCode>>) -> Option<Self> {
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            Some(Self::Up)
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            Some(Self::Down)
        } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            Some(Self::Left)
        } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            Some(Self::Right)
        } else {
            None
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    pub fn left(&self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
            Self::Right => Self::Up,
        }
    }

    pub fn right(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
            Self::Right => Self::Down,
        }
    }
}

#[derive(Bundle)]
pub struct SnakeHeadBundle {
    orientation: SnakeOrientation,
    pos: GridPos,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

impl SnakeHeadBundle {
    fn new(pos: GridPos, assets: &GlobalAssets) -> Self {
        let (mesh, material) = assets.snake_head_mesh_material.clone();
        Self {
            orientation: SnakeOrientation::Up,
            pos,
            mesh: MaterialMesh2dBundle {
                mesh,
                material,
                transform: Transform::from_translation(pos.as_rect_translation()),
                ..default()
            },
        }
    }

    pub fn move_to(
        self_orientation: &mut SnakeOrientation,
        self_pos: &mut GridPos,
        self_transform: &mut Transform,
        apple_pos: &mut GridPos,
        apple_transform: &mut Transform,
        commands: &mut Commands,
        assets: &GlobalAssets,
        collider_query: &mut Query<&mut Transform, With<ColliderVariant>>,
        scene: &mut Scene,
        orientation: SnakeOrientation,
    ) -> Result<(), GridPos> {
        let new_head_pos = orientation.next(&self_pos);
        if scene.colliders.contains_key(&new_head_pos) {
            return Err(new_head_pos);
        } else if *apple_pos == new_head_pos {
            SnakeHeadBundle::increase_to_unchecked(
                self_orientation,
                self_pos,
                self_transform,
                commands,
                assets,
                scene,
                orientation,
            );

            let mut rng = thread_rng();

            let mut new_apple_pos = *apple_pos;
            while scene.colliders.contains_key(&new_apple_pos) || new_apple_pos == new_head_pos {
                new_apple_pos = GridPos::new(
                    rng.gen_range(assets.arena.min.x..=assets.arena.max.x),
                    rng.gen_range(assets.arena.min.y..=assets.arena.max.y),
                );
            }
            apply_translation_to(apple_pos, apple_transform, new_apple_pos);
            Ok(())
        } else {
            *self_orientation = orientation;
            let old_head_pos = apply_translation_to(self_pos, self_transform, new_head_pos);

            let last_body_part_id = scene
                .colliders
                .remove(&scene.snake_body_parts.remove(0))
                .unwrap();
            let mut last_body_part = collider_query.get_mut(last_body_part_id).unwrap();

            last_body_part.translation = old_head_pos.as_rect_translation();

            scene.colliders.insert(old_head_pos, last_body_part_id);
            scene.snake_body_parts.push(old_head_pos);
            Ok(())
        }
    }

    fn increase_to_unchecked(
        self_orientation: &mut SnakeOrientation,
        self_pos: &mut GridPos,
        self_transform: &mut Transform,
        commands: &mut Commands,
        assets: &GlobalAssets,
        scene: &mut Scene,
        orientation: SnakeOrientation,
    ) {
        let new_head_pos = orientation.next(&self_pos);
        *self_orientation = orientation;
        let old_head_pos = apply_translation_to(self_pos, self_transform, new_head_pos);
        scene.push_collider(
            commands,
            Collider::from_variant(ColliderVariant::SnakeBody, old_head_pos, &assets),
        );
        scene.snake_body_parts.push(old_head_pos);
    }
}

#[derive(Component)]
pub struct Apple;

#[derive(Bundle)]
pub struct AppleBundle {
    marker: Apple,
    pos: GridPos,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

impl AppleBundle {
    fn new(pos: GridPos, assets: &GlobalAssets) -> Self {
        let (mesh, material) = assets.apple_mesh_material.clone();
        Self {
            marker: Apple,
            pos,
            mesh: MaterialMesh2dBundle {
                mesh,
                material,
                transform: Transform::from_translation(pos.as_rect_translation()),
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub enum ColliderVariant {
    Wall,
    SnakeBody,
}

#[derive(Bundle)]
struct Collider {
    variant: ColliderVariant,
    pos: GridPos,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

impl Collider {
    fn from_variant(variant: ColliderVariant, pos: GridPos, assets: &GlobalAssets) -> Self {
        let ((mesh, material), variant) = match &variant {
            ColliderVariant::Wall => (assets.wall_mesh_material.clone(), ColliderVariant::Wall),
            ColliderVariant::SnakeBody => (
                assets.snake_body_mesh_material.clone(),
                ColliderVariant::SnakeBody,
            ),
        };

        Self {
            variant,
            pos,
            mesh: MaterialMesh2dBundle {
                mesh,
                material,
                transform: Transform::from_translation(pos.as_rect_translation()),
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub struct Scene {
    self_entity: Entity,
    pub snake_head: Entity,
    pub apple: Entity,
    snake_body_parts: Vec<GridPos>,
    colliders: HashMap<GridPos, Entity>,
}

#[derive(Bundle)]
struct SceneBundle {
    scene: Scene,

    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}

impl Scene {
    fn push_collider(&mut self, commands: &mut Commands, collider: Collider) {
        let pos = collider.pos.clone();
        let collider_id = commands.spawn(collider).id();
        if let Some(replaced) = self.colliders.insert(pos, collider_id) {
            panic!("Collider override. Replaced: {:?}", replaced)
        }
        commands.entity(self.self_entity).add_child(collider_id);
    }
}

fn apply_translation_to(
    self_pos: &mut GridPos,
    self_transform: &mut Transform,
    new_pos: GridPos,
) -> GridPos {
    let old_head_pos = std::mem::replace(self_pos, new_pos);
    self_transform.translation = self_pos.as_rect_translation();
    old_head_pos
}

pub fn init_scene(mut commands: Commands, assets: Res<GlobalAssets>) {
    let mut rng = thread_rng();

    let snake_head_id = commands.spawn_empty().id();
    let mut snake_head = SnakeHeadBundle::new(GridPos::new(0, 0), &assets);

    let apple_id = commands.spawn_empty().id();
    let apple = AppleBundle::new(
        GridPos::new(
            rng.gen_range(assets.arena.min.x..=assets.arena.max.x),
            rng.gen_range(assets.arena.min.y..=assets.arena.max.y),
        ),
        &assets,
    );

    let scene_id = commands.spawn_empty().id();
    let mut scene = Scene {
        self_entity: scene_id,
        snake_head: snake_head_id,
        apple: apple_id,
        snake_body_parts: Vec::new(),
        colliders: HashMap::new(),
    };

    let mut walls = Vec::new();
    for x in assets.arena.min.x - 1..=assets.arena.max.x + 1 {
        walls.push((x, assets.arena.min.y - 1));
        walls.push((x, assets.arena.max.y + 1));
    }
    for y in (assets.arena.min.y - 1 + 1)..assets.arena.max.y + 1 {
        walls.push((assets.arena.min.x - 1, y));
        walls.push((assets.arena.max.x + 1, y));
    }
    for (x, y) in walls {
        scene.push_collider(
            &mut commands,
            Collider::from_variant(ColliderVariant::Wall, GridPos::new(x, y), &assets),
        );
    }

    for _ in 0..3 {
        SnakeHeadBundle::increase_to_unchecked(
            &mut snake_head.orientation,
            &mut snake_head.pos,
            &mut snake_head.mesh.transform,
            &mut commands,
            &assets,
            &mut scene,
            SnakeOrientation::Up,
        );
    }

    commands.entity(snake_head_id).insert(snake_head);
    commands.entity(apple_id).insert(apple);
    commands
        .entity(scene_id)
        .insert(SceneBundle {
            scene,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            global_transform: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            view_visibility: Default::default(),
        })
        .add_child(snake_head_id)
        .add_child(apple_id);
}
