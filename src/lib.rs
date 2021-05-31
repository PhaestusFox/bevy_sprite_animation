use std::marker::PhantomData;

use bevy::prelude::*;
use self::core_new::NodeTree;

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use nodes::TestNode;
    const DUPLICATE_NAME : &'static str = "duplicate";
    const UNIQUE_NAME : &'static str = "unique";
    const TEST_NODE_NAME : &'static str = "TestNode";
    struct TestFlag;
    #[test]
    fn nodetree_test(){
        let mut nodetree = NodeTree::<TestFlag>::default();
        let node = TestNode::new(DUPLICATE_NAME);
        nodetree.add(node);
        //returns true if no node with that name is in the tree
        let node_unique = TestNode::new(UNIQUE_NAME);
        assert!(nodetree.add(node_unique));
        //returns false if node is already in the tree
        let node_duplicate = TestNode::new(DUPLICATE_NAME);
        assert!(!nodetree.add(node_duplicate));
        let node_replace_duplicate = TestNode::new(DUPLICATE_NAME);
        assert!(nodetree.add_replace(node_replace_duplicate));
    }

    #[test]
    fn animator_test(){
        let mut node_tree = NodeTree::<TestFlag>::default();
        node_tree.add(TestNode::new(TEST_NODE_NAME));
        node_tree.add(TestNode::new_next(TEST_NODE_NAME));
        node_tree.add(TestNode::new_alt(TEST_NODE_NAME));
        node_tree.add(TestNode::new_temp(TEST_NODE_NAME));
        node_tree.add(TestNode::error());
        let mut animator = Animator::new(
            vec!["temp".to_string()],
            Frame::default(),
            TEST_NODE_NAME,
            30,
        );
        let mut out = String::new();
        for i in 0..10 {
            out.push_str(&format!("\nframe {}/10\n", i));
            let _frame = animator.next_frame(1, &node_tree);
            //out.push_str(animator.get_error_for_test());
            #[cfg(feature = "node_trace")]
            out.push_str(&animator.path_to_string(false));
            #[cfg(not(feature = "node_trace"))]
            out.push_str(&animator.get_string_for_test());
        }
        println!("{}",out);
        #[cfg(feature = "node_trace")]
        let out_const = "\nframe 0/10\nstart -> TestNode -> test_next_TestNode -> Frame -> End\nframe 1/10\nstart -> TestNode -> test_alt_TestNode -> Frame -> End\nframe 2/10\nstart -> TestNode -> test_temp_TestNode -> test_alt_TestNode -> Frame -> End\nframe 3/10\nstart -> TestNode -> test_temp_TestNode -> Frame -> End\nframe 4/10\nstart -> TestNode -> test_next_TestNode -> Frame -> End\nframe 5/10\nstart -> TestNode -> test_alt_TestNode -> Frame -> End\nframe 6/10\nstart -> TestNode -> test_temp_TestNode -> test_alt_TestNode -> Frame -> End\nframe 7/10\nstart -> TestNode -> test_temp_TestNode -> Frame -> End\nframe 8/10\nstart -> TestNode -> test_next_TestNode -> Frame -> End\nframe 9/10\nstart -> TestNode -> test_alt_TestNode -> Frame -> End";
        #[cfg(not(feature = "node_trace"))]
        let out_const = "\nframe 0/10\nFrame:0\nframe 1/10\nFrame:1\nframe 2/10\nFrame:2\nframe 3/10\nFrame:3\nframe 4/10\nFrame:0\nframe 5/10\nFrame:1\nframe 6/10\nFrame:2\nframe 7/10\nFrame:3\nframe 8/10\nFrame:0\nframe 9/10\nFrame:1";
        assert!(out == out_const);
    }

    
}

pub mod core_new; //dont mind the new part VSCode decided that it was not going to let me lower case the c and still work proper so i needed to rename it
pub mod nodes;
mod datatype_impl;
pub mod prelude{
    pub use super::AnimationPlugin;
    
    pub use super::nodes;
    pub use super::core_new;
    pub use super::core_new::NodeTree;
    pub use super::core_new::Animator;
    pub use super::core_new::DataType;
    pub use super::core_new::Cell;
    pub use super::core_new::NodeResult;
    pub use super::core_new::Frame;
}

pub struct AnimationPlugin<T: 'static + Send + Sync>(pub PhantomData<T>);
impl<T: 'static + Send + Sync> Default for AnimationPlugin<T> {
    fn default() -> Self {
        AnimationPlugin(PhantomData::<T>)
    }
}

impl<T: 'static + Send + Sync> Plugin for AnimationPlugin<T>{
    fn build(&self, app: &mut AppBuilder){
        app.add_system(animation_update::<T>.system());
        app.insert_resource(NodeTree::<T>::default());
        app.add_system_to_stage(CoreStage::First, NodeTree::<T>::build.exclusive_system());
    }
}

fn animation_update<T:'static + Send + Sync>(
    mut animators : Query<(&mut core_new::Animator, &mut TextureAtlasSprite, &mut Handle<TextureAtlas>), With<T>>,
    time : Res<Time>,
    node_tree : Res<NodeTree<T>>,
){
    let dt = time.delta_seconds();
    for (mut animator, mut sprite, mut texture) in animators.iter_mut(){
        animator.time_since_last_frame += dt;
        let frame_time = 1.0 / animator.frame_rate as f32;
        if animator.time_since_last_frame > frame_time{
            let frames = (animator.time_since_last_frame / frame_time).floor() ;
            animator.time_since_last_frame -= frame_time * frames;
            let next_frame = animator.next_frame(frames as usize, &node_tree);
            sprite.index = next_frame.index;
            if texture.id != next_frame.sprite_sheet.id {
                texture.set(Box::new(next_frame.sprite_sheet)).expect("Failed to set Handle<TextureAtlas>");
            }
        }
    }
}