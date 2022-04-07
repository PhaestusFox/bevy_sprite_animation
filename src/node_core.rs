use bevy::prelude::*;
use crate::error::BevySpriteAnimationError as Error;

pub trait AnimationNode: Send + Sync
{
    fn run(&self, state: &mut super::state::AnimationState) -> NodeResult;
    fn name(&self) -> &str;
    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, context: &mut bevy_inspector_egui::Context) -> bool;
    fn id(&self) -> NodeID {
        self.name().into()
    }
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

#[derive(Debug, Default ,Hash, PartialEq, Eq, Clone, Copy, Reflect)]
pub struct NodeID(
    u64
);

impl NodeID {
    pub fn as_u64(self) -> u64 {
        self.0
    }
    pub fn from_u64(id: u64) -> Self {
        NodeID(id)
    }
    pub fn from_str(id: &str) -> NodeID {
        let id = if id.starts_with("NodeID(") {
            &id[7..id.len()-1]
        } else {
            id
        };
        if id.starts_with("0") {
        if id[1..].starts_with(|c: char| c == 'x' || c == 'X') {
            NodeID(u64::from_str_radix(&id[2..], 16).unwrap())
        } else {
            NodeID(u64::from_str_radix(id, 10).unwrap())
        }
        } else {
            use std::hash::Hash;
            use std::hash::Hasher;
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            id.hash(&mut hasher);
            NodeID(hasher.finish())
        }
    }
}

#[cfg(feature = "serialize")]
mod serde {
    use serde::{Serialize, Deserialize};
    #[derive(Serialize, Deserialize)]
    struct NodeID(String);
    
    impl Serialize for super::NodeID {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        NodeID(format!("{:#020X}", self.0)).serialize(serializer)
    }
    }

    impl<'de> Deserialize<'de> for super::NodeID {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
            let r = NodeID::deserialize(deserializer)?;
            let res = u64::from_str_radix(&r.0[2..], 16);
            if let Ok(id) = res {Ok(Self(id))} else {bevy::log::error!("NodeID deserialize error {:?}", res); Ok(Self(0))}
        }
    }
}


#[cfg(feature = "bevy-inspector-egui")]
impl bevy_inspector_egui::Inspectable for NodeID {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _: Self::Attributes, _: &mut bevy_inspector_egui::Context) -> bool {
        ui.label(self.to_string());
        false
    }
}

impl std::fmt::Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("NodeID({:#020X})",self.0))
    }
}

impl Into<NodeID> for &str{
    fn into(self) -> NodeID {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        self.hash(&mut hasher);
        NodeID(hasher.finish())
    }
}

impl Into<NodeID> for String {
    fn into(self) -> NodeID {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        self.hash(&mut hasher);
        NodeID(hasher.finish())
    }
}

#[derive(Debug)]
pub enum NodeResult {
    Next(NodeID),
    Done(Handle<Image>),
    Error(String),
}

impl std::fmt::Display for NodeResult{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeResult::Next(id) => f.write_fmt(format_args!("Next({:#x})", id.0)),
            NodeResult::Done(_) => f.write_str("Done"),
            NodeResult::Error(_) => f.write_str("Error"),
        }
    }
}

pub trait NodeLoader: 'static + Send + Sync {
    fn load(&mut self, data: &str, asset_server: &AssetServer) -> Result<Box<dyn AnimationNode>, crate::error::BevySpriteAnimationError>;
    fn can_load(&self) -> &[&str];
}