use bevy::prelude::*;
use bevy_sprite_animation::prelude::*;

use animation::ZState;

mod animation {
use bevy_sprite_animation::prelude::*;

use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(zombie_state_update.before("AnimationUpdate"));
        app.add_system(zombie_update_state.after("AnimationUpdate"));
        app.add_system(zombie_type_update.before("AnimationUpdate"));
    }
}

#[derive(Debug, Component, Hash, PartialEq, Eq, Clone, Copy, Reflect,
bevy_inspector_egui::Inspectable, serde::Serialize, serde::Deserialize,
PartialOrd, Ord)]
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

fn zombie_state_update(
    mut zombies: Query<(&mut AnimationState, &ZState), With<super::Zombie>>
) {
    let att = Attribute::from_str("ZombieState");
    for (mut state, name) in zombies.iter_mut() {
        state.set_attribute(att, name);
    }
}

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

fn zombie_type_update(
    mut zombies: Query<(&mut AnimationState, &super::Zombie)>
){
    let att = Attribute::new_attribute("ZombieType");
    for (mut state, zombie) in zombies.iter_mut() {
        state.set_attribute(att, zombie.0);
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
                        ZState::Test => todo!(),
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
                        ZState::StandF => {let index = animation.get_attribute::<usize>(&local[1]); animation.set_attribute(local[2], index); ZState::FallF},
                        ZState::FallB => {ZState::FallB},
                        ZState::StandB => {let index = animation.get_attribute::<usize>(&local[1]); animation.set_attribute(local[2], index); ZState::FallB},
                        ZState::Test => todo!(),
                        ZState::LayingB => {ZState::LayingB},
                        ZState::LayingF => {ZState::LayingF},
                    }
                },
                KeyCode::Space => {
                    *zstate = ZState::Attacking;
                },
                _ => {}
            }
        }
        animation.set_attribute(local[0], *zstate);
    }
}

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(animation::AnimationPlugin)
    .add_plugin(SpriteAnimationPlugin::<Zombie>::default())
    .add_startup_system(setup_animations)
    .add_plugin(player::Player)
    .run()
}

use bevy_inspector_egui::Inspectable;
#[derive(Component, Default, Inspectable, Reflect)]
#[reflect(Component)]
struct Zombie(
    #[inspectable(min = 1, max = 8)]
    usize);

fn setup_animations(
    mut commands: Commands,
    mut nodes: ResMut<AnimationNodeTree<Zombie>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let nodes = nodes.as_mut();
    nodes.registor_node::<MatchNode::<ZState>>();

    println!("\n\n{}\n\n", std::any::type_name::<MatchNode::<ZState>>());

    let fall_index = Attribute::new_index("Fall");
    let stand_index = Attribute::new_index("Stand");
    let attack_index = Attribute::new_index("Attack");

    if let Err(e) = nodes.load("test.node", &asset_server) {
        error!("{}", e)
    }

    for i in 1..=1{
        if let Err(e) = nodes.load(&format!("./Zombie{}.nodetree",i), &asset_server) {
            error!("{}", e)
        }   
    }

    let mut start = AnimationState::default();
    start.set_temporary(fall_index);
    start.set_temporary(stand_index);
    start.set_temporary(attack_index);
    commands.spawn_bundle(SpriteBundle{
        transform: Transform::from_translation(Vec3::X * 10.),
        ..Default::default()
    })
    .insert(Zombie(1))
    .insert(ZState::Attacking)
    .insert(start)
    .insert(player::Player)
    .insert(StartNode::from_str("0x0"));
}