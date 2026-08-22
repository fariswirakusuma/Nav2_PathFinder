use bevy::prelude::*;

pub mod navigation;
pub mod setup;
pub mod states;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // Register resources, states, or systems exported by child modules
        app.init_state::<states::AppState>()
           .init_resource::<setup::SetupConfig>()
           .init_resource::<navigation::NavStack>();
    }
}