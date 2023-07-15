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
        Err(RunError::Custom("Reference Node should not be root of a tree".to_string()))
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
    
    fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &bevy::prelude::AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label={:?}];\n", self.1));
        this.dot(out);
        out.push_str(" [color=brown];\n");
        for node in self.0.iter() {
            this.dot(out);
            out.push_str(" -> ");
            crate::dot::handle_to_node(node.id()).dot(out);
            out.push(';');
            out.push('\n');
        }
    }
}