use std::borrow::Cow;
use std::hash::Hash;
use std::hash::Hasher;

use bevy::reflect::Reflect;
use bevy::reflect::ReflectDeserialize;
use bevy::reflect::ReflectSerialize;

use crate::utils::get_hash;

#[derive(Default, Reflect, PartialOrd, Ord, strum_macros::AsRefStr)]
#[reflect_value(Serialize, Deserialize)]
pub enum Attribute {
    Custom(u64, Cow<'static, str>),
    CustomId(u64),
    #[default]
    Default,
    Delta,
    Frames,
    TimeThisFrame,
    FlipX,
    FlipY,
    LastFPS,
    Index(u64, Cow<'static, str>),
    IndexId(u64),
}

impl Hash for Attribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Attribute::Index(id, _) |
            Attribute::IndexId(id) => {state.write_u8(u8::MAX - 1); state.write_u64(*id)},
            Attribute::CustomId(id) |
            Attribute::Custom(id, _) => {state.write_u8(u8::MAX); state.write_u64(*id)},
            _ => core::mem::discriminant(self).hash(state),
        }
    }
}

impl Eq for Attribute {}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Custom(l0, _), Self::Custom(r0, _)) |
            (Self::Custom(l0, _), Self::CustomId(r0)) | 
            (Self::CustomId(l0), Self::Custom(r0,_)) |
            (Self::CustomId(l0), Self::CustomId(r0)) |
            (Self::Index(l0, _), Self::Index(r0, _)) | 
            (Self::Index(l0, _), Self::IndexId(r0)) |
            (Self::IndexId(l0), Self::Index(r0, _)) |
            (Self::IndexId(l0), Self::IndexId(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Clone for Attribute {
    fn clone(&self) -> Self {
        match self {
            Self::Custom(id, _) => Self::CustomId(*id),
            Self::CustomId(arg0) => Self::CustomId(arg0.clone()),
            Self::Default => Self::Default,
            Self::Delta => Self::Delta,
            Self::Frames => Self::Frames,
            Self::TimeThisFrame => Self::TimeThisFrame,
            Self::FlipX => Self::FlipX,
            Self::FlipY => Self::FlipY,
            Self::LastFPS => Self::LastFPS,
            Self::Index(id, _) => Self::IndexId(*id),
            Self::IndexId(arg0) => Self::IndexId(arg0.clone()),
        }
    }
}

impl Attribute {
    #[inline(always)]
    pub fn is_core(&self) -> bool {
        match self {
            Attribute::Custom(_, _) |
            Attribute::CustomId(_) |
            Attribute::Index(_, _) |
            Attribute::IndexId(_) => false,
            _ => true
        }
    }
    #[inline(always)]
    pub fn is_index(&self) -> bool {
        match self {
            Attribute::Index(_, _) | Attribute::IndexId(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    pub fn is_custom(&self) -> bool {
        match self {
            Attribute::Custom(_, _) | Attribute::CustomId(_) => true,
            _ => false
        }
    }

    pub fn deep_clone(&self) -> Self {
        match self {
            Self::Custom(id, arg0) => Self::Custom(*id, arg0.clone()),
            Self::CustomId(arg0) => Self::CustomId(arg0.clone()),
            Self::Default => Self::Default,
            Self::Delta => Self::Delta,
            Self::Frames => Self::Frames,
            Self::TimeThisFrame => Self::TimeThisFrame,
            Self::FlipX => Self::FlipX,
            Self::FlipY => Self::FlipY,
            Self::LastFPS => Self::LastFPS,
            Self::Index(id, arg0) => Self::Index(*id, arg0.clone()),
            Self::IndexId(arg0) => Self::IndexId(arg0.clone()),
        }
    }

    /// turns a &str into a Attribute by attemting to deserialize it with ron
    /// if this failes it will just return Attribute::Custom(s.to_string().into())
    pub fn from_str(s: &str) -> Attribute {
        match ron::from_str::<Attribute>(s) {
            Ok(ok) => ok,
            Err(_) => {
                Attribute::Custom(get_hash(&s), s.to_string().into())
            },
        }
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        match self {
            Attribute::Custom(_, name) |
            Attribute::Index(_, name) => {
                f.write_str("::")?;
                f.write_str(name.as_ref())
            }
            Attribute::CustomId(_) |
            Attribute::IndexId(_) => {
                f.write_str("(")?;
                f.write_str(self.name_or_id().as_ref())?;
                f.write_str(")")
            },
            _ => {Ok(())}
        }
    }
}

impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        match self {
            Attribute::Custom(_, _) |
            Attribute::CustomId(_) |
            Attribute::Index(_, _) |
            Attribute::IndexId(_) => {
                f.write_str("(")?;
                f.write_str(self.name_or_id().as_ref())?;
                f.write_str(")")
            },
            _ => Ok(())
        }
    }
}

impl Attribute {
    /// Returns the Attribute::Index() for the given name
    /// The Attribute will have a name
    /// If you .deep_clone it the name will stay
    /// Rust doesn't let you have diffrent behaviure for clone and copy :'[ or even imle your own copy :(
    /// If you Clone it the name will be hashed to save allocateing stings all the time
    #[inline(always)]
    pub fn new_index(name: impl Into<Cow<'static, str>>) -> Attribute{
        let name = name.into();
        Attribute::Index(get_hash(&name), name)
    }

    /// Returns the Attribute::Custom() for the given name
    /// The Attribute will have a name
    /// If you .deep_clone it the name will stay
    /// Rust doesn't let you have diffrent behaviure for clone and copy :'[ or even imle your own copy :(
    /// If you Clone it the name will be hashed to save allocateing stings all the time
    #[inline(always)]
    pub fn new_attribute(name: impl Into<Cow<'static, str>>) -> Attribute{
        let name = name.into();
        Attribute::Custom(get_hash(&name), name)
    }

    /// Returns the Attribute::CustomId() for the given name
    /// The Attribute will **not** have a name
    pub fn new_attribute_id<T: Hash>(id: &T) -> Attribute {
        Attribute::CustomId(get_hash(id))
    }

    /// Will Return the id of an Attribute
    /// #Returns
    /// `None` - If Core Attribute
    /// `Some(id)` - If CustomId or IndexId
    /// `Some(hash)` - If Custom or Index
    pub fn get_id(&self) -> Option<u64> {
        match self {
            Attribute::CustomId(id) |
            Attribute::IndexId(id) |
            Attribute::Custom(id, _) |
            Attribute::Index(id, _) => Some(*id),
            _ => None
        }
    }

    /// Returns the Attribute::IndexId() for the given name
    /// The Attribute will **not** have a name
    pub fn new_index_id<T: Hash>(id: &T) -> Attribute {
        Attribute::IndexId(get_hash(id))
    }
    
    /// Returns the name of the Attribute or Index will return None if the name has been errased
    pub fn name(&self) -> Option<&str> {
        match self {
            Attribute::Custom(_, name) |
            Attribute::Index(_, name) => Some(name.as_ref()),
            Attribute::CustomId(_) |
            Attribute::IndexId(_) => None,
            e => Some(e.as_ref())
        }
    }

    /// Returns the name of the Attribute or Index or its hash id if the name has been errased
    /// You should used name if you only want the name since this will allocate as string to hold the id
    pub fn name_or_id(&self) -> Cow<'_, str> {
        if let Some(name) = self.name() {
            name.into()
        } else {
            match self {
                Attribute::CustomId(id) |
                Attribute::IndexId(id) => format!("{}", id).into(),
                _ => unreachable!("all named variants should return Some(&str) from self.name()")
            }
        }
    }

    fn get_variant(&self) -> Variant {
        match self {
            Attribute::Custom(_, _) => Variant::Custom,
            Attribute::CustomId(_) => Variant::Custom,
            Attribute::Default => Variant::Default,
            Attribute::Delta => Variant::Delta,
            Attribute::Frames => Variant::Frames,
            Attribute::TimeThisFrame => Variant::TimeThisFrame,
            Attribute::FlipX => Variant::FlipX,
            Attribute::FlipY => Variant::FlipY,
            Attribute::LastFPS => Variant::LastFPS,
            Attribute::Index(_, _) => Variant::Index,
            Attribute::IndexId(_) => Variant::Index,
        }
    }
}

impl<'de> serde::Deserialize<'de> for Attribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_enum(SERDE_NAME, &[], AttributeVisitor::Custom)
    }
}
const SERDE_NAME: &'static str = "Attribute";

impl serde::Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
                let variant = self.get_variant();
                let variant_index = variant as u32;
                let variant_name: &'static str = variant.into();
        match self {
            Attribute::Custom(_, name) => serializer.serialize_newtype_variant(SERDE_NAME, variant_index, variant_name, name),
            Attribute::CustomId(id) => serializer.serialize_newtype_variant(SERDE_NAME, variant_index, variant_name, id),
            Attribute::Index(_, name) => serializer.serialize_newtype_variant(SERDE_NAME, variant_index, variant_name, name),
            Attribute::IndexId(id) => serializer.serialize_newtype_variant(SERDE_NAME, variant_index, variant_name, id),
            _ => serializer.serialize_unit_variant(SERDE_NAME, variant_index, variant_name),
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize, strum_macros::AsRefStr, strum_macros::IntoStaticStr)]
enum Variant {
    Custom,
    Default,
    Delta,
    Frames,
    TimeThisFrame,
    FlipX,
    FlipY,
    LastFPS,
    Index,
}

enum AttributeVisitor {
    Custom,
    Index,
}
impl<'de> serde::de::Visitor<'de> for AttributeVisitor {
    type Value = Attribute;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expect Variant Id(u64) or Name(String | U64)")
    }
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::EnumAccess<'de>, {
                use serde::de::VariantAccess;
        let v = data.variant::<Variant>()?;
        Ok(match v.0 {
            Variant::Custom => v.1.newtype_variant_seed(AttributeVisitor::Custom)?,
            Variant::Default => Attribute::Default,
            Variant::Delta => Attribute::Delta,
            Variant::Frames => Attribute::Frames,
            Variant::TimeThisFrame => Attribute::TimeThisFrame,
            Variant::FlipX => Attribute::FlipX,
            Variant::FlipY => Attribute::FlipY,
            Variant::LastFPS => Attribute::LastFPS,
            Variant::Index => v.1.newtype_variant_seed(AttributeVisitor::Index)?,
        })
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
                Ok(match self {
                    AttributeVisitor::Custom => Attribute::Custom(get_hash(&v), v.into()),
                    AttributeVisitor::Index => Attribute::Index(get_hash(&v), v.into()),
                })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
                Ok(match self {
                    AttributeVisitor::Custom => Attribute::CustomId(v),
                    AttributeVisitor::Index => Attribute::IndexId(v),
                })
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(v.to_owned())
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for AttributeVisitor {
    type Value = Attribute;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de> {
        Ok(deserializer.deserialize_any(self).unwrap())
    }
}


#[test]
fn test_serde() {
    let ser_cid = ron::to_string(&Attribute::CustomId(2)).expect("ron to work");
    let ser_cname = ron::to_string(&Attribute::new_attribute("Two")).expect("ron to work");
    let ser_iid = ron::to_string(&Attribute::IndexId(2)).expect("ron to work");
    let ser_iname = ron::to_string(&Attribute::new_index("Two")).expect("ron to work");
    let ser_v = ron::to_string(&Attribute::Default).expect("ron to work");

    assert_eq!(ser_cid, "Custom(2)");
    assert_eq!(ser_cname, "Custom(\"Two\")");
    assert_eq!(ser_iid, "Index(2)");
    assert_eq!(ser_iname, "Index(\"Two\")");
    assert_eq!(ser_v, "Default");
    
    assert_eq!(Ok(Attribute::CustomId(2)), ron::from_str(&ser_cid));
    assert_eq!(Ok(Attribute::new_attribute("Two")), ron::from_str(&ser_cname));
    assert_eq!(Ok(Attribute::IndexId(2)), ron::from_str(&ser_iid));
    assert_eq!(Ok(Attribute::new_index("Two")), ron::from_str(&ser_iname));
    assert_eq!(Ok(Attribute::Default), ron::from_str(&ser_v));
}

#[test]
fn assert_eq() {
    // test custom
    assert_eq!(Attribute::CustomId(0), Attribute::CustomId(0));
    assert_ne!(Attribute::CustomId(0), Attribute::CustomId(1));
    assert_ne!(Attribute::CustomId(0), Attribute::new_attribute("Test"));
    let name = Attribute::new_attribute("Test");
    let name_hash = Attribute::CustomId(10729580169200549928);
    assert_eq!(Attribute::new_attribute("Test"), Attribute::new_attribute(String::from("Test")));
    assert_eq!(Attribute::new_attribute("Test"), Attribute::CustomId(10729580169200549928));
    assert_eq!(Attribute::CustomId(10729580169200549928), Attribute::new_attribute("Test"));
    assert_eq!(Attribute::CustomId(10729580169200549928), Attribute::CustomId(10729580169200549928));
    assert_eq!(name, name_hash);
    assert_eq!(name_hash, name);

    // test index
    assert_eq!(Attribute::IndexId(0), Attribute::IndexId(0));
    assert_ne!(Attribute::IndexId(0), Attribute::IndexId(1));
    assert_ne!(Attribute::IndexId(0), Attribute::new_index("Test"));
    let name = Attribute::new_index("Test");
    let name_hash = Attribute::IndexId(10729580169200549928);
    assert_eq!(Attribute::new_index("Test"), Attribute::new_index(String::from("Test")));
    assert_eq!(Attribute::new_index("Test"), Attribute::IndexId(10729580169200549928));
    assert_eq!(Attribute::IndexId(10729580169200549928), Attribute::new_index("Test"));
    assert_eq!(Attribute::IndexId(10729580169200549928), Attribute::IndexId(10729580169200549928));
    assert_eq!(name, name_hash);
    assert_eq!(name_hash, name);

    // test index not custom
    assert_ne!(Attribute::CustomId(0), Attribute::IndexId(0));
    assert_ne!(Attribute::IndexId(0), Attribute::CustomId(0));
    assert_ne!(Attribute::new_attribute("Test"), Attribute::new_index("Test"));
    assert_ne!(Attribute::new_index("Test"), Attribute::new_attribute("Test"));
    assert_ne!(Attribute::CustomId(0), Attribute::Default);
    assert_ne!(Attribute::IndexId(0), Attribute::Default);

    //test not working in example
    let test = Attribute::new_attribute("ZombieState");
    assert_eq!(Ok(Attribute::new_attribute("ZombieState")), ron::from_str("Custom(\"ZombieState\")"));
    assert_eq!(test.clone(), test);
    assert_eq!(test, test.clone());
    assert_eq!(Attribute::new_attribute("ZombieState"), Attribute::CustomId(4771896640381021065));
}

#[test]
fn test_hash() {
    use crate::utils::get_hasher;
    use std::hash::Hasher;
    // custom vs custom id
    let mut hash = get_hasher();
    Attribute::CustomId(0).hash(&mut hash);
    let hash_0 = hash.finish();
    let mut hash = get_hasher();
    Attribute::CustomId(0).hash(&mut hash);
    let hash_0_1 = hash.finish();
    let mut hash = get_hasher();
    Attribute::CustomId(1).hash(&mut hash);
    let hash_1 = hash.finish();
    let mut hash = get_hasher();
    Attribute::new_attribute("Test").hash(&mut hash);
    let hash_name = hash.finish();
    let mut hash = get_hasher();
    Attribute::CustomId(10729580169200549928).hash(&mut hash);
    let hash_name_1 = hash.finish();
    assert_ne!(hash_0, hash_1);
    assert_eq!(hash_0, hash_0_1);
    assert_eq!(hash_name, hash_name_1);


    // index vs index id
    let mut hash = get_hasher();
    Attribute::IndexId(0).hash(&mut hash);
    let hash_0 = hash.finish();
    let mut hash = get_hasher();
    Attribute::IndexId(0).hash(&mut hash);
    let hash_0_1 = hash.finish();
    let mut hash = get_hasher();
    Attribute::IndexId(1).hash(&mut hash);
    let hash_1 = hash.finish();
    let mut hash = get_hasher();
    Attribute::new_index("Test").hash(&mut hash);
    let hash_name = hash.finish();
    let mut hash = get_hasher();
    Attribute::IndexId(10729580169200549928).hash(&mut hash);
    let hash_name_1 = hash.finish();
    assert_ne!(hash_0, hash_1);
    assert_eq!(hash_0, hash_0_1);
    assert_eq!(hash_name, hash_name_1);

    // index vs custom
    let mut hash = get_hasher();
    Attribute::CustomId(0).hash(&mut hash);
    let hash_0 = hash.finish();
    let mut hash = get_hasher();
    Attribute::IndexId(0).hash(&mut hash);
    let hash_0_1 = hash.finish();
    let mut hash = get_hasher();
    Attribute::new_attribute("Test").hash(&mut hash);
    let hash_name = hash.finish();
    let mut hash = get_hasher();
    Attribute::CustomId(10729580169200549928).hash(&mut hash);
    let hash_name_1 = hash.finish();

    let mut hash = get_hasher();
    Attribute::new_index("Test").hash(&mut hash);
    let hash_index = hash.finish();
    let mut hash = get_hasher();
    Attribute::IndexId(10729580169200549928).hash(&mut hash);
    let hash_index_1 = hash.finish();
    assert_ne!(hash_0, hash_0_1);
    assert_ne!(hash_name, hash_index);
    assert_ne!(hash_index_1, hash_name_1);

    // Core vs custom
    let mut hash = get_hasher();
    Attribute::CustomId(0).hash(&mut hash);
    let hash_0 = hash.finish();
    let mut hash = get_hasher();
    Attribute::Default.hash(&mut hash);
    let hash_0_1 = hash.finish();
    assert_ne!(hash_0, hash_0_1);

    // core vs core
    let mut hash = get_hasher();
    Attribute::Default.hash(&mut hash);
    let hash_0 = hash.finish();
    let mut hash = get_hasher();
    Attribute::Default.hash(&mut hash);
    let hash_0_1 = hash.finish();
    assert_eq!(hash_0, hash_0_1);

    let mut hash = get_hasher();
    Attribute::FlipX.hash(&mut hash);
    let hash_0 = hash.finish();
    let mut hash = get_hasher();
    Attribute::Default.hash(&mut hash);
    let hash_0_1 = hash.finish();
    assert_ne!(hash_0, hash_0_1);
}