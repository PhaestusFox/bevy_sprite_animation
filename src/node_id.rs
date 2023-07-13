#[derive(Debug, Hash, Reflect)]
pub enum NodeId<'a> {
    Name(Cow<'a, str>),
    U64(u64),
    Hash(u64),
}

impl<'de> serde::Deserialize<'de> for NodeId<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_enum(SERDE_NAME, &["Id", "Name"], NodeVisitor)
    }
}
const SERDE_NAME: &'static str = "Node";

impl serde::Serialize for NodeId<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        match self {
            NodeId::Name(name) => serializer.serialize_newtype_variant(SERDE_NAME, Variant::Name as u32, Variant::Name.as_ref(), name),
            NodeId::U64(id) => serializer.serialize_newtype_variant(SERDE_NAME, Variant::Id as u32, Variant::Id.as_ref(), id),
            NodeId::Hash(hash) => serializer.serialize_newtype_variant(SERDE_NAME, Variant::Name as u32, Variant::Name.as_ref(), hash),
        }
    }
}

#[derive(Debug, serde::Deserialize, strum_macros::AsRefStr)]
enum Variant {
    Name,
    Id,
}

struct NodeVisitor;
impl<'de> serde::de::Visitor<'de> for NodeVisitor {
    type Value = NodeId<'static>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expect Variant Id(u64) or Name(String | U64)")
    }
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::EnumAccess<'de>, {
                use serde::de::VariantAccess;
        let v = data.variant::<Variant>()?;
        match v.0 {
            Variant::Name => v.1.newtype_variant_seed(NodeVisitor),
            Variant::Id => Ok(NodeId::U64(v.1.newtype_variant::<u64>()?)),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(NodeId::Name(v.into()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(NodeId::Hash(v as u64))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(v.to_owned())
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for NodeVisitor {
    type Value = NodeId<'static>;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de> {
        Ok(deserializer.deserialize_any(self).unwrap())
    }
}

#[test]
fn test_serde() {
    let ser_u64 = ron::to_string(&NodeId::U64(2)).expect("ron to work");
    let ser_name = ron::to_string(&NodeId::from_name("Two")).expect("ron to work");
    let ser_hash = ron::to_string(&NodeId::Hash(2)).expect("ron to work");

    assert_eq!(ser_u64, "Id(2)");
    assert_eq!(ser_name, "Name(\"Two\")");
    assert_eq!(ser_hash, "Name(2)");
    
    assert_eq!(Ok(NodeId::U64(2)), ron::from_str(&ser_u64));
    assert_eq!(Ok(NodeId::from_name("Two")), ron::from_str(&ser_name));
    assert_eq!(Ok(NodeId::Hash(2)), ron::from_str(&ser_hash));
}

#[test]
fn assert_eq() {
    assert_eq!(NodeId::U64(0), NodeId::U64(0));
    assert_ne!(NodeId::U64(0), NodeId::U64(1));
    assert_ne!(NodeId::U64(0), NodeId::from_name("Test"));
    let u64 = NodeId::U64(0);
    assert_eq!(u64.to_static(), u64);
    assert_eq!(u64.to_static(), u64.to_static());
    let name = NodeId::from_name("Test");
    let name_hash = NodeId::Hash(10729580169200549928);
    assert_eq!(name.to_static(), name);
    assert_eq!(NodeId::from_name("Test"), NodeId::from_name(String::from("Test")));
    assert_eq!(NodeId::from_name("Test"), NodeId::Hash(10729580169200549928));
    assert_eq!(NodeId::Hash(10729580169200549928), NodeId::from_name("Test"));
    assert_eq!(NodeId::Hash(10729580169200549928), NodeId::Hash(10729580169200549928));
    assert_eq!(name.to_static(), NodeId::from_name("Test"));
    assert_eq!(name.to_static(), NodeId::Hash(10729580169200549928));
    assert_eq!(NodeId::from_name("Test"), name.to_static());
    assert_eq!(NodeId::Hash(10729580169200549928), name.to_static());
    assert_ne!(u64.to_static(), name.to_static());
    assert_eq!(name, name_hash);
    assert_eq!(name_hash, name);
    assert_eq!(name_hash.to_static(), name_hash);
    assert_eq!(name.to_static(), name);
    assert_eq!(name_hash.to_static(), name.to_static());
}

impl Eq for NodeId<'_> {}

impl PartialEq for NodeId<'_> {
    fn eq(&self, other: &Self) -> bool {
        let id = match self {
            NodeId::U64(id) => return if let NodeId::U64(other) = other {*id == *other} else {false},
            NodeId::Name(name) => NodeId::hash_name(name),
            NodeId::Hash(id) => *id,
        };
        let other = match other {
            NodeId::U64(_) => return false,
            NodeId::Name(name) => NodeId::hash_name(name),
            NodeId::Hash(id) => *id,
        };
        id == other
    }
}

impl std::fmt::Display for NodeId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       match self {
            NodeId::Name(name) =>  f.write_fmt(format_args!("NodeName(\"{}\")", name)),
            NodeId::U64(id) =>  f.write_fmt(format_args!("NodeId({})", id)),
            NodeId::Hash(id) =>  f.write_fmt(format_args!("NodeName({})", id)),
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
            NodeId::Hash(id) => bevy::asset::HandleId::Id(NodeId::FROM_NAME, id),
        }
    }
}

use std::{borrow::Cow, hash::Hasher};

use bevy::reflect::Reflect;

impl std::str::FromStr for NodeId<'_> {
    type Err = std::num::ParseIntError; //todo!
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
                    NodeId::Hash(if let Some(data) = data.strip_prefix("0x") {
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

    pub fn hash_name<T: AsRef<str>>(name: T) -> u64 {
        pub(crate) fn get_hasher() -> bevy::utils::AHasher {
            use std::hash::BuildHasher;
            bevy::utils::RandomState::with_seeds(42, 23, 13, 8).build_hasher()
        }
        let mut hasher = get_hasher();
        let name = name.as_ref();
        std::hash::Hash::hash(name, &mut hasher);
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
            NodeId::Name(name) => NodeId::Hash(NodeId::hash_name(name)),
            NodeId::U64(id) => NodeId::U64(*id),
            NodeId::Hash(id) => NodeId::Hash(*id),
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
