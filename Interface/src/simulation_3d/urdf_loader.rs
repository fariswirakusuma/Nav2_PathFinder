
use std::fs;
use std::path::Path;

pub fn generate_urdf_to_workspace(selected_filename: &str) -> std::io::Result<()> {
    let source_dir = "Test/urdf";
    let target_dir = "ROS_workspace/description/urdf";

    let source_path = Path::new(source_dir).join(selected_filename);
    let target_path = Path::new(target_dir).join(selected_filename);

    fs::create_dir_all(target_dir)?;
    fs::copy(&source_path, &target_path)?;

    println!("Successfully deployed {} to ROS workspace.", selected_filename);
    Ok(())
}

pub fn parse_and_animate_robot(){
    
     
}