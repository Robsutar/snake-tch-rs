use std::collections::HashMap;

use crate::RECT_SIZE;

use super::GlobalAssets;
use bevy::{
    prelude::*,
    sprite::{ColorMaterial, MaterialMesh2dBundle},
};

#[derive(Component, Debug, Eq, PartialEq, Hash, Clone, Copy)]
struct GridPos {
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

#[derive(Component)]
enum SnakeOrientation {
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
            SnakeOrientation::Left => GridPos::new(pos.x + 1, pos.y),
            SnakeOrientation::Right => GridPos::new(pos.x - 1, pos.y),
        }
    }
}

#[derive(Bundle)]
struct SnakeHead {
    orientation: SnakeOrientation,
    pos: GridPos,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

impl SnakeHead {
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

    fn move_to(&mut self, new_head_pos: GridPos) -> GridPos {
        let old_head_pos = std::mem::replace(&mut self.pos, new_head_pos);
        self.mesh.transform.translation = self.pos.as_rect_translation();
        old_head_pos
    }
}

#[derive(Component)]
enum ColliderVariant {
    Apple,
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
            ColliderVariant::Apple => (assets.apple_mesh_material.clone(), ColliderVariant::Apple),
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
    snake_head: Entity,
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
    fn get_collider(&mut self, pos: &GridPos) -> Option<&Entity> {
        self.colliders.get(&pos)
    }

    fn push_collider(&mut self, commands: &mut Commands, collider: Collider) {
        let pos = collider.pos.clone();
        let collider_id = commands.spawn(collider).id();
        if let Some(replaced) = self.colliders.insert(pos, collider_id) {
            panic!("Collider override. Replaced: {:?}", replaced)
        }
        commands.entity(self.self_entity).add_child(collider_id);
    }
}

pub fn init_scene(mut commands: Commands, assets: Res<GlobalAssets>) {
    let mut snake_head = SnakeHead::new(GridPos::new(0, 0), &assets);
    let snake_head_id = commands.spawn_empty().id();
    let scene_id = commands.spawn_empty().id();

    let mut scene = Scene {
        self_entity: scene_id,
        snake_head: snake_head_id,
        snake_body_parts: Vec::new(),
        colliders: HashMap::new(),
    };

    let arena_len = 10;
    let arena_corners = (-arena_len, arena_len, arena_len, -arena_len);
    let mut walls = Vec::new();
    for x in arena_corners.0..=arena_corners.2 {
        walls.push((x, arena_corners.3));
        walls.push((x, arena_corners.1));
    }
    for y in (arena_corners.3 + 1)..arena_corners.1 {
        walls.push((arena_corners.0, y));
        walls.push((arena_corners.2, y));
    }
    for (x, y) in walls {
        scene.push_collider(
            &mut commands,
            Collider::from_variant(ColliderVariant::Apple, GridPos::new(x, y), &assets),
        );
    }

    for _ in 0..3 {
        let new_head_pos = snake_head.orientation.next(&snake_head.pos);
        if scene.colliders.contains_key(&new_head_pos) {
            panic!("Collision");
        }
        let old_head_pos = snake_head.move_to(new_head_pos);
        scene.push_collider(
            &mut commands,
            Collider::from_variant(ColliderVariant::SnakeBody, old_head_pos, &assets),
        );
        scene.snake_body_parts.push(old_head_pos);
    }

    commands.entity(snake_head_id).insert(snake_head);
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
        .add_child(snake_head_id);
}
