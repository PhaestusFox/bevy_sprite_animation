use std::any::Any;

use bevy::prelude::*;
use crate::{error::BevySpriteAnimationError as Error, prelude::*};

pub trait AnimationNodeTrait: Send + Sync + Any
{
    fn run(&self, state: &mut super::state::AnimationState) -> NodeResult;
    fn name(&self) -> &str;
    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, context: &mut bevy_inspector_egui::Context) -> bool;
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
    #[cfg(feature = "hash")]
    fn hash(&self) -> u64;
}

pub trait CanLoad {
    fn loader() -> Box<dyn NodeLoader>;
}

#[derive(Debug)]
pub enum NodeResult {
    Next(NodeId<'static>),
    Done(Handle<Image>),
    Error(String),
}

impl std::fmt::Display for NodeResult{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeResult::Next(id) => f.write_fmt(format_args!("Next({:#?})", id)),
            NodeResult::Done(_) => f.write_str("Done"),
            NodeResult::Error(_) => f.write_str("Error"),
        }
    }
}

pub trait NodeLoader: 'static + Send + Sync {
    fn load(&mut self, data: &str, asset_server: &AssetServer) -> Result<Box<dyn AnimationNodeTrait>, crate::error::BevySpriteAnimationError>;
    fn can_load(&self) -> &[&str];
}