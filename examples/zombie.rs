use bevy::prelude::*;
use bevy_sprite_animation::prelude::*;

use animation::ZState;

mod animation {
    use bevy_sprite_animation::prelude::*;

    use bevy::prelude::*;

    ///this is a representaion of how you get data in and out of animation states
    pub struct YourAnimationPlugin;

    impl Plugin for YourAnimationPlugin {
        fn build(&self, app: &mut App) {
            app.add_system(zombie_state_update.before(AnimationLabel::Update));
            app.add_system(zombie_update_state.after(AnimationLabel::Update));
        }
    }


    ///ZState in the state the zombie is currently in
    #[derive(Debug, Component, Hash, PartialEq, Eq, Clone, Copy, Reflect,
        serde::Serialize, serde::Deserialize, PartialOrd, Ord)]
    #[reflect_value()]
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
        let att = Attribute::from_str("ZombieState");
        for (mut state, name) in zombies.iter_mut() {
            state.set_attribute(att, *name);
        }
    }

    ///update the zombies program state is it was changed by the animation
    fn zombie_update_state(
        mut zombies: Query<(&AnimationState, &mut ZState), With<super::Zombie>>
    ) {
        let attribute = Attribute::from_str("ZombieState");
        for (state,mut name) in zombies.iter_mut() {
            if state.changed(&attribute) {
                *name = state.get_attribute::<ZState>(&attribute);
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
            app.add_system(player_animation_update);
        }
    }
    
    fn player_animation_update(
        mut player: Query<(&mut ZState, &mut AnimationState), With<Player>>,
        mut local: Local<[Attribute; 3]>,
        input: Res<Input<KeyCode>>,
    ){
        if local[0] == Attribute::NULL {
            local[0] = Attribute::from_str("ZombieState");
            local[1] = Attribute::new_index("Stand");
            local[2] = Attribute::new_index("Fall");
        }
        let (mut zstate, mut animation) = player.single_mut();
        for key in input.get_just_pressed() {
            match key {
                KeyCode::A | KeyCode::Left => {
                    if zstate.as_ref() == &ZState::Walking || zstate.as_ref() == &ZState::Idle {
                        animation.set_attribute(Attribute::FLIP_X, true);
                    }
                },
                KeyCode::D | KeyCode::Right => {
                    if zstate.as_ref() == &ZState::Walking || zstate.as_ref() == &ZState::Idle {
                        animation.set_attribute(Attribute::FLIP_X, false);
                    }
                },
                KeyCode::LShift | KeyCode::RShift | KeyCode::Up => {
                    *zstate = match zstate.as_ref() {
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
                KeyCode::Down | KeyCode::LControl | KeyCode::RControl => {
                    *zstate = match zstate.as_ref() {
                        ZState::Idle => {ZState::FallF},
                        ZState::Walking => {ZState::Idle},
                        ZState::Running => {ZState::Walking},
                        ZState::Attacking => {ZState::Attacking},
                        ZState::FallF => {ZState::FallF},
                        ZState::StandF => {let index = animation.get_attribute::<usize>(&local[1]);
                            animation.set_attribute(local[2], 6 - index); ZState::FallF},
                        ZState::FallB => {ZState::FallB},
                        ZState::StandB => {let index = animation.get_attribute::<usize>(&local[1]);
                            animation.set_attribute(local[2], 7 - index); ZState::FallB},
                        ZState::Test => {ZState::Test},
                        ZState::LayingB => {ZState::LayingB},
                        ZState::LayingF => {ZState::LayingF},
                    }
                },
                KeyCode::Space => {
                    *zstate = ZState::Attacking;
                },
                KeyCode::T => {
                    *zstate = ZState::Test;
                }
                _ => {}
            }
        }
        animation.set_attribute(local[0], *zstate);
    }
}

fn main() {
    App::new()
    .insert_resource(bevy::render::texture::ImageSettings::default_nearest())
    .add_plugins(DefaultPlugins)
    .add_plugin(animation::YourAnimationPlugin)
    .add_plugin(SpriteAnimationPlugin::<Zombie>::default())
    .add_startup_system(setup_animations)
    .add_plugin(player::Player)
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
    commands.spawn_bundle(Camera2dBundle::default());

    let nodes = nodes.as_mut();
    nodes.registor_node::<MatchNode::<ZState>>();

    let mut images = Vec::new();
    for i in 0..=67 {
        images.push(asset_server.load(&format!("Zombie1/Zombie1_{:05}.png", i)));
    }
    nodes.insert_node(NodeID::from_u64(0x3), Box::new(
        bevy_sprite_animation::nodes::IndexNode::new("test", &images, true)
    ));

    let fall_index = Attribute::new_index("Fall");
    let stand_index = Attribute::new_index("Stand");
    let attack_index = Attribute::new_index("Attack");

    if let Err(e) = nodes.load("test.node", &asset_server) {
        error!("{}", e)
    }

    if let Err(e) = nodes.load("./Zombie1.nodetree", &asset_server) {
        error!("{}", e)
    }

    let mut start = AnimationState::default();
    start.set_temporary(fall_index);
    start.set_temporary(stand_index);
    start.set_temporary(attack_index);
    commands.spawn_bundle(SpriteBundle{
        transform: Transform::from_translation(Vec3::X * 10.),
        sprite: Sprite{custom_size: Some(Vec2::splat(1000.)), ..Default::default()},
        ..Default::default()
    })
    .insert(Zombie)
    .insert(ZState::Attacking)
    .insert(start)
    .insert(player::Player)
    .insert(StartNode::from_str("0x0"));
}