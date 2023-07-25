use std::path::PathBuf;

use bevy::{
    prelude::{warn, Handle},
    reflect::Reflect,
};

use crate::{prelude::*, utils::get_node_hash, AnimationNode};

/// this is used to stop Nodes Unloading as soon as they are loaded drop its handle when you are done with the file it represents
#[derive(Reflect)]
pub struct ReferenceNode(pub Vec<Handle<AnimationNode>>, pub PathBuf);

impl ReferenceNode {
    pub fn iter(&self) -> impl Iterator<Item = &Handle<AnimationNode>> {
        self.0.iter()
    }
}

impl AnimationNodeTrait for ReferenceNode {
    fn run(&self, _: &mut crate::state::AnimationState) -> Result<NodeResult, RunError> {
        if let Some(id) = self.0.first() {
            Ok(NodeResult::Next(handle_to_node(id.id())))
        } else {
            Err(RunError::Custom("No Nodes in ReferenceNode".to_string()))
        }
    }

    fn name(&self) -> &str {
        "Reference Node"
    }

    fn id(&self) -> crate::prelude::NodeId<'_> {
        crate::prelude::NodeId::from_u64(get_node_hash(&self.1))
    }

    fn set_id(&mut self, _: NodeId<'_>) {
        warn!("Can't Set Id of a ReferenceNode");
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, _: &bevy::prelude::AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label={:?}];\n", self.1));
        this.dot(out);
        out.push_str(" [color=brown];\n");
        for (i, node) in self.0.iter().enumerate() {
            this.dot(out);
            out.push_str(" -> ");
            handle_to_node(node.id()).dot(out);
            if i == 0 {
                out.push_str(";\n");
            } else {
                out.push_str("[style=dotted];\n");
            }
        }
    }
}
