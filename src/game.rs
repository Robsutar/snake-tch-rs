use std::collections::HashMap;

use crate::{MaterialMesh, RECT_SIZE};

use super::GlobalAssets;
use bevy::{
    prelude::*,
    sprite::{ColorMaterial, MaterialMesh2dBundle},
};
use rand::{thread_rng, Rng};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    fn as_rect_translation(&self) -> Vec3 {
        Vec3::new(RECT_SIZE * self.x as f32, RECT_SIZE * self.y as f32, 0.0)
    }
}

pub struct GridEntity {
    pub id: Entity,
    pub pos: GridPos,
}
impl GridEntity {
    fn new(id: Entity, pos: GridPos) -> Self {
        Self { id, pos }
    }
    fn create_raw_bundle(
        pos: &GridPos,
        mesh_material: &MaterialMesh,
    ) -> MaterialMesh2dBundle<ColorMaterial> {
        let (mesh, material) = mesh_material.clone();
        MaterialMesh2dBundle {
            mesh,
            material,
            transform: Transform::from_translation(pos.as_rect_translation()),
            ..default()
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
}

#[derive(PartialEq, Eq)]
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

#[derive(Component)]
pub struct SnakeHeadMarker;
impl SnakeHeadMarker {
    fn create_bundle(assets: &GlobalAssets, pos: &GridPos) -> impl Bundle {
        (
            SnakeHeadMarker,
            GridEntity::create_raw_bundle(&pos, &assets.snake_head_mesh_material),
        )
    }
}

pub struct SnakeHead {
    pub orientation: SnakeOrientation,
    pub ge: GridEntity,
}

impl SnakeHead {
    pub fn move_to(
        self_transform: &mut Transform,
        apple_transform: &mut Transform,
        commands: &mut Commands,
        assets: &GlobalAssets,
        collider_query: &mut Query<&mut Transform, With<ColliderMarker>>,
        scene: &mut Scene,
        orientation: SnakeOrientation,
    ) -> Result<(), GridPos> {
        let new_head_pos = orientation.next(&scene.snake_head.ge.pos);
        if scene.colliders.contains_key(&new_head_pos) {
            return Err(new_head_pos);
        } else if scene.apple.ge.pos == new_head_pos {
            Self::increase_to_unchecked(self_transform, commands, assets, scene, orientation);

            let apple_pos = &mut scene.apple.ge.pos;

            let mut rng = thread_rng();

            let mut new_apple_pos = *apple_pos;
            while scene.colliders.contains_key(&new_apple_pos) || new_apple_pos == new_head_pos {
                new_apple_pos = GridPos::new(
                    rng.gen_range(assets.arena.min.x..=assets.arena.max.x),
                    rng.gen_range(assets.arena.min.y..=assets.arena.max.y),
                );
            }
            GridEntity::apply_translation_to(apple_pos, apple_transform, new_apple_pos);
            Ok(())
        } else {
            let self_pos = &mut scene.snake_head.ge.pos;
            scene.snake_head.orientation = orientation;

            let old_head_pos =
                GridEntity::apply_translation_to(self_pos, self_transform, new_head_pos);

            let last_body_part_id = scene
                .colliders
                .remove(&scene.snake_body_parts.remove(0))
                .unwrap();
            let mut last_body_part = collider_query.get_mut(last_body_part_id.ge.id).unwrap();

            last_body_part.translation = old_head_pos.as_rect_translation();

            scene.colliders.insert(old_head_pos, last_body_part_id);
            scene.snake_body_parts.push(old_head_pos);
            Ok(())
        }
    }

    fn increase_to_unchecked(
        self_transform: &mut Transform,
        commands: &mut Commands,
        assets: &GlobalAssets,
        scene: &mut Scene,
        orientation: SnakeOrientation,
    ) {
        let new_head_pos = orientation.next(&scene.snake_head.ge.pos);
        scene.snake_head.orientation = orientation;
        let old_head_pos = GridEntity::apply_translation_to(
            &mut scene.snake_head.ge.pos,
            self_transform,
            new_head_pos,
        );
        scene.push_collider(commands, &assets, ColliderVariant::SnakeBody, old_head_pos);
        scene.snake_body_parts.push(old_head_pos);
    }
}

#[derive(Component)]
pub struct AppleMarker;
impl AppleMarker {
    fn create_bundle(assets: &GlobalAssets, pos: &GridPos) -> impl Bundle {
        (
            AppleMarker,
            GridEntity::create_raw_bundle(&pos, &assets.apple_mesh_material),
        )
    }
}

pub struct Apple {
    pub ge: GridEntity,
}

pub enum ColliderVariant {
    Wall,
    SnakeBody,
}
impl ColliderVariant {
    fn mesh_material<'a>(&self, assets: &'a GlobalAssets) -> &'a MaterialMesh {
        match &self {
            ColliderVariant::Wall => &assets.wall_mesh_material,
            ColliderVariant::SnakeBody => &assets.snake_body_mesh_material,
        }
    }
}

#[derive(Component)]
pub struct ColliderMarker;
impl ColliderMarker {
    fn create_bundle(
        variant: &ColliderVariant,
        assets: &GlobalAssets,
        pos: &GridPos,
    ) -> impl Bundle {
        (
            ColliderMarker,
            GridEntity::create_raw_bundle(&pos, &variant.mesh_material(&assets)),
        )
    }
}

struct Collider {
    variant: ColliderVariant,
    pub ge: GridEntity,
}

#[derive(Component)]
pub struct Scene {
    self_entity: Entity,
    pub snake_head: SnakeHead,
    snake_body_parts: Vec<GridPos>,
    pub apple: Apple,
    colliders: HashMap<GridPos, Collider>,
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
    fn push_collider(
        &mut self,
        commands: &mut Commands,
        assets: &GlobalAssets,
        variant: ColliderVariant,
        pos: GridPos,
    ) {
        let collider_id = commands
            .spawn(ColliderMarker::create_bundle(&variant, &assets, &pos))
            .id();
        if let Some(replaced) = self.colliders.insert(
            pos,
            Collider {
                variant,
                ge: GridEntity::new(collider_id, pos),
            },
        ) {
            panic!("Collider override. Replaced: {:?}", replaced.ge.pos)
        }
        commands.entity(self.self_entity).add_child(collider_id);
    }

    pub fn is_collision(&self, pos: &GridPos) -> bool {
        self.colliders.contains_key(pos)
    }

    pub fn reset(
        &mut self,
        commands: &mut Commands,
        assets: &GlobalAssets,
        snake_head_transform: &mut Transform,
        apple_transform: &mut Transform,
    ) {
        let mut rng = thread_rng();

        for body_pos in &self.snake_body_parts {
            let body = self.colliders.remove(body_pos).unwrap();
            commands.entity(body.ge.id).despawn();
        }
        self.snake_body_parts.clear();

        self.snake_head.orientation = SnakeOrientation::Up;

        self.snake_head.ge.pos = GridPos::new(0, 0);
        for _ in 0..3 {
            let snake_head = &mut self.snake_head;
            let new_head_pos = snake_head.orientation.next(&snake_head.ge.pos);
            let old_head_pos = std::mem::replace(&mut snake_head.ge.pos, new_head_pos);
            self.push_collider(commands, &assets, ColliderVariant::SnakeBody, old_head_pos);
            self.snake_body_parts.push(old_head_pos);
        }

        let snake_pos = self.snake_head.ge.pos;
        GridEntity::apply_translation_to(
            &mut self.snake_head.ge.pos,
            snake_head_transform,
            snake_pos,
        );
        GridEntity::apply_translation_to(
            &mut self.apple.ge.pos,
            apple_transform,
            GridPos::new(
                rng.gen_range(assets.arena.min.x..=assets.arena.max.x),
                rng.gen_range(assets.arena.min.y..=assets.arena.max.y),
            ),
        );
    }
}

pub fn init_scene(mut commands: Commands, assets: Res<GlobalAssets>) {
    let mut rng = thread_rng();

    let snake_head_id = commands.spawn_empty().id();
    let snake_head = SnakeHead {
        orientation: SnakeOrientation::Up,
        ge: GridEntity::new(snake_head_id, GridPos::new(0, 0)),
    };

    let apple_id = commands.spawn_empty().id();
    let apple = Apple {
        ge: GridEntity::new(
            apple_id,
            GridPos::new(
                rng.gen_range(assets.arena.min.x..=assets.arena.max.x),
                rng.gen_range(assets.arena.min.y..=assets.arena.max.y),
            ),
        ),
    };

    let scene_id = commands.spawn_empty().id();
    let mut scene = Scene {
        self_entity: scene_id,
        snake_head,
        apple,
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
            &assets,
            ColliderVariant::Wall,
            GridPos::new(x, y),
        );
    }

    for _ in 0..3 {
        let snake_head = &mut scene.snake_head;
        let new_head_pos = snake_head.orientation.next(&snake_head.ge.pos);
        let old_head_pos = std::mem::replace(&mut snake_head.ge.pos, new_head_pos);
        scene.push_collider(
            &mut commands,
            &assets,
            ColliderVariant::SnakeBody,
            old_head_pos,
        );
        scene.snake_body_parts.push(old_head_pos);
    }

    commands
        .entity(snake_head_id)
        .insert(SnakeHeadMarker::create_bundle(
            &assets,
            &scene.snake_head.ge.pos,
        ));
    commands
        .entity(apple_id)
        .insert(AppleMarker::create_bundle(&assets, &scene.apple.ge.pos));
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
