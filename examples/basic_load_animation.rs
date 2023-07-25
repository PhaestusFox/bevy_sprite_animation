use bevy::prelude::*;
use bevy_sprite_animation::prelude::*;

/// this is an exaple of how to load a single animation from a file and add it to you game
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: bevy::render::texture::ImageSampler::nearest_descriptor(),
        }))
        // add the plugin to our game, the 10 is the max number of nodes in a single chain
        // this provents the app getting stuck in a loop
        .add_plugins(SpriteAnimationPlugin::<10>)
        .add_systems(Startup, setup_animations)
        // register the Match node with our games Generic matching vairiable
        .register_type::<MatchNode<ZState>>()
        .run()
}

///ZState in the state the zombie is currently in
#[derive(
    Debug,
    Component,
    Hash,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Reflect,
    serde::Serialize,
    serde::Deserialize,
    PartialOrd,
    Ord,
)]
#[reflect_value(Deserialize)]
pub enum ZState {
    Idle,
    Walking,
    Running,
    Attacking,
    FallF,
    StandF,
    FallB,
    StandB,
    LayingF,
    LayingB,
    Test,
}

#[derive(Resource)]
struct NodeTree(Handle<AnimationNode>);

fn setup_animations(mut commands: Commands, asset_server: Res<AssetServer>) {
    // add a camera
    commands.spawn(Camera2dBundle::default());

    // loading from a .node file will return a handle to that single node
    // this will override any given id to the node
    let fps = asset_server.load("test.node");
    // loading form a .nodetree will return a handle to ReferenceNode this holdes all handles to all
    // the nodes in the tree this is done so the nodes can have custom ids, but would then unload the next
    // frame
    let tree = asset_server.load("Zombie1.nodetree");

    // crate a state that tells trees dynamic data, would be pointless
    // if it always the same each frame
    let mut state = AnimationState::default();
    state.set_attribute(Attribute::new_attribute("ZombieState"), ZState::Idle);

    // spawn SpriteBundle
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::X * 10.),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(1000.)),
                ..Default::default()
            },
            ..Default::default()
        },
        // add default AnimationState
        state,
        // add a startnode to our entity
        // this it the first node run each frame
        StartNode::from_handle(fps),
    ));
    // need to keep a strong handle for the ReferenceNodes or
    // they will unload along with all the nodes they hold
    commands.insert_resource(NodeTree(tree));
}
