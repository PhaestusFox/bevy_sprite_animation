use std::collections::HashMap;

use bevy::reflect::Reflect;
use bevy::reflect::ReflectDeserialize;

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn custom_from_str() {
        let test = Attribute::new_attribute("Test".to_string());
        assert_eq!(test, Attribute::new_attribute("Test".to_string()));
        assert_eq!(test, Attribute::from_str("Test"));
        assert_eq!(test, Attribute::from_str("Custom(Test)"));
    }

    #[test]
    fn index_from_str() {
        let test = Attribute::new_index("Test".to_string());
        assert!(test.0 < 65536);
        assert_eq!(test, Attribute::new_index("Test".to_string()));
        assert_eq!(test, Attribute::from_str("Index(Test)"));
        assert_ne!(test, Attribute::from_str("Test"));
        assert_ne!(test, Attribute::new_attribute("Test".to_string()));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, bevy_inspector_egui::Inspectable, Clone, Copy, Reflect, PartialOrd, Ord)]
#[reflect_value(Serialize, Deserialize)]
pub struct Attribute(u64);

impl Attribute {
    pub const DELTA: Attribute = Attribute(0);
    pub const FRAMES: Attribute = Attribute(1);
    pub const TIME_ON_FRAME: Attribute = Attribute(2);
    pub const INDEX: Attribute = Attribute(256);
}

impl serde::Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let serde: AttributesSerde = self.into();
        serde.serialize(serializer)
    }
}

impl Into<AttributesSerde> for &Attribute {
    fn into(self) -> AttributesSerde {
        if self.0 < 256 {
            match self.0 {
            0 => AttributesSerde::Delta,
            1 => AttributesSerde::Frames,
            2 => AttributesSerde::FrameTime,
            _ => panic!("Reserved for futer use")
            }
        } else if self.0 < 65536 {
            match self.name() {
                Some(n) => {AttributesSerde::IndexName(n)},
                None => {AttributesSerde::IndexID(self.0 as u16)},
            }
        } else {
            match self.name() {
                Some(n) => {AttributesSerde::CustomName(n)},
                None => {AttributesSerde::CustomID(self.0)},
            }
        }
    }
}

impl Into<Attribute> for AttributesSerde {
    fn into(self) -> Attribute {
        match self {
            AttributesSerde::IndexID(id) => Attribute(id as u64),
            AttributesSerde::IndexName(name) => Attribute::new_index(name),
            AttributesSerde::Delta => Attribute::DELTA,
            AttributesSerde::FrameTime => Attribute::TIME_ON_FRAME,
            AttributesSerde::Frames => Attribute::FRAMES,
            AttributesSerde::CustomName(name) => Attribute::new_attribute(name),
            AttributesSerde::CustomID(r) => Attribute(r),
            _ => panic!()
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum AttributesSerde {
    Loop,
    IndexID(u16),
    IndexName(String),
    Delta,
    FrameTime,
    Frames,
    Next,
    CustomID(u64),
    CustomName(String),
}

impl<'de> serde::Deserialize<'de> for Attribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let serde: AttributesSerde = serde::Deserialize::deserialize(deserializer)?;
        Ok(serde.into())
    }
}

impl Default for Attribute {
    fn default() -> Attribute {
        Attribute(0)
    }
}

lazy_static::lazy_static! {
    static ref CUSTOMATTRIBUTES: std::sync::Mutex<HashMap<Attribute, String>> = {
        let mut map = HashMap::new();
        map.insert(Attribute::DELTA,       "Delta".to_string());
        map.insert(Attribute::TIME_ON_FRAME,   "FrameTime".to_string());
        map.insert(Attribute::FRAMES,      "Frames".to_string());
        std::sync::Mutex::new(map)
    };
}

impl Attribute {
    #[inline(always)]
    pub fn new_attribute(name: String) -> Attribute{
        let att = Attribute(Attribute::hash_for_custom(&name));
        CUSTOMATTRIBUTES.lock().unwrap().insert(att, name.to_string());
        att
    }

    #[inline(always)]
    pub fn new_index(name: String) -> Attribute{
        let att = Attribute(Attribute::hash_for_index(&name));
        CUSTOMATTRIBUTES.lock().unwrap().insert(att, name.to_string());
        att
    }

    fn hash_for_custom(name: &str) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let name = name.trim();
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        name.hash(&mut hasher);
        let mut res = hasher.finish();
        while res < 65536 {
            hasher.write_u8(0);
            res = hasher.finish();
        }
        res
    }

    fn hash_for_index(name: &str) -> u64{
        let name = name.trim();
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        name.hash(&mut hasher);
        let mut res = hasher.finish() as u16;
        while res < 256 {
            hasher.write_u8(0);
            res = hasher.finish() as u16;
        }
        res as u64
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
            Some(s) => {
                s.to_string()
            },
            None => {
                if self.0 < 256 {
                    panic!("All Core Attributes should have names")
                } else if self.0 < 65536 {
                    format!("Index({:#04X})", self.0)
                } else {
                    format!("Custom({:#016X})", self.0)
                }
            }
        }
    }


    /// Creates a string to an attribute primarly by hasinging it
    /// will check if the str is a known attribute such as DELTA
    /// will check if the str is Index(_) or Custom(_)
    /// regestoring there names or reading there hex value accordingly
    pub fn from_str(from: &str) -> Attribute {
        let from = from.trim();
        if from.starts_with("Index(0X") || from.starts_with("Custom(0X") {
            let start = from.find("(").unwrap() + 2;
            return Attribute(u64::from_str_radix(&from[start..from.len()-1], 16).expect("proper hex format"));
        }
        if from.starts_with("Index(") {
            return Attribute::new_index(from[6..from.len()-1].to_string());
        }
        if from.starts_with("Custom(") {
            return Attribute::new_attribute(from[7..from.len()-1].to_string());
        }
        match from {
            "Delta" => {Attribute::DELTA},
            "FrameTime" => {Attribute::TIME_ON_FRAME},
            "Frames" => {Attribute::FRAMES},
            _ => {Attribute(Attribute::hash_for_custom(&from))}
        }
    }
}