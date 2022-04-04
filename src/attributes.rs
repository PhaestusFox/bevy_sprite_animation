use std::collections::HashMap;

use bevy::reflect::Reflect;
use bevy::reflect::ReflectDeserialize;

#[derive(Debug, Hash, PartialEq, Eq, bevy_inspector_egui::Inspectable, Clone, Copy, Reflect, PartialOrd, Ord)]
#[reflect_value(Serialize, Deserialize)]
pub enum Attributes {
    Loop,
    Index,
    Delta,
    FrameTime,
    Frames,
    Next,
    Custom(u64),
}

impl serde::Serialize for Attributes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let serde: AttributesSerde = self.into();
        serde.serialize(serializer)
    }
}

impl Into<AttributesSerde> for &Attributes {
    fn into(self) -> AttributesSerde {
        match self {
            Attributes::Loop => AttributesSerde::Loop,
            Attributes::Index => AttributesSerde::Index,
            Attributes::Delta => AttributesSerde::Delta,
            Attributes::FrameTime => AttributesSerde::FrameTime,
            Attributes::Frames => AttributesSerde::Frames,
            Attributes::Next => AttributesSerde::Next,
            Attributes::Custom(r) => match self.name(){
                Some(n) =>    {AttributesSerde::CustomName(n.to_string())},
                None =>             {AttributesSerde::CustomID(*r)}
            },
        }
    }
}

impl Into<Attributes> for AttributesSerde {
    fn into(self) -> Attributes {
        match self {
            AttributesSerde::Loop => Attributes::Loop,
            AttributesSerde::Index => Attributes::Index,
            AttributesSerde::Delta => Attributes::Delta,
            AttributesSerde::FrameTime => Attributes::FrameTime,
            AttributesSerde::Frames => Attributes::Frames,
            AttributesSerde::Next => Attributes::Next,
            AttributesSerde::CustomName(n) => Attributes::new_custom(&n),
            AttributesSerde::CustomID(r) => Attributes::Custom(r),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum AttributesSerde {
    Loop,
    Index,
    Delta,
    FrameTime,
    Frames,
    Next,
    CustomID(u64),
    CustomName(String),
}

impl<'de> serde::Deserialize<'de> for Attributes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let serde: AttributesSerde = serde::Deserialize::deserialize(deserializer)?;
        Ok(serde.into())
    }
}

struct AttributeVisitor;

impl<'de> serde::de::Visitor<'de> for AttributeVisitor {
    type Value = Attributes;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Expected str reprentaion of a Attribute")
    }

    fn visit_str<E>(self, str: &str) -> Result<Self::Value, E>
    where E: serde::de::Error
    {
        match str {
            "Loop" => {Ok(Attributes::Loop)}
            "Index" => {Ok(Attributes::Index)}
            "Delta" => {Ok(Attributes::Delta)}
            "FrameTime" => {Ok(Attributes::FrameTime)}
            "Frames" => {Ok(Attributes::Frames)}
            "Next" => {Ok(Attributes::Next)}
            _ => {
                if str.starts_with("Custom(") {Err(E::unknown_field(str, &["Custom()"]))}
                else {
                    let short = &str[7..str.len()-1];
                    if short.starts_with("0x") {
                        Ok(Attributes::Custom(u64::from_str_radix(short, 16).unwrap()))
                    } else {
                        use std::hash::Hash;
                        use std::hash::Hasher;
                        let mut hasher = std::collections::hash_map::DefaultHasher::default();
                        short.hash(&mut hasher);
                        Ok(Attributes::Custom(hasher.finish()))
                    }
                }
            }
        }
    }
}

impl Default for Attributes {
    fn default() -> Attributes {
        Attributes::Index
    }
}

lazy_static::lazy_static! {
    static ref CUSTOMATTRIBUTES: std::sync::Mutex<HashMap<Attributes, String>> = {
        let mut map = HashMap::new();
        map.insert(Attributes::Loop,        "Loop".to_string());
        map.insert(Attributes::Index,       "Index".to_string());
        map.insert(Attributes::Delta,       "Delta".to_string());
        map.insert(Attributes::FrameTime,   "FrameTime".to_string());
        map.insert(Attributes::Frames,      "Frames".to_string());
        map.insert(Attributes::Next,        "Next".to_string());
        std::sync::Mutex::new(map)
    };
}

impl Attributes {
    pub fn new_custom(name: &str) -> Attributes{
        let att = Attributes::from_str(name);
        if let Attributes::Custom(_) = att {
            CUSTOMATTRIBUTES.lock().unwrap().insert(att, name.to_string());
        };
        att
    }

    fn hash_to_custom(name: &str) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        name.hash(&mut hasher);
        hasher.finish()
    }

    pub fn name(&self) -> Option<String> {
        if let Some(v) = CUSTOMATTRIBUTES.lock().unwrap().get(self) {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn name_or_id(&self) -> String {
        match self.name() {
            Some(s) => {s.to_string()},
            None => {
                if let Attributes::Custom(id) = self {
                    format!("{:#x}", id)
                } else {panic!("Core Attributes are all named")}
            }
        }
    }

    pub fn from_str(from: &str) -> Attributes {
        if from.starts_with("Index(") {
            todo!("add custom index names")
        }
        match from {
            "Loop" => {Attributes::Loop},
            "Index" => {Attributes::Index},
            "Delta" => {Attributes::Delta},
            "FrameTime" => {Attributes::FrameTime},
            "Frames" => {Attributes::Frames},
            "Next" => {Attributes::Next},
            _ => {Attributes::Custom(Attributes::hash_to_custom(from))}
        }
    }
}