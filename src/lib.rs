use bevy::prelude::*;

#[cfg(test)]
mod tests {
    //use bevy::prelude::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    /*pub(super) fn animation_test(
        texture_atlases: Res<Assets<TextureAtlas>>,
        animators : Query<&super::Animator>,
    ){
        for animator in animators.iter(){
            for animation in animator.animations.iter(){
                for frame in animation.frames.iter(){
                    let ta = texture_atlases.get(&frame.sprite_sheet).unwrap();
                    assert!(frame.sprite_index < ta.textures.len())
                }
            }
        }
    }*/
}

//mod intern;
#[allow(dead_code)]
pub mod core;
pub mod nodes;
pub mod prelude{
    pub use super::core::DataType;
    pub use super::core::Frame;
    pub use super::core::Cell;
    pub use super::core::Animator;
    pub use super::core::NodeTree;
    pub use super::AnimationPlugin;
    pub use super::nodes;
}
mod serde;
#[allow(dead_code,non_snake_case)]
mod Core;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin{
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(core::animation_update.system());
        app.insert_resource(core::NodeTree::default());
    }
}