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


#[derive(Bundle)]
struct SnakeHead {
    pos: GridPos,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
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
