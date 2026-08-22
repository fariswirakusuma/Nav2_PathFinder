use bevy::prelude::*;
use crate::config::states::AppState;

#[derive(Component)]
pub(crate) struct MainMenuEntity;

#[derive(Component)]
enum MenuAction { Play2D, Play3D }

pub(crate) fn setup_main_menu(mut commands: Commands) {
    commands.spawn((Camera2d, MainMenuEntity));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(25.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.12)),
        MainMenuEntity,
    )).with_children(|parent| {
        parent.spawn((Text::new("BawalPathFinder"), TextFont { font_size: 45.0, ..default() }, TextColor(Color::WHITE)));
        
        parent.spawn((
            Button,
            Node { width: Val::Px(250.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
            BackgroundColor(Color::srgb(0.2, 0.4, 0.8)), BorderColor::all(Color::srgb(0.4, 0.6, 1.0)), MenuAction::Play2D,
        )).with_children(|btn| { btn.spawn((Text::new("2D SIMULATION"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
        
        parent.spawn((
            Button,
            Node { width: Val::Px(250.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
            BackgroundColor(Color::srgb(0.2, 0.8, 0.4)), BorderColor::all(Color::srgb(0.4, 1.0, 0.6)), MenuAction::Play3D,
        )).with_children(|btn| { btn.spawn((Text::new("3D SIMULATION"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
    });
}

pub(crate) fn handle_main_menu_buttons(
    mut next_state: ResMut<NextState<AppState>>, 
    interaction_query: Query<(&Interaction, &MenuAction), (Changed<Interaction>, With<Button>)>
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action { 
                MenuAction::Play2D => next_state.set(AppState::AlgorithmSelection2D), 
                MenuAction::Play3D => next_state.set(AppState::AlgorithmSelection3D), 
            }
        }
    }
}