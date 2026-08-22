use bevy::prelude::*;
use crate::simulation_2d::Point;

#[derive(Resource, Default, Debug)]
pub struct SetupConfig {
    pub algorithm: String,
    pub urdf_model: String,
    pub map_name: String,
    pub start_pos: Option<Point>,
    pub goal_pos: Option<Point>,
    pub interaction_mode: InteractionMode, 
}

#[derive(Default, PartialEq, Clone, Copy, Debug)] 
pub enum InteractionMode { #[default] Obstacle, Start, Goal }

#[derive(Component)]
pub struct SelectionItem {
    pub category: ConfigCategory,
    pub value: String,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ConfigCategory { Algorithm, Map, Urdf }