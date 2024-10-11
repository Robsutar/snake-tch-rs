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
