use std::collections::HashMap;

use bevy::reflect::Reflect;
use bevy::reflect::ReflectDeserialize;

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn custom_from_str() {
        let test = Attributes::new_attribute("Test".to_string());
        assert_eq!(test, Attributes::new_attribute("Test".to_string()));
        assert_eq!(test, Attributes::from_str("Test"));
        assert_eq!(test, Attributes::from_str("Custom(Test)"));
    }

    #[test]
    fn index_from_str() {
        let test = Attributes::new_index("Test".to_string());
        assert!(test.0 < 65536);
        assert_eq!(test, Attributes::new_index("Test".to_string()));
        assert_eq!(test, Attributes::from_str("Index(Test)"));
        assert_ne!(test, Attributes::from_str("Test"));
        assert_ne!(test, Attributes::new_attribute("Test".to_string()));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, bevy_inspector_egui::Inspectable, Clone, Copy, Reflect, PartialOrd, Ord)]
#[reflect_value(Serialize, Deserialize)]
pub struct Attributes(u64);

impl Attributes {
    pub const DELTA: Attributes = Attributes(0);
    pub const FRAMES: Attributes = Attributes(1);
    pub const TIME_ON_FRAME: Attributes = Attributes(2);
    pub const INDEX: Attributes = Attributes(256);
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

impl Into<Attributes> for AttributesSerde {
    fn into(self) -> Attributes {
        match self {
            AttributesSerde::IndexID(id) => Attributes(id as u64),
            AttributesSerde::IndexName(name) => Attributes::new_index(name),
            AttributesSerde::Delta => Attributes::DELTA,
            AttributesSerde::FrameTime => Attributes::TIME_ON_FRAME,
            AttributesSerde::Frames => Attributes::FRAMES,
            AttributesSerde::CustomName(name) => Attributes::new_attribute(name),
            AttributesSerde::CustomID(r) => Attributes(r),
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

impl<'de> serde::Deserialize<'de> for Attributes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let serde: AttributesSerde = serde::Deserialize::deserialize(deserializer)?;
        Ok(serde.into())
    }
}

impl Default for Attributes {
    fn default() -> Attributes {
        Attributes(0)
    }
}

lazy_static::lazy_static! {
    static ref CUSTOMATTRIBUTES: std::sync::Mutex<HashMap<Attributes, String>> = {
        let mut map = HashMap::new();
        map.insert(Attributes::DELTA,       "Delta".to_string());
        map.insert(Attributes::TIME_ON_FRAME,   "FrameTime".to_string());
        map.insert(Attributes::FRAMES,      "Frames".to_string());
        std::sync::Mutex::new(map)
    };
}

impl Attributes {
    #[inline(always)]
    pub fn new_attribute(name: String) -> Attributes{
        let att = Attributes(Attributes::hash_for_custom(&name));
        CUSTOMATTRIBUTES.lock().unwrap().insert(att, name.to_string());
        att
    }

    #[inline(always)]
    pub fn new_index(name: String) -> Attributes{
        let att = Attributes(Attributes::hash_for_index(&name));
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
    pub fn from_str(from: &str) -> Attributes {
        let from = from.trim();
        if from.starts_with("Index(0X") || from.starts_with("Custom(0X") {
            let start = from.find("(").unwrap() + 2;
            return Attributes(u64::from_str_radix(&from[start..from.len()-1], 16).expect("proper hex format"));
        }
        if from.starts_with("Index(") {
            return Attributes::new_index(from[6..from.len()-1].to_string());
        }
        if from.starts_with("Custom(") {
            return Attributes::new_attribute(from[7..from.len()-1].to_string());
        }
        match from {
            "Delta" => {Attributes::DELTA},
            "FrameTime" => {Attributes::TIME_ON_FRAME},
            "Frames" => {Attributes::FRAMES},
            _ => {Attributes(Attributes::hash_for_custom(&from))}
        }
    }
}