use std::{any::Any, fmt::Debug};

use bevy::prelude::*;
use crate::{error::{BevySpriteAnimationError as Error, RunError}, prelude::*};

pub trait AnimationNodeTrait: Send + Sync + Any + AnimationNodeAsAny
{
    fn run(&self, state: &mut super::state::AnimationState) -> Result<NodeResult, RunError>;
    fn name(&self) -> &str;
    fn id(&self) -> NodeId<'_>;
    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, asset_server: &AssetServer) -> Result<(), Error> {
        let _ = asset_server;
        data.push_str("serializetion for ");
        data.push_str(&self.node_type());
        data.push_str(" not implemented\n");
        Ok(())
    }
    fn node_type(&self) -> String;

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("You can impl AnimationNodeTrait::debug for more info\n")?;
        f.write_str("Node: \n")?;
        f.write_str(self.name())
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name()));
    }
}

impl Debug for dyn AnimationNodeTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug(f)
    }
}

impl dyn AnimationNodeTrait {
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }
}

pub trait AnimationNodeAsAny: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AnimationNodeAsAny for T {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

pub trait CanLoad {
    fn loader() -> Box<dyn NodeLoader>;
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

pub trait NodeLoader: 'static + Send + Sync {
    fn load(&self, data: &str, asset_server: &AssetServer) -> Result<Box<dyn AnimationNodeTrait>, crate::error::BevySpriteAnimationError>;
    fn can_load(&self) -> &[&str];
}