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
}

#[derive(Bundle)]
struct SnakeHead {
    pos: GridPos,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

impl SnakeHead {
    fn new(pos: GridPos, assets: &GlobalAssets) -> Self {
        let (mesh, material) = assets.snake_head_mesh_material.clone();
        Self {
            pos,
            mesh: MaterialMesh2dBundle {
                mesh,
                material,
                transform: Transform::from_xyz(
                    RECT_SIZE * pos.x as f32,
                    RECT_SIZE * pos.y as f32,
                    0.0,
                ),
                ..default()
            },
        }
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
                transform: Transform::from_xyz(
                    RECT_SIZE * pos.x as f32,
                    RECT_SIZE * pos.y as f32,
                    0.0,
                ),
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub struct Scene {
    self_entity: Entity,
    snake_head: Entity,
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
    let snake_head = commands
        .spawn(SnakeHead::new(GridPos::new(0, 0), &assets))
        .id();
    let scene_id = commands.spawn_empty().id();

    let mut scene = Scene {
        self_entity: scene_id,
        snake_head,
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
        .add_child(snake_head);
}
