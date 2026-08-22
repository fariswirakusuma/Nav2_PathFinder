use bevy::prelude::*;
use bevy::camera::*;
pub use wgpu;
// Declare the sub-file inside this directory

pub mod urdf_loader; 
pub mod robot;
pub mod camera;

pub struct Simulation3dPlugin;

impl Plugin for Simulation3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_3d_world)
           .add_systems(Update, urdf_loader::parse_and_animate_robot);
    }
}



fn setup_3d_world(mut commands: Commands) {
    // Spawn 3D camera, directional lights, and origin ground plane
}