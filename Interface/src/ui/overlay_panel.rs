use bevy::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use std::fs;

use crate::simulation_2d::{SimulationState, PlannerLog, RosBridge};
// use crate::config::states::AppState;
use crate::config::setup::SetupConfig;

#[derive(Component)]
pub(crate) struct PanelEntity;

#[derive(Component)]
struct StatsText;

#[derive(Component)]
struct CalcLogText;

#[derive(Component)]
struct ResetButton;

#[derive(Component)]
struct ScrollingList {
    position: f32,
}

pub(crate) fn setup_panel(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        PanelEntity,
    )).with_children(|root| {
        root.spawn(Node { flex_grow: 1.0, ..default() });

        root.spawn((
            Node {
                width: Val::Px(460.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::left(Val::Px(2.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.98)),
            BorderColor::all(Color::srgb(0.2, 0.3, 0.4)),
        ))
        .with_children(|sidebar| {
            sidebar.spawn((
                Text::new("NAV2 METRICS"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.2, 0.8, 1.0)),
            ));

            sidebar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.3, 0.4)),
            ));

            sidebar.spawn((
                Text::new("Awaiting data..."),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::WHITE),
                StatsText,
            ));

            sidebar.spawn((
                Node {
                    flex_grow: 1.0, 
                    width: Val::Percent(100.0),
                    overflow: Overflow::clip_y(), 
                    position_type: PositionType::Relative, 
                    ..default()
                },
            )).with_children(|viewport| {
                viewport.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        position_type: PositionType::Absolute, 
                        width: Val::Percent(100.0),
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        ..default()
                    },
                    ScrollingList { position: 0.0 },
                )).with_children(|scroll_content| {
                    scroll_content.spawn((
                        Text::new(""),
                        TextFont { font_size: 13.0, ..default() }, 
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        CalcLogText,
                    ));
                });
            });

            sidebar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.12, 0.15, 1.0)),
                BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
            )).with_children(|info| {
                info.spawn((
                    Text::new("MOUSE CONTROLS"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
                info.spawn((
                    Text::new("• Mid Click  : Set Start  | Right Click: Set Goal\n• Left Click : Obstacle    | Scroll/Key: Scroll Log"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                ));
            });
            
            sidebar.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(42.0), 
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.7, 0.2, 0.2)),
                BorderColor::all(Color::srgb(0.9, 0.4, 0.4)),
                ResetButton,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("RESET PATH"),
                    TextFont { font_size: 15.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

pub(crate) fn manual_mouse_scroll(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_messages: MessageReader<MouseWheel>, 
    mut query_list: Query<(&mut ScrollingList, &mut Node)>,
) {
    let mut dy = 0.0;
    
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        dy += 25.0; 
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        dy -= 25.0; 
    }

    for message in mouse_wheel_messages.read() {
        match message.unit {
            MouseScrollUnit::Line => dy += message.y * 20.0,
            MouseScrollUnit::Pixel => dy += message.y,
        }
    }

    for (mut scroll_list, mut node) in &mut query_list {
        if dy != 0.0 {
            scroll_list.position += dy;
            if scroll_list.position > 0.0 {
                scroll_list.position = 0.0;
            }
        }
        node.top = Val::Px(scroll_list.position);
    }
}

pub(crate) fn update_panel_stats(
    state: Res<SimulationState>,
    planner_log: Res<PlannerLog>,
    config: Res<SetupConfig>,
    mut q_stats: Query<&mut Text, (With<StatsText>, Without<CalcLogText>)>,
    mut q_calc: Query<&mut Text, (With<CalcLogText>, Without<StatsText>)>,
) {
    let mut total_distance = 0.0;
    if state.path.len() > 1 {
        for i in 0..state.path.len() - 1 {
            let dx = state.path[i + 1].x - state.path[i].x;
            let dy = state.path[i + 1].y - state.path[i].y;
            total_distance += (dx.powi(2) + dy.powi(2)).sqrt();
        }
    }

    for mut text in &mut q_stats {
        **text = format!(
            "Algorithm   : {}\nObstacles   : {}\nPath Nodes  : {}\nDistance    : {:.2}m",
            config.algorithm, state.obstacles.len(), state.path.len(), total_distance
        );
    }

    for mut text in &mut q_calc {
        let mut log_text = String::from("\n[ SYSTEM LOGS ]\n");
        if planner_log.system_logs.is_empty() {
            log_text.push_str("Standby...\n");
        } else {
            for log in &planner_log.system_logs { 
                log_text.push_str(&format!(" {}\n", log)); 
            }
        }

        log_text.push_str("\n[ CALCULATION LOG ]\n");
        if state.is_calculating {
            log_text.push_str(&format!("Calculation ongoing... {:.1}s\n\n", state.calc_elapsed));
        }

        if planner_log.history.is_empty() && !state.is_calculating {
            log_text.push_str("Waiting for algorithm process...\n");
        } else {
            log_text.push_str("+----------+-------+-------+-------+\n");
            log_text.push_str("|  NODE    |   F   |   G   |   H   |\n");
            log_text.push_str("+----------+-------+-------+-------+\n");
            
            for log in planner_log.history.iter().rev() {
                let (f_str, g_str, h_str) = match config.algorithm.as_str() {
                    "AStar" => (format!("{:.1}", log.f), format!("{:.1}", log.g), format!("{:.1}", log.h)),
                    "UCS" | "Dijkstra" => (format!("{:.1}", log.f), format!("{:.1}", log.g), " - ".to_string()),
                    "GBFS" => (format!("{:.1}", log.f), " - ".to_string(), format!("{:.1}", log.h)),
                    _ => (format!("{:.1}", log.f), " - ".to_string(), " - ".to_string()),
                };
                
                let node_name = format!("N[{}]", log.index);
                log_text.push_str(&format!(
                    "| {:<8} | {:^5} | {:^5} | {:^5} |\n", 
                    node_name, f_str, g_str, h_str
                ));
            }
            log_text.push_str("+----------+-------+-------+-------+\n");
        }
        **text = log_text;
    }
}
pub(crate) fn handle_reset_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ResetButton>),
    >,
    mut state: ResMut<SimulationState>,
    mut planner_log: ResMut<PlannerLog>,
    bridge: Res<RosBridge>,
) {
    for (interaction, mut color, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.5, 0.1, 0.1));
                *border = BorderColor::all(Color::srgb(0.6, 0.2, 0.2));
                
                state.path.clear();
                state.obstacles.clear();
                state.start_pos = None;
                state.goal_pos = None;
                state.is_calculating = false;
                state.calc_elapsed = 0.0;
                
                planner_log.history.clear();
                planner_log.system_logs.clear();

                let clear_srv_msg = serde_json::json!({
                    "op": "call_service",
                    "service": "/global_costmap/clear_entirely_global_costmap",
                    "args": {}
                });
                let _ = bridge.tx.lock().unwrap().send(clear_srv_msg.to_string());
                
                planner_log.system_logs.push("[INFO] System Reset. Global Costmap Cleared.".to_string());
                let _ = fs::write("../Test/backendTest/path_result.json", "[]");
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.8, 0.3, 0.3));
                *border = BorderColor::all(Color::srgb(1.0, 0.5, 0.5));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.7, 0.2, 0.2));
                *border = BorderColor::all(Color::srgb(0.9, 0.4, 0.4));
            }
        }
    }
}