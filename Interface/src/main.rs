use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use config::states::AppState;
use config::navigation::NavStack;
use bevy_html_tailwind::HtmlTailwindPlugin; 


mod simulation_2d;
mod simulation_3d;
mod config;
mod ui;

use config::setup::SetupConfig;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {file_path: "../".into(),..default()}).set(ImagePlugin::default_nearest())
        )
        .add_plugins(HtmlTailwindPlugin { hot_reload: true })
        
        .init_state::<AppState>()
        .insert_resource(NavStack::default())
        .insert_resource(SetupConfig::default()) 
        
        .add_plugins(simulation_2d::Simulation2dPlugin)
        .add_plugins(simulation_3d::Simulation3dPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(config::ConfigPlugin)
        .run();
}