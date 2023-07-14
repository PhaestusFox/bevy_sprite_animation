use std::borrow::Cow;
use std::hash::Hasher;

use bevy::reflect::Reflect;
use bevy::reflect::ReflectDeserialize;
use bevy::reflect::ReflectSerialize;

use crate::prelude::get_hasher;

#[derive(Default, Hash, PartialEq, Eq, Reflect, PartialOrd, Ord, serde::Serialize, serde::Deserialize, strum_macros::AsRefStr)]
#[reflect_value(Serialize, Deserialize)]
pub enum Attribute {
    Custom(Cow<'static, str>),
    CustomId(u64),
    #[default]
    Default,
    Delta,
    Frames,
    TimeThisFrame,
    FlipX,
    FlipY,
    LastFPS,
    Index(Cow<'static, str>),
    IndexId(u64),
}

impl Clone for Attribute {
    fn clone(&self) -> Self {
        match self {
            Self::Custom(arg0) => Self::new_attribute_id(arg0),
            Self::CustomId(arg0) => Self::CustomId(arg0.clone()),
            Self::Default => Self::Default,
            Self::Delta => Self::Delta,
            Self::Frames => Self::Frames,
            Self::TimeThisFrame => Self::TimeThisFrame,
            Self::FlipX => Self::FlipX,
            Self::FlipY => Self::FlipY,
            Self::LastFPS => Self::LastFPS,
            Self::Index(arg0) => Self::new_index_id(arg0),
            Self::IndexId(arg0) => Self::IndexId(arg0.clone()),
        }
    }
}

impl Attribute {
    #[inline(always)]
    pub fn is_core(&self) -> bool {
        match self {
            Attribute::Custom(_) |
            Attribute::CustomId(_) |
            Attribute::Index(_) |
            Attribute::IndexId(_) => false,
            _ => true
        }
    }
    #[inline(always)]
    pub fn is_index(&self) -> bool {
        match self {
            Attribute::Index(_) | Attribute::IndexId(_) => true,
            _ => false
        }
    }
    #[inline(always)]
    pub fn is_custom(&self) -> bool {
        match self {
            Attribute::Custom(_) | Attribute::CustomId(_) => true,
            _ => false
        }
    }

    fn deep_clone(&self) -> Self {
        match self {
            Self::Custom(arg0) => Self::Custom(arg0.clone()),
            Self::CustomId(arg0) => Self::CustomId(arg0.clone()),
            Self::Default => Self::Default,
            Self::Delta => Self::Delta,
            Self::Frames => Self::Frames,
            Self::TimeThisFrame => Self::TimeThisFrame,
            Self::FlipX => Self::FlipX,
            Self::FlipY => Self::FlipY,
            Self::LastFPS => Self::LastFPS,
            Self::Index(arg0) => Self::Index(arg0.clone()),
            Self::IndexId(arg0) => Self::IndexId(arg0.clone()),
        }
    }

    /// turns a &str into a Attribute by attemting to deserialize it with ron
    /// if this failes it will just return Attribute::Custom(s.to_string().into())
    pub fn from_str(s: &str) -> Attribute {
        match ron::from_str::<Attribute>(s) {
            Ok(ok) => ok,
            Err(_) => Attribute::Custom(s.to_string().into()),
        }
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        match self {
            Attribute::Custom(name) |
            Attribute::Index(name) => {
                f.write_str(name.as_ref())
            }
            Attribute::CustomId(_) |
            Attribute::IndexId(_) => {
                f.write_str("(")?;
                f.write_str(self.name_or_id().as_ref())?;
                f.write_str(")")
            },
            _ => f.write_str(self.as_ref())
        }
    }
}

impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        match self {
            Attribute::Custom(_) |
            Attribute::CustomId(_) |
            Attribute::Index(_) |
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
        Attribute::Index(name.into())
    }

    /// Returns the Attribute::Custom() for the given name
    /// The Attribute will have a name
    /// If you .deep_clone it the name will stay
    /// /// Rust doesn't let you have diffrent behaviure for clone and copy :'[ or even imle your own copy :(
    /// If you Clone it the name will be hashed to save allocateing stings all the time
    #[inline(always)]
    pub fn new_attribute(name: impl Into<Cow<'static, str>>) -> Attribute{
        Attribute::Custom(name.into())
    }

    /// Returns the Attribute::CustomId() for the given name
    /// The Attribute will **not** have a name
    pub fn new_attribute_id(name: impl std::hash::Hash) -> Attribute {
        let mut hasher = get_hasher();
        name.hash(&mut hasher);
        Attribute::CustomId(hasher.finish())
    }

    /// Returns the Attribute::IndexId() for the given name
    /// The Attribute will **not** have a name
    pub fn new_index_id(name: impl std::hash::Hash) -> Attribute {
        let mut hasher = get_hasher();
        name.hash(&mut hasher);
        Attribute::IndexId(hasher.finish())
    }
    
    /// Returns the name of the Attribute or Index will return None if the name has been errased
    pub fn name(&self) -> Option<&str> {
        match self {
            Attribute::Custom(name) => Some(name.as_ref()),
            Attribute::CustomId(_) => None,
            Attribute::Index(name) => Some(name.as_ref()),
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

}