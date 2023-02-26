use bevy::prelude::*;
use bevy_sprite_animation::prelude::*;

/// this is an exaple of how to load a single animation from a str and add it to you game
fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin {
        default_sampler: bevy::render::texture::ImageSampler::nearest_descriptor(),
    }))
    .add_plugin(SpriteAnimationPlugin::<Zombie>::default())
    .add_startup_system(setup_animations)
    .run()
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Zombie;

fn setup_animations(
    mut commands: Commands,
    mut nodes: ResMut<AnimationNodeTree<Zombie>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let nodes = nodes.as_mut();

    // the str that is used to load the node
    // can be read from a file or any other source
    // im useing a string so I can comment on each part
    let mut index_data = String::new();
    let mut fps_data = String::new();

    // prefix index data with a node id followed by a : so that it gets a specifide id
    // this is optional but used here to make index node easy to remember in fps nodes then feld
    index_data.push_str("NodeID(\"0x0000000000000001\"):");

    // start a node with its node type followed by an (
    // this allows the correct loader to be used
    index_data.push_str("IndexNode(\n");
    fps_data.push_str("FPSNode(\n");
    
    // both index and fps nodes need a name
    index_data.push_str("name: \"Zombie1_Idle\",\n");
    fps_data.push_str("name: \"Zombie_fps\",\n");
    
    // this sets the fps of the node to 7
    fps_data.push_str("fps: 7,");
    // this sets the node that is used after the fps node
    fps_data.push_str("then: NodeID(\"0x1\"),");
    
    // this sets all the frames in order that the index node goes thrue
    index_data.push_str("
    frames: [
    Zombie1/Zombie1_00000.png,
    Zombie1/Zombie1_00001.png,
    Zombie1/Zombie1_00002.png,
    Zombie1/Zombie1_00003.png,
    Zombie1/Zombie1_00004.png,
    Zombie1/Zombie1_00005.png,
    Zombie1/Zombie1_00006.png,
    Zombie1/Zombie1_00007.png,
    Zombie1/Zombie1_00008.png,
    ],");
    
    // set if the animation should loop
    index_data.push_str("is_loop: true,");
    // this set the index attribute that this node looks
    // it can be any attribute as long as that attribute contains a usize
    // attrbutes 256 to 2^16 - 1 are reserved for this purpuse
    index_data.push_str("index: IndexID(256),");
    //finish each node with )
    fps_data.push(')');
    index_data.push(')');
    
    // load a node manulay like this
    // all non custom nodes have loades
    let fps_node = bevy_sprite_animation::nodes::fps_node::FPSNodeLoader.load(
    &fps_data,
    // needed so nodes that need to load assets can do so when they are loaded
    &asset_server).unwrap();
        
    // or load the nodes directly into the node tree like so
    let _indexid = nodes.load_node_from_str(&index_data, &asset_server).unwrap();
    // dont forget to add the nodes to the tree if you manualy loaded them
    let fps_start = nodes.add_node(fps_node);

    // spawn SpriteBundle
    commands.spawn((SpriteBundle{
        transform: Transform::from_translation(Vec3::X * 10.),
        sprite: Sprite{custom_size: Some(Vec2::splat(1000.)), ..Default::default()},
        ..Default::default()
    },
    // add animation flag
    Zombie,
    // add default AnimationState
    AnimationState::default(),
    // add a startnode to our entity with the fps node as its first node
    StartNode::from_nodeid(fps_start)));
}
