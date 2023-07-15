use bevy::prelude::*;
use bevy_sprite_animation::prelude::*;

/// this is an exaple of how to make a single animation in code and add it to you game
fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin {
        default_sampler: bevy::render::texture::ImageSampler::nearest_descriptor(),
    }))
    // add the plugin to our game, the 10 is the max number of nodes in a single chain
    // this provents the app getting stuck in a loop
    .add_plugins(SpriteAnimationPlugin::<10>)
    .add_systems(Startup, setup_animations)
    .run()
}

fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut nodes: ResMut<Assets<AnimationNode>>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut images = Vec::new();
    // Load all the images
    for i in 0..=67 {
        images.push(asset_server.load(&format!("Zombie1/Zombie1_{:05}.png", i)));
    }

    // Add a new IndexNode
    let index = nodes.add(AnimationNode::new(
        bevy_sprite_animation::nodes::IndexNode::new(
        // this node will be called test
        "test",
        // this is the frames in oreder that it will use
        &images,
        // we want it to loop after it gets to the end
        true)
    ));
    // Add a node with a auto generated id
    let fps_start = nodes.add(AnimationNode::new(
        bevy_sprite_animation::nodes::FPSNode::new(
        // this node is call fps
        "fps",
        // it will change frames 7 times a seconed
        7,
        // it will go to the IndexNode we just inserted
        NodeId::Handle(index))
    ));

    // spawn SpriteBundle
    commands.spawn((SpriteBundle{
        transform: Transform::from_translation(Vec3::X * 10.),
        sprite: Sprite{custom_size: Some(Vec2::splat(1000.)), ..Default::default()},
        ..Default::default()
    },
    // add AnimationState
    AnimationState::default(),
    // add a startnode to our entity with the fps node as its first node
    StartNode::from_handle(fps_start)));
}