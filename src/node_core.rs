use std::fmt::Debug;

use bevy::prelude::*;
use crate::{error::{BevySpriteAnimationError as Error, RunError}, prelude::*};

pub trait AnimationNodeTrait: Reflect
{
    fn run(&self, state: &mut super::state::AnimationState) -> Result<NodeResult, RunError>;
    fn name(&self) -> &str {
        self.reflect_short_type_path()
    }
    fn id(&self) -> NodeId<'_>;
    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, asset_server: &AssetServer) -> Result<(), Error> {
        let _ = asset_server;
        data.push_str(self.reflect_type_path());
        data.push_str("(serializetion not implemented)\n");
        Ok(())
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AnimationNode(")?;
        f.write_str(self.type_name())?;
        f.write_str(")")
    }

    #[cfg(feature = "dot")]
    #[allow(unused)]
    fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name()));
    }

    fn set_id(&mut self, id: NodeId<'_>);
}

#[derive(Debug)]
pub enum NodeResult {
    Next(NodeId<'static>),
    Done(Handle<Image>),
}

impl std::fmt::Display for NodeResult{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeResult::Next(id) => f.write_fmt(format_args!("Next({:#?})", id)),
            NodeResult::Done(_) => f.write_str("Done"),
        }
    }
}