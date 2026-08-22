use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use serde_json::Value;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::fs;
use tungstenite::{connect, Message};
use url::Url;

use crate::config::states::AppState;
use crate::config::navigation::{NavStack, pop_state};
use crate::config::setup::SetupConfig;

pub mod message;
use self::message::{SimulationPayload, MapSize, Point2D, StepLog};

pub struct Simulation2dPlugin;

#[derive(Component)]
pub struct Sim2DEntity;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone,Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Resource)] 
pub struct SimulationState {
    pub obstacles: Vec<Point>,
    pub path: Vec<Point>,
    pub selected_algorithm: String,
    pub start_pos: Option<Point>,
    pub goal_pos: Option<Point>,
    pub is_calculating: bool,  
    pub calc_elapsed: f32,     
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            obstacles: Vec::new(),
            path: Vec::new(),
            selected_algorithm: "AStar".to_string(),
            start_pos: None,
            goal_pos: None,
            is_calculating: false,
            calc_elapsed: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct PlannerLog {
    pub history: Vec<StepLog>, 
    pub system_logs: Vec<String>,
}

#[derive(Resource)]
pub struct RosBridge {
    pub tx: std::sync::Mutex<Sender<String>>,
    pub rx: std::sync::Mutex<Receiver<String>>,
}

#[derive(Component)]
pub struct LoadingUiMarker;

#[derive(Component)]
pub struct BackButton;

const SCALE: f32 = 400.0;
const GRID_SIZE: f32 = 0.05;
// let rosbridge_url = std::env::var("ROSBRIDGE_URL").unwrap_or_else(|_| "ws://127.0.0.1:9090".to_string());

impl Plugin for Simulation2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationState>()
            .init_resource::<PlannerLog>()
            
            .add_systems(OnEnter(AppState::Sim2DLoading), (setup_rosbridge, setup_loading_screen))
            .add_systems(Update, check_backend_ready.run_if(in_state(AppState::Sim2DLoading)))
            .add_systems(OnExit(AppState::Sim2DLoading), cleanup_loading_screen)
            
            .add_systems(OnEnter(AppState::Sim2DRun), (setup_2d_grid, setup_back_button))
            .add_systems(
                Update,
                (
                    handle_click, 
                    receive_path_data, 
                    draw_visualization, 
                    handle_back_button,
                    update_timer
                )
                .run_if(in_state(AppState::Sim2DRun)),
            )
            .add_systems(OnExit(AppState::Sim2DRun), (cleanup_sim2d, cleanup_back_button));
    }
}

fn setup_loading_screen(mut commands: Commands) {
    commands.spawn((Camera2d, LoadingUiMarker));
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.0)), LoadingUiMarker,
    )).with_children(|parent| {
        parent.spawn((Text::new("Connecting to ROS 2 Backend...\nPlease Wait."), TextFont { font_size: 30.0, ..default() }, TextColor(Color::WHITE)));
    });
}

fn check_backend_ready(mut next_state: ResMut<NextState<AppState>>, time: Res<Time>, mut timer: Local<f32>) {
    *timer += time.delta_secs();
    if *timer > 2.5 { next_state.set(AppState::Sim2DRun); *timer = 0.0; }
}

fn cleanup_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingUiMarker>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}

fn setup_back_button(mut commands: Commands) {
    commands.spawn((
        Button,
        Node { width: Val::Px(100.0), height: Val::Px(40.0), position_type: PositionType::Absolute, top: Val::Px(20.0), left: Val::Px(20.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)), BackButton,
    )).with_children(|parent| {
        parent.spawn((Text::new("< Back"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
    });
}

fn handle_back_button(interaction_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>, mut next_state: ResMut<NextState<AppState>>, mut nav_stack: ResMut<NavStack>) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed { pop_state(&mut next_state, &mut nav_stack); }
    }
}

fn cleanup_back_button(mut commands: Commands, query: Query<Entity, With<BackButton>>) {
    for entity in &query { commands.entity(entity).despawn(); }
}

fn setup_2d_grid(mut commands: Commands, config: Res<SetupConfig>, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, Transform::from_scale(Vec3::splat(4.5)), Sim2DEntity));
    if !config.map_name.is_empty() {
        let png_name = config.map_name.replace(".yaml", ".png");
        let image_handle: Handle<Image> = asset_server.load(format!("Test/maps/{}", png_name));
        commands.spawn((
            Sprite { image: image_handle, custom_size: Some(Vec2::new(10.0 * SCALE, 10.0 * SCALE)), ..default() },
            Transform::from_xyz(0.0, 0.0, -1.0), Sim2DEntity,
        ));
    }
}

fn inject_map_to_yaml(map_name: &str) {
    let yaml_path = std::env::var("NAV2_PARAMS_PATH").unwrap_or_else(|_| "../ROS_workspace/src/navigation/config/nav2_params.yaml".to_string());
    
    if let Ok(content) = fs::read_to_string(&yaml_path) {
        let mut new_lines = Vec::new();
        let mut in_map_server = false;
        let mut in_ros_params = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("map_server:") {
                in_map_server = true;
                new_lines.push(line.to_string());
            } else if in_map_server && trimmed.starts_with("ros__parameters:") {
                in_ros_params = true;
                new_lines.push(line.to_string());
            } else if in_map_server && in_ros_params && trimmed.starts_with("yaml_filename:") {
                let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                let docker_map_path = format!("/workspace/src/navigation/maps/{}", map_name);
                new_lines.push(format!("{}yaml_filename: \"{}\"", indent, docker_map_path));
                in_map_server = false;
                in_ros_params = false;
            } else {
                if in_map_server && !trimmed.is_empty() && !line.starts_with(' ') && !trimmed.starts_with("map_server:") {
                    in_map_server = false;
                    in_ros_params = false;
                }
                new_lines.push(line.to_string());
            }
        }
        let _ = fs::write(&yaml_path, new_lines.join("\n"));
    }
}

fn setup_rosbridge(mut commands: Commands, config: Res<SetupConfig>) {
    let map_name = config.map_name.clone();
    let rosbridge_url = std::env::var("ROSBRIDGE_URL").unwrap_or_else(|_| "ws://127.0.0.1:9090".to_string());
    inject_map_to_yaml(&map_name);

    let (tx_out, rx_out) = mpsc::channel::<String>(); 
    let (tx_in, rx_in) = mpsc::channel::<String>(); 

    thread::spawn(move || {
        let url = Url::parse(&rosbridge_url).expect("URL tidak valid");
        let mut socket;
        println!("[DEBUG] Attempting to connect to {}", rosbridge_url);
        loop {
        match connect(url.clone()) {
            Ok((s, _)) => {
                socket = s;
                println!("[DEBUG] Connected to ROSBridge!");
                break;
            }
            Err(e) => {
                println!("[DEBUG] Connection failed: {}. Retrying...", e);
                thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    }
        match socket.get_mut() { tungstenite::stream::MaybeTlsStream::Plain(s) => s.set_nonblocking(true).unwrap(), _ => (), }

        let subscribe_plan = serde_json::json!({ "op": "subscribe", "topic": "/plan", "type": "nav_msgs/msg/Path" });
        let _ = socket.send(Message::Text(subscribe_plan.to_string()));

        let subscribe_log = serde_json::json!({ "op": "subscribe", "topic": "/planner_log", "type": "std_msgs/msg/String" });
        let _ = socket.send(Message::Text(subscribe_log.to_string()));

        if !map_name.is_empty() {
            thread::sleep(std::time::Duration::from_secs(5));
            let docker_map_path = format!("/workspace/src/navigation/maps/{}", map_name);
            let load_map_msg = serde_json::json!({
                "op": "call_service",
                "service": "/map_server/load_map",
                "args": { "map_url": docker_map_path }
            });
            let _ = socket.send(Message::Text(load_map_msg.to_string()));
            let clear_costmap_msg = serde_json::json!({
                "op": "call_service",
                "service": "/global_costmap/clear_entirely_global_costmap",
                "args": {}
            });
            let _ = socket.send(Message::Text(clear_costmap_msg.to_string()));
        }

        loop {
            if let Ok(msg) = rx_out.try_recv() {
                println!("[DEBUG] Sending to ROS: {}", msg);
                if let Err(e) = socket.send(Message::Text(msg)) {
                    println!("[ERROR] Send failed: {}", e);
                }
            }
            
            match socket.read() { 
                Ok(Message::Text(text)) => { let _ = tx_in.send(text); } 
                _ => {} 
            }
            
            thread::sleep(std::time::Duration::from_secs_f32(0.01));
        }
    });

    commands.insert_resource(RosBridge { tx: std::sync::Mutex::new(tx_out), rx: std::sync::Mutex::new(rx_in) });
}

fn update_timer(time: Res<Time>, mut state: ResMut<SimulationState>) {
    if state.is_calculating {
        state.calc_elapsed += time.delta_secs();
    }
}

fn receive_path_data(mut state: ResMut<SimulationState>, mut planner_log: ResMut<PlannerLog>, bridge: Res<RosBridge>) {
    while let Ok(msg) = bridge.rx.lock().unwrap().try_recv() {
        if let Ok(parsed) = serde_json::from_str::<Value>(&msg) {
            let op = parsed["op"].as_str().unwrap_or("");
            let topic = parsed["topic"].as_str().unwrap_or("");

            if op == "publish" {
                match topic {
                    "/plan" => {
                        if let Some(poses) = parsed["msg"]["poses"].as_array() {
                            state.path.clear();
                            state.is_calculating = false; 
                            planner_log.system_logs.push(format!("[INFO] Path found! Nodes: {}", poses.len()));
                            
                            for pose_obj in poses {
                                if let Some(pos) = pose_obj["pose"]["position"].as_object() {
                                    let x = pos["x"].as_f64().unwrap_or(0.0) as f32;
                                    let y = pos["y"].as_f64().unwrap_or(0.0) as f32;
                                    state.path.push(Point { x, y });
                                }
                            }

                            if !state.path.is_empty() {
                                if let Some(goal) = state.goal_pos {
                                    let last_idx = state.path.len() - 1;
                                    state.path[last_idx].x = goal.x;
                                    state.path[last_idx].y = goal.y;
                                }
                            }
                        }
                    },
                    "/planner_log" => {
                        if let Some(json_str) = parsed["msg"]["data"].as_str() {
                            if let Ok(log_data) = serde_json::from_str::<StepLog>(json_str) {
                                planner_log.history.push(log_data);
                                if planner_log.history.len() > 100 { planner_log.history.remove(0); } 
                            } else {
                                planner_log.system_logs.push(json_str.to_string());
                                if planner_log.system_logs.len() > 15 { planner_log.system_logs.remove(0); }
                                
                                if json_str.contains("ABORT") || json_str.contains("failed") || json_str.contains("Gagal") || json_str.contains("ERROR") {
                                    state.is_calculating = false;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
fn handle_click(
    buttons: Res<ButtonInput<MouseButton>>, q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Sim2DEntity>>,
    mut state: ResMut<SimulationState>, mut planner_log: ResMut<PlannerLog>, bridge: Res<RosBridge>,
) {
    let mut data_changed = false;
    let mut click_pos = None;

    if buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right) || buttons.just_pressed(MouseButton::Middle) {
        if let Some(window) = q_windows.iter().next() {
            if let Some((camera, camera_transform)) = q_camera.iter().next() {
                if let Some(cursor_position) = window.cursor_position() {
                    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                        let sim_x = world_position.x / SCALE;
                        let sim_y = world_position.y / SCALE;
                        
                        let snapped_micro_x = (sim_x / GRID_SIZE).round() * GRID_SIZE;
                        let snapped_micro_y = (sim_y / GRID_SIZE).round() * GRID_SIZE;
                        
                        let snapped_macro_x = (sim_x / 0.5).round() * 0.5;
                        let snapped_macro_y = (sim_y / 0.5).round() * 0.5;
                        
                        click_pos = Some((snapped_micro_x, snapped_micro_y, snapped_macro_x, snapped_macro_y, sim_x, sim_y));
                    }
                }
            }
        }
    }

    if let Some((snapped_micro_x, snapped_micro_y, snapped_macro_x, snapped_macro_y, sim_x, sim_y)) = click_pos {
        if buttons.just_pressed(MouseButton::Right) {
            state.goal_pos = Some(Point { x: snapped_micro_x, y: snapped_micro_y }); 
            data_changed = true;
        } else if buttons.just_pressed(MouseButton::Middle) {
            state.start_pos = Some(Point { x: snapped_micro_x, y: snapped_micro_y }); 
            data_changed = true;
        } else if buttons.just_pressed(MouseButton::Left) {
            let mut removed = false;
            let click_radius = 0.25; 
            
            state.obstacles.retain(|p| {
                let dist = ((p.x - sim_x).powi(2) + (p.y - sim_y).powi(2)).sqrt();
                if dist < click_radius { removed = true; false } else { true }
            });

            if !removed {
                state.obstacles.push(Point { x: snapped_macro_x, y: snapped_macro_y });
            }
            data_changed = true;
        }
    }

    if data_changed {
        state.path.clear();
        planner_log.history.clear();
        
        if let (Some(start), Some(goal)) = (state.start_pos.clone(), state.goal_pos.clone()) {
            state.is_calculating = true;
            state.calc_elapsed = 0.0;
            
            let payload = SimulationPayload {
                map_size: MapSize { width: 10.0, height: 10.0, resolution: 0.05 },
                start: Point2D { x: start.x as f64, y: start.y as f64 },
                goal: Point2D { x: goal.x as f64, y: goal.y as f64 },
                algorithm: if state.selected_algorithm == "UCS" { "Dijkstra".to_string() } else { state.selected_algorithm.clone() },
                obstacles: state.obstacles.iter().map(|p| Point2D { x: p.x as f64, y: p.y as f64 }).collect(),
            };

            if let Ok(json_str) = serde_json::to_string(&payload) {
                planner_log.system_logs.push("[INFO] Calculating new path...".to_string());
                
                let pub_msg = serde_json::json!({ "op": "publish", "topic": "/frontend/obstacles", "msg": { "data": json_str } });
                let _ = bridge.tx.lock().unwrap().send(pub_msg.to_string());
            }
        }
    }
}

fn draw_visualization(mut gizmos: Gizmos, state: Res<SimulationState>) {
    const VISUAL_GRID_SIZE: f32 = 0.5; 
    for i in -20..=20 { 
        let offset = (i as f32) * VISUAL_GRID_SIZE * SCALE;
        let color = if i == 0 { Color::srgb(0.4, 0.4, 0.4) } else { Color::srgb(0.15, 0.15, 0.15) };
        gizmos.line_2d(Vec2::new(-10.0 * SCALE, offset), Vec2::new(10.0 * SCALE, offset), color);
        gizmos.line_2d(Vec2::new(offset, -10.0 * SCALE), Vec2::new(offset, 10.0 * SCALE), color);
    }

    gizmos.line_2d(Vec2::new(-5.0 * SCALE, 0.0), Vec2::new(5.0 * SCALE, 0.0), Color::WHITE);
    gizmos.line_2d(Vec2::new(0.0, -5.0 * SCALE), Vec2::new(0.0, 5.0 * SCALE), Color::WHITE);

    for obs in &state.obstacles {
        let center = Vec2::new(obs.x * SCALE, obs.y * SCALE);
        let size_macro = VISUAL_GRID_SIZE * SCALE;
        let half = size_macro / 2.0;
        
        gizmos.rect_2d(center, Vec2::splat(size_macro), Color::srgb(1.0, 0.1, 0.1));
        gizmos.line_2d(center - Vec2::new(half, half), center + Vec2::new(half, half), Color::srgb(1.0, 0.1, 0.1));
        gizmos.line_2d(center - Vec2::new(half, -half), center + Vec2::new(half, -half), Color::srgb(1.0, 0.1, 0.1));
    }
    if !state.path.is_empty() {
        for i in 0..state.path.len() - 1 {
            gizmos.line_2d(
                Vec2::new(state.path[i].x * SCALE, state.path[i].y * SCALE), 
                Vec2::new(state.path[i + 1].x * SCALE, state.path[i + 1].y * SCALE), 
                Color::srgb(0.2, 0.9, 0.2)
            );
        }

        if let Some(goal) = state.goal_pos {
            let last = state.path.last().unwrap();
            gizmos.line_2d(
                Vec2::new(last.x * SCALE, last.y * SCALE),
                Vec2::new(goal.x * SCALE, goal.y * SCALE),
                Color::srgb(0.2, 0.9, 0.2)
            );
        }
    }
    
    for (i, p) in state.path.iter().enumerate() {
        if i > 0 && i < state.path.len() - 1 { 
            gizmos.circle_2d(Vec2::new(p.x * SCALE, p.y * SCALE), 12.0, Color::srgb(0.0, 0.6, 1.0)); 
        }
    }
    
    if let Some(start) = &state.start_pos { 
        gizmos.circle_2d(Vec2::new(start.x * SCALE, start.y * SCALE), 40.0, Color::srgb(1.0, 1.0, 0.0)); 
    }
    
    if let Some(goal) = &state.goal_pos {
        let goal_vec = Vec2::new(goal.x * SCALE, goal.y * SCALE);
        gizmos.circle_2d(goal_vec, 48.0, Color::srgb(1.0, 0.0, 1.0));
        gizmos.line_2d(goal_vec - Vec2::new(32.0, 32.0), goal_vec + Vec2::new(32.0, 32.0), Color::srgb(1.0, 0.0, 1.0));
        gizmos.line_2d(goal_vec - Vec2::new(-32.0, 32.0), goal_vec + Vec2::new(-32.0, 32.0), Color::srgb(1.0, 0.0, 1.0));
    }
}

fn cleanup_sim2d(
    mut commands: Commands, 
    query: Query<Entity, With<Sim2DEntity>>, 
    mut logs: ResMut<PlannerLog>,
    mut state: ResMut<SimulationState>,
    bridge: Option<Res<RosBridge>>
) {
    if let Some(b) = bridge {
        let cancel_msg = serde_json::json!({
            "op": "call_service",
            "service": "/compute_path_to_pose/_action/cancel_goal",
            "args": {}
        });
        let _ = b.tx.lock().unwrap().send(cancel_msg.to_string());

        let rx = b.rx.lock().unwrap();
        while let Ok(_) = rx.try_recv() {}
        
        commands.remove_resource::<RosBridge>();
    }

    for entity in query.iter() { commands.entity(entity).despawn(); }
    logs.history.clear();
    logs.system_logs.clear();
    state.path.clear();
    state.obstacles.clear();
    state.start_pos = None;
    state.goal_pos = None;
    state.is_calculating = false;
    state.calc_elapsed = 0.0;
}