use clap::Parser;
use i3m::asset::manager::ResourceManager;
use i3m::asset::ResourceDataRef;
use i3m::core::task::TaskPool;
use i3m::resource::model::Model;
use i3m::resource::model::AnimationSource;
use i3m_graph::BaseSceneGraph; // Import the trait for node access
use serde::{Serialize, Deserialize};
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "I3M Scene Converter")]
#[command(about = "Recursively converts .rgs files in a directory to .i3m format for the IThreeM Engine")]
struct Cli {
    /// Path to the directory containing .rgs files
    #[arg(short, long)]
    input_dir: String,

    /// Path to the output directory where .i3m files will be saved
    #[arg(short, long)]
    output_dir: String,
}

#[derive(Serialize, Deserialize)]
struct I3MNode {
    name: String,
    position: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
    children: Vec<I3MNode>,
}

#[derive(Serialize, Deserialize)]
struct I3MScene {
    nodes: Vec<I3MNode>,
    assets: Vec<String>,
}

fn load_i3m_scene(file_path: &str, resource_manager: &ResourceManager) -> I3MScene {
    let resource = resource_manager.request::<Model>(file_path); // Store the request in a variable
    let model: ResourceDataRef<Model> = resource.data_ref(); // Use the variable here

    let graph = model.inner_graph();

    let mut scene = I3MScene {
        nodes: Vec::new(),
        assets: Vec::new(),
    };

    // Assuming iteration over node handles by index
    for index in 0..graph.node_count() { // Replace with actual method if different
        let handle = graph.handle_from_index(index); // Replace with actual handle retrieval method
        let node = graph.node(handle);

        let scale = node.local_transform().scale();
        let scale_vec: [f32; 3] = [
            scale.x as f32, // Adjust based on actual scale structure
            scale.y as f32,
            scale.z as f32,
        ];

        let i3m_node = I3MNode {
            name: node.name().to_string(),
            position: [
                node.local_transform().position().x,
                node.local_transform().position().y,
                node.local_transform().position().z,
            ],
            rotation: node.local_transform().rotation().coords.as_slice().try_into().unwrap(),
            scale: scale_vec,
            children: vec![],
        };

        scene.nodes.push(i3m_node);
    }

    scene
}


fn save_to_i3m(scene: &I3MScene, output_file: &str) {
    let json = serde_json::to_string_pretty(scene).unwrap();
    let mut file = File::create(output_file).expect("Unable to create file");
    file.write_all(json.as_bytes()).expect("Unable to write data");
}

fn main() {
    let args = Cli::parse();

    let input_dir = Path::new(&args.input_dir);
    let output_dir = Path::new(&args.output_dir);
    let task_pool = Arc::new(TaskPool::new());
    let resource_manager = ResourceManager::new(task_pool);

    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.extension().map(|ext| ext == "rgs").unwrap_or(false) {
            let relative_path = path.strip_prefix(input_dir).unwrap();
            let output_file_path = output_dir.join(relative_path).with_extension("i3m");

            if let Some(parent_dir) = output_file_path.parent() {
                create_dir_all(parent_dir).expect("Failed to create output directories");
            }

            println!("Processing file: {:?}", path);
            let scene = load_i3m_scene(path.to_str().unwrap(), &resource_manager);
            save_to_i3m(&scene, output_file_path.to_str().unwrap());

            println!("Saved converted scene to: {:?}", output_file_path);
        }
    }

    println!("Conversion complete!");
}
