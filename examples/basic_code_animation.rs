use bevy::prelude::*;
use bevy_sprite_animation::prelude::*;

/// this is an exaple of how to make a single animation in code and add it to you game
fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin {
        default_sampler: bevy::render::texture::ImageSampler::nearest_descriptor(),
    }))
    .add_plugins(SpriteAnimationPlugin)
    .add_systems(Startup, setup_animations)
    .run()
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Zombie;

fn setup_animations(
    mut commands: Commands,
    mut nodes: ResMut<AnimationNodeTree>,
    asset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<AnimationNode>>,
) {
    commands.spawn(Camera2dBundle::default());

    let nodes = nodes.as_mut();

    let mut images = Vec::new();
    // Load all the images
    for i in 0..=67 {
        images.push(asset_server.load(&format!("Zombie1/Zombie1_{:05}.png", i)));
    }

    let start = NodeId::from_u64(0x1);

    // Add a new IndexNode with a custom id of 0x1
    let index = assets.set(NodeId::from_u64(0x1), AnimationNode::new(
        bevy_sprite_animation::nodes::IndexNode::new(
        // this node will be called test
        "test",
        // this is the frames in oreder that it will use
        &images,
        // we want it to loop after it gets to the end
        true)
    ));
    // Add a node with a self generated id
    let fps_start = assets.set(start.to_static(), AnimationNode::new(
        bevy_sprite_animation::nodes::FPSNode::new(
        // this node is call fps
        "fps",
        // it will change frames 7 times a seconed
        7,
        // it will go to the frame we just inserted with an id of 0x1
        NodeId::from_u64(0x1))
    ));


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
    StartNode(start)));
}