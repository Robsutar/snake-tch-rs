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
