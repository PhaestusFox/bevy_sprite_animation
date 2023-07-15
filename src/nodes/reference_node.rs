use std::path::PathBuf;

use bevy::prelude::Handle;

use crate::{AnimationNode, prelude::*};

pub struct ReferenceNode(pub Vec<Handle<AnimationNode>>, pub PathBuf);

impl ReferenceNode {
    pub fn iter(&self) -> impl Iterator<Item = &Handle<AnimationNode>> {
        self.0.iter()
    }
}

impl AnimationNodeTrait for ReferenceNode {
    fn run(&self, _: &mut crate::state::AnimationState) -> Result<NodeResult, RunError> {
        Err(RunError::Custom("Reference Node should not be part of the tree".to_string()))
    }

    fn name(&self) -> &str {
        "Reference Node;\n
        this it used to stop Nodes Unloading as soon as they are loaded drop its handle when you are done with the file it represents"
    }

    fn id(&self) -> crate::prelude::NodeId<'_> {
        crate::prelude::NodeId::from_name("Reference Node Dont Point to Me")
    }

    fn node_type(&self) -> String {
        "Reference Node".to_string()
    }
}