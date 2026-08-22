use bevy::prelude::*;
use bevy::camera::*;
use std::thread;
pub use wgpu;
use serde_json::Value;
use std::sync::mpsc::{self, Receiver, Sender};
use std::fs;
use tungstenite::{connect, Message};
use url::Url;
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


fn setup_3d_world(mut commands: Commands) {
    
    // Spawn 3D camera, directional lights, and origin ground plane
}