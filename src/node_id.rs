#[derive(Debug, Hash, serde::Serialize, serde::Deserialize, Reflect)]
pub enum NodeId<'a> {
    Name(Cow<'a, str>),
    U64(u64),
    NameHash(u64),
}

impl Eq for NodeId<'_> {}

impl PartialEq for NodeId<'_> {
    fn eq(&self, other: &Self) -> bool {
        let id = match self {
            NodeId::U64(id) => return if let NodeId::U64(other) = other {*id == *other} else {false},
            NodeId::Name(name) => NodeId::hash_name(name),
            NodeId::NameHash(id) => *id,
        };
        let other = match other {
            NodeId::U64(_) => return false,
            NodeId::Name(name) => NodeId::hash_name(name),
            NodeId::NameHash(id) => *id,
        };
        id == other
    }
}

impl std::fmt::Display for NodeId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       match self {
            NodeId::Name(name) =>  f.write_fmt(format_args!("NodeName({})", name)),
            NodeId::U64(id) =>  f.write_fmt(format_args!("NodeId({:#018X})", id)),
            NodeId::NameHash(id) =>  f.write_fmt(format_args!("NodeName({:018x})", id)),
        }
    }
}

impl Default for NodeId<'_> {
    fn default() -> Self {
        NodeId::U64(0)
    }
}

impl NodeId<'_> {
    const FROM_ID: uuid::Uuid = uuid::uuid!("8ec27710-7e5d-4f0c-864a-49e403dad6a1");
    const FROM_NAME: uuid::Uuid = uuid::uuid!("559dd81c-ec17-4c83-b3f2-eb7471d64d76");
}

impl From<NodeId<'_>> for bevy::asset::HandleId {
    fn from(value: NodeId) -> Self {
        match value {
            NodeId::Name(name) => bevy::asset::HandleId::Id(NodeId::FROM_NAME, NodeId::hash_name(&name)),
            NodeId::U64(id) => bevy::asset::HandleId::Id(NodeId::FROM_ID, id),
            NodeId::NameHash(id) => bevy::asset::HandleId::Id(NodeId::FROM_NAME, id),
        }
    }
}

use std::{borrow::Cow, hash::Hasher};

use bevy::reflect::Reflect;

impl std::str::FromStr for NodeId<'_> {
    type Err = std::num::ParseIntError; //todo!
    fn from_str(s: &str) -> Result<Self, Self::Err> {
            println!("NodeId::from_str: {}", s);
            let data = s.trim();
            let data = if let Some(data) = data.strip_prefix("NodeId(") {
                if !data.ends_with(')') {
                    panic!("NodeId: started with 'NodeId(' but did not end with ')'");
                }
                NodeId::from_u64(if let Some(data) = data.strip_prefix("0x") {
                    println!("NodeId::from_str hex: {}", &data[..data.len()-1]);
                    u64::from_str_radix(&data[..data.len() -1], 16)?
                } else if let Some(data) = data.strip_prefix("0b") {
                    println!("NodeId::from_str bin: {}", &data[..data.len()-1]);
                    u64::from_str_radix(&data[..data.len() -1], 2)?
                } else {
                    println!("NodeId::from_str dec: {}", &data[..data.len()-1]);
                    data[..data.len()-1].parse()?
                })
            } else if let Some(data) = data.strip_prefix("NodeName(") {
                if !data.ends_with(')') {
                    panic!("NodeName: started with 'NodeName(' but did not end with ')'");
                }
                if data.starts_with(|c: char| c.is_numeric()) {
                    NodeId::NameHash(if let Some(data) = data.strip_prefix("0x") {
                        u64::from_str_radix(&data[..data.len() - 1], 16)?
                    } else if let Some(data) = data.strip_prefix("0b") {
                        u64::from_str_radix(&data[..data.len()-1], 2)?
                    } else {
                        data[..data.len()-1].parse()?
                    })
                } else {
                    NodeId::Name(String::from(&data[..data.len()-1]).into())
                }
            } else {
                panic!("Must start with Nodexx")
            };
            Ok(data)
    }
}

impl NodeId<'_> {
    pub fn from_u64(id: u64) -> Self {
        NodeId::U64(id)
    }

    pub fn hash_name(name: &Cow<'_, str>) -> u64 {
        pub(crate) fn get_hasher() -> bevy::utils::AHasher {
            use std::hash::BuildHasher;
            bevy::utils::RandomState::with_seeds(42, 23, 13, 8).build_hasher()
        }
        let mut hasher = get_hasher();
        std::hash::Hash::hash(&name, &mut hasher);
        hasher.finish()
    }
}

impl<'a> NodeId<'a> {
    pub fn from_name(name: impl Into<Cow<'a, str>>) -> NodeId<'a> {
        NodeId::Name(name.into())
    }
}

impl<'a> NodeId<'a> {
    pub fn to_static(&self) -> NodeId<'static> {
        match self {
            NodeId::Name(name) => NodeId::NameHash(NodeId::hash_name(name)),
            NodeId::U64(id) => NodeId::U64(*id),
            NodeId::NameHash(id) => NodeId::NameHash(*id),
        }
    }
}

impl Clone for NodeId<'_> {
    fn clone(&self) -> Self {
        self.to_static()
    }
}

#[cfg(feature = "bevy-inspector-egui")]
impl bevy_inspector_egui::Inspectable for NodeId {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _: Self::Attributes, _: &mut bevy_inspector_egui::Context) -> bool {
        ui.label(self.to_string());
        false
    }
}
