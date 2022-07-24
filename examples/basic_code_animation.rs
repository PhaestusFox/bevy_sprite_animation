use bevy::prelude::*;
use bevy_sprite_animation::prelude::*;

/// this is an exaple of how to make a single animation in code and add it to you game
fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
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
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let nodes = nodes.as_mut();

    let mut images = Vec::new();
    // Load all the images
    for i in 0..=67 {
        images.push(asset_server.load(&format!("Zombie1/Zombie1_{:05}.png", i)));
    }

    // Add a new IndexNode with a custom id of 0x1
    nodes.insert_node(NodeID::from_u64(0x1), Box::new(
        bevy_sprite_animation::nodes::IndexNode::new(
        // this node will be called test
        "test",
        // this is the frames in oreder that it will use
        &images,
        // we want it to loop after it gets to the end
        true)
    ));
    // Add a node with a self generated id
    let fps_start = nodes.add_node(Box::new(
        bevy_sprite_animation::nodes::FPSNode::new(
        // this node is call fps
        "fps",
        // it will change frames 7 times a seconed
        7,
        // it will go to the frame we just inserted with an id of 0x1
        NodeID::from_u64(0x1))
    ));


    // spawn SpriteBundle
    commands.spawn_bundle(SpriteBundle{
        transform: Transform::from_translation(Vec3::X * 10.),
        sprite: Sprite{custom_size: Some(Vec2::splat(1000.)), ..Default::default()},
        ..Default::default()
    })
    // add animation flag
    .insert(Zombie)
    // add default AnimationState
    .insert(AnimationState::default())
    // add a startnode to our entity with the fps node as its first node
    .insert(StartNode::from_nodeid(fps_start));
}