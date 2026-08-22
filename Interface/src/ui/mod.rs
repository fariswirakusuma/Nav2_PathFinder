use bevy::prelude::*;
use crate::config::states::AppState;
use crate::config::setup::ConfigCategory;

mod main_menu;
mod algo_menu;
mod overlay_panel;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveDropdown>()
           // Main Menu
           .add_systems(OnEnter(AppState::MainMenu), main_menu::setup_main_menu)
           .add_systems(Update, main_menu::handle_main_menu_buttons.run_if(in_state(AppState::MainMenu)))
           .add_systems(OnExit(AppState::MainMenu), cleanup_entity::<main_menu::MainMenuEntity>)
           
           // Algorithm Selection Menu
           .add_systems(OnEnter(AppState::AlgorithmSelection2D), algo_menu::setup_algo_menu)
           .add_systems(Update, (
               algo_menu::handle_dropdown_toggle,
               algo_menu::handle_selection_input,
               algo_menu::handle_start_button,
               algo_menu::update_selection_buttons,
               algo_menu::update_start_button_visual,
               algo_menu::handle_scroll,
               algo_menu::update_dropdown_visibility,
               algo_menu::update_dropdown_labels,
           ).run_if(in_state(AppState::AlgorithmSelection2D)))
           .add_systems(OnExit(AppState::AlgorithmSelection2D), cleanup_entity::<algo_menu::AlgoMenuEntity>)

           // Simulation Overlay Panel
           .add_systems(OnEnter(AppState::Sim2DRun), overlay_panel::setup_panel)
           .add_systems(
               Update,
               (
                   overlay_panel::update_panel_stats, 
                   overlay_panel::handle_reset_button, 
                   overlay_panel::manual_mouse_scroll
               ).run_if(in_state(AppState::Sim2DRun)),
           )
           .add_systems(OnExit(AppState::Sim2DRun), cleanup_entity::<overlay_panel::PanelEntity>);
    }
}

#[derive(Resource, Default)]
pub(crate) struct ActiveDropdown(pub(crate) Option<ConfigCategory>);

pub(crate) fn cleanup_entity<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() { 
        commands.entity(entity).despawn(); 
    }
}

pub(crate) fn get_files(path: &str, ext: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    let name = entry.file_name().into_string().unwrap_or_default();
                    if name.ends_with(ext) { files.push(name); }
                }
            }
        }
    }
    files.sort();
    files
}