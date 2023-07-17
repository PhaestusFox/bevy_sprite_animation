use bevy::{prelude::*, render::texture::ImageSampler};
use bevy_sprite_animation::prelude::*;

use animation::ZState;

mod animation {
    use bevy_sprite_animation::prelude::*;

    use bevy::prelude::*;

    ///this is a representaion of how you get data in and out of animation states
    pub struct YourAnimationPlugin;

    impl Plugin for YourAnimationPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, (zombie_state_update.before(AnimationSet::Update), zombie_update_state.after(AnimationSet::Update)))
            .register_type::<ZState>();
        }
    }


    ///ZState in the state the zombie is currently in
    #[derive(Debug, Component, Hash, PartialEq, Eq, Clone, Copy, Reflect,
        serde::Serialize, serde::Deserialize, PartialOrd, Ord)]
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

    impl Default for ZState {
        fn default() -> Self {
            ZState::Idle
        }
    }

    ///updates the zombies animation state to reflect its progam state
    fn zombie_state_update(
        mut zombies: Query<(&mut AnimationState, &ZState), (With<super::Zombie>, Changed<ZState>)>
    ) {
        let att = Attribute::new_attribute("ZombieState");
        for (mut state, name) in zombies.iter_mut() {
            state.set_attribute(att.clone(), *name);
        }
    }

    ///update the zombies program state is it was changed by the animation
    fn zombie_update_state(
        mut zombies: Query<(&AnimationState, &mut ZState), With<super::Zombie>>
    ) {
        let attribute = Attribute::from_str("ZombieState");
        for (state,mut name) in zombies.iter_mut() {
            if state.changed(&attribute) {
                *name = *state.attribute::<ZState>(&attribute);
            }
        }
    }

}

mod player {
    use bevy::prelude::*;
    use bevy_sprite_animation::{state::AnimationState, prelude::Attribute};

    use super::animation::ZState;
    
    #[derive(Debug, Component)]
    pub struct Player;
    
    impl Plugin for Player {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, player_animation_update);
        }
    }
    
    fn player_animation_update(
        mut player: Query<(&mut ZState, &mut AnimationState), With<Player>>,
        mut local: Local<[Attribute; 3]>,
        input: Res<Input<KeyCode>>,
    ){
        if local[0] == Attribute::Default {
            local[0] = Attribute::new_attribute("ZombieState");
            local[1] = Attribute::new_index("Stand");
            local[2] = Attribute::new_index("Fall");
        }
        let (mut zstate, mut animation) = player.single_mut();
        for key in input.get_just_pressed() {
            match (key, *zstate) {
                (KeyCode::A, ZState::Walking) | (KeyCode::Left, ZState::Walking) |
                (KeyCode::A, ZState::Idle) | (KeyCode::Left, ZState::Idle) => {
                    if zstate.as_ref() == &ZState::Walking || zstate.as_ref() == &ZState::Idle {
                        animation.set_attribute(Attribute::FlipX, true);
                    }
                },
                (KeyCode::D, ZState::Walking) | (KeyCode::Right, ZState::Walking) |
                (KeyCode::D, ZState::Idle) | (KeyCode::Right, ZState::Idle) => {
                    if zstate.as_ref() == &ZState::Walking || zstate.as_ref() == &ZState::Idle {
                        animation.set_attribute(Attribute::FlipX, false);
                    }
                },
                (KeyCode::Up, state) => {
                    *zstate = match state {
                        ZState::Idle => {ZState::Walking},
                        ZState::Walking => {ZState::Running},
                        ZState::Running => {ZState::Running},
                        ZState::Attacking => {ZState::Attacking},
                        ZState::FallF => {ZState::FallF},
                        ZState::StandF => {ZState::StandF},
                        ZState::FallB => {ZState::FallB},
                        ZState::StandB => {ZState::StandB},
                        ZState::Test => {ZState::Test},
                        ZState::LayingB => {ZState::StandB},
                        ZState::LayingF => {ZState::StandF},
                    }
                },
                (KeyCode::Down, state) => {
                    let facing = *animation.attribute::<bool>(&Attribute::FlipX);
                    *zstate = match state {
                        ZState::Idle => {if facing { ZState::FallF } else {ZState::FallB}},
                        ZState::Walking => {ZState::Idle},
                        ZState::Running => {ZState::Walking},
                        ZState::Attacking => {ZState::Attacking},
                        ZState::FallF => {ZState::FallF},
                        ZState::StandF => {let index = animation.index(&local[1]);
                            animation.set_attribute(local[2].clone(), 6 - index); ZState::FallF},
                        ZState::FallB => {ZState::FallB},
                        ZState::StandB => {let index = animation.index(&local[1]);
                            animation.set_attribute(local[2].clone(), 7 - index); ZState::FallB},
                        ZState::Test => {ZState::Test},
                        ZState::LayingB => {ZState::LayingB},
                        ZState::LayingF => {ZState::LayingF},
                    }
                },
                (KeyCode::Space, ZState::Idle) | (KeyCode::Space, ZState::Walking) => {
                    *zstate = ZState::Attacking;
                },
                (KeyCode::NumpadSubtract, state) => {
                    println!("{:?}", state);
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin {
        default_sampler: ImageSampler::nearest_descriptor(),
    }));
    //add the editor if you want it
    #[cfg(feature = "editor")]
    app.add_plugins(bevy_editor_pls::EditorPlugin::default());

    //add this plugin
    app.add_plugins((animation::YourAnimationPlugin,
        SpriteAnimationPlugin::<20>,
        player::Player))
    .add_systems(Startup ,setup_animations)
    .register_type::<MatchNode<ZState>>()
    .add_systems(Update, print_tree)
    .run()
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Zombie;

#[derive(Resource)]
struct Handles(Vec<Handle<AnimationNode>>);

fn print_tree(
    nodes: Res<Assets<AnimationNode>>,
    roots: Res<Handles>,
    input: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    if input.just_pressed(KeyCode::F7) {
        let mut string = String::new();
        for root in &roots.0 {
            let Some(root) = nodes.get(root) else {error!("Node not loaded"); continue;};
            root.serialize(&mut string, &asset_server).unwrap();
            let Some(root) = root.downcast_ref::<ReferenceNode>() else {error!("Not ReferenceNode"); continue;};
            println!("{:?}", root.1);
            for node in root.iter() {
                let Some(node) = nodes.get(node) else {continue;};
                println!("\t{:?}", node);
                node.serialize(&mut string, &asset_server).unwrap();
            }
            print!("\n");
            println!("{}", string);
        }
    }
}

fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let fall_index = Attribute::new_index("Fall");
    let stand_index = Attribute::new_index("Stand");
    let attack_index = Attribute::new_index("Attack");

    let test_handle: Handle<AnimationNode> = asset_server.load("test.node");
    let tree_handle: Handle<AnimationNode> = asset_server.load("./Zombie1.nodetree");

    let mut start = AnimationState::default();
    start.set_temporary(fall_index);
    start.set_temporary(stand_index);
    start.set_temporary(attack_index);
    commands.spawn((SpriteBundle{
        transform: Transform::from_translation(Vec3::X * 10.),
        sprite: Sprite{custom_size: Some(Vec2::splat(1000.)), ..Default::default()},
        ..Default::default()
    },
    Zombie,
    ZState::Attacking,
    start,
    player::Player,
    StartNode::from_handle(test_handle),
    ));

    commands.insert_resource(Handles(vec![tree_handle]));
}