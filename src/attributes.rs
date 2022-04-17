use std::collections::HashMap;

use bevy::reflect::Reflect;
use bevy::reflect::ReflectDeserialize;

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn custom_from_str() {
        let test = Attribute::new_attribute("Test");
        assert_eq!(test, Attribute::new_attribute("Test"));
        assert_eq!(test, Attribute::from_str("Test"));
        assert_eq!(test, Attribute::from_str("Attribute(Test)"));
    }

    #[test]
    fn statics_work() {
        assert_eq!(Attribute::DELTA, Attribute::new_attribute("Delta"));
        assert_eq!(Attribute::DELTA, Attribute::from_str("Delta"));
        assert_eq!(Attribute::DELTA, Attribute::from_str("Core(Delta)"));
        assert_ne!(Attribute::DELTA, Attribute::new_attribute("Attribute(Delta)"));
        assert_eq!(Attribute::FRAMES, Attribute::new_attribute("Frames"));
        assert_eq!(Attribute::FRAMES, Attribute::from_str("Frames"));
        assert_eq!(Attribute::FRAMES, Attribute::from_str("Core(Frames)"));
        assert_ne!(Attribute::FRAMES, Attribute::new_attribute("Attribute(Frames)"));
        assert_ne!(Attribute::TIME_ON_FRAME, Attribute::new_attribute("FrameTime"));
        assert_eq!(Attribute::TIME_ON_FRAME, Attribute::from_str("FrameTime"));
        assert_eq!(Attribute::TIME_ON_FRAME, Attribute::from_str("Core(FrameTime)"));
        assert_ne!(Attribute::TIME_ON_FRAME, Attribute::from_str("Attribute(FrameTime)"));
    }

    #[test]
    fn index_from_str() {
        let test = Attribute::new_index("Test");
        assert!(test.0 < 65536);
        assert_eq!(test, Attribute::new_index("Test"));
        assert_eq!(test, Attribute::from_str("Index(Test)"));
        assert_ne!(test, Attribute::from_str("Test"));
        assert_ne!(test, Attribute::new_attribute("Test"));
    }
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, PartialOrd, Ord)]
#[reflect_value(Serialize, Deserialize)]
pub struct Attribute(u64);

#[cfg(feature = "bevy-inspector-egui")]
impl bevy_inspector_egui::Inspectable for Attribute {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _options: Self::Attributes, _context: &mut bevy_inspector_egui::Context) -> bool {
        let mut edit = false;
        let mut name = self.name_or_id();
        if ui.text_edit_singleline(&mut name).changed() {
            if name.starts_with("Index(") || name.starts_with("Attribute(") {
                if name.ends_with(")") {
                    name.push(')');
                }
            }
            *self = Attribute::from_str(&name);
            edit = true;
        }
        edit
    }
}

impl Attribute {
    pub const NULL: Attribute = Attribute(0);
    pub const DELTA: Attribute = Attribute(1);
    pub const FRAMES: Attribute = Attribute(2);
    pub const TIME_ON_FRAME: Attribute = Attribute(3);
    pub const FLIP_X: Attribute = Attribute(4);
    pub const FLIP_Y: Attribute = Attribute(5);
    pub const INDEX: Attribute = Attribute(256);
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name_or_id())
    }
}

impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name_or_id())
    }
}

impl serde::Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let serde: AttributeSerde = self.into();
        serde.serialize(serializer)
    }
}

impl Into<AttributeSerde> for &Attribute {
    fn into(self) -> AttributeSerde {
        if self.0 < 256 {
            match self.0 {
            1 => AttributeSerde::Delta,
            2 => AttributeSerde::Frames,
            3 => AttributeSerde::FrameTime,
            4 => AttributeSerde::FlipX,
            5 => AttributeSerde::FlipY,
            _ => panic!("Reserved for futer use")
            }
        } else if self.0 < 65536 {
            match self.name() {
                Some(n) => {AttributeSerde::IndexName(n)},
                None => {AttributeSerde::IndexID(self.0 as u16)},
            }
        } else {
            match self.name() {
                Some(n) => {AttributeSerde::AttributeName(n)},
                None => {AttributeSerde::AttributeID(self.0)},
            }
        }
    }
}

impl Into<Attribute> for AttributeSerde {
    fn into(self) -> Attribute {
        match self {
            AttributeSerde::IndexID(id) => Attribute(id as u64),
            AttributeSerde::IndexName(name) => Attribute::new_index(&name),
            AttributeSerde::Delta => Attribute::DELTA,
            AttributeSerde::FrameTime => Attribute::TIME_ON_FRAME,
            AttributeSerde::Frames => Attribute::FRAMES,
            AttributeSerde::FlipX => Attribute::FLIP_X,
            AttributeSerde::FlipY => Attribute::FLIP_Y,
            AttributeSerde::AttributeName(name) => Attribute::new_attribute(&name),
            AttributeSerde::AttributeID(r) => Attribute(r),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum AttributeSerde {
    IndexID(u16),
    IndexName(String),
    Delta,
    FrameTime,
    Frames,
    FlipX,
    FlipY,
    AttributeID(u64),
    AttributeName(String),
}

impl<'de> serde::Deserialize<'de> for Attribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let serde: AttributeSerde = serde::Deserialize::deserialize(deserializer)?;
        Ok(serde.into())
    }
}

lazy_static::lazy_static! {
    static ref CUSTOMATTRIBUTES: std::sync::Mutex<HashMap<Attribute, String>> = {
        let mut map = HashMap::new();
        map.insert(Attribute::NULL,             "null".to_string());
        map.insert(Attribute::DELTA,            "Delta".to_string());
        map.insert(Attribute::TIME_ON_FRAME,    "FrameTime".to_string());
        map.insert(Attribute::FRAMES,           "Frames".to_string());
        map.insert(Attribute::FLIP_X,           "FlipX".to_string());
        map.insert(Attribute::FLIP_Y,           "FlipY".to_string());
        std::sync::Mutex::new(map)
    };
}

impl Attribute {

    /// Returns the attribute for the given Index name
    /// adds the name to the list of known names if it is not already there
    /// use from_str to get the attribute from a string
    #[inline(always)]
    pub fn new_index(name: &str) -> Attribute{
        let mut map = CUSTOMATTRIBUTES.lock().unwrap();
        let att = Attribute(Attribute::hash_for_index(name));
        if !map.contains_key(&att) {
            map.insert(att, name.to_string());
        }
        att
    }

    /// Returns the attribute for the given name
    /// adds the name to the list of known names if it is not already there
    /// use from_str to get the attribute from a string

    #[inline(always)]
    pub fn new_attribute(name: &str) -> Attribute{
        let mut map = CUSTOMATTRIBUTES.lock().unwrap();
        let att = Attribute(Attribute::hash_for_custom(name));
        if !map.contains_key(&att) {
            map.insert(att, name.to_string());
        }
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

    /// Returns the name of the attribute or None if a custom attribute that is not in the list
    pub fn name(&self) -> Option<String> {
        if let Some(v) = CUSTOMATTRIBUTES.lock().unwrap().get(self) {
            Some(v.clone())
        } else {
            None
        }
    }

    /// Returns the name of the attribute or its inner u64 hex if it is a custom attribute that is not in the list
    pub fn name_or_id(&self) -> String {
        let mut res = String::new();
        if self.0 < 256 {
            res.push_str("Core(");
        } else if self.0 < 65536 {
            res.push_str("Index(");
        } else {
            res.push_str("Attribute(");
        };
        match self.name() {
            Some(s) => {
                res.push_str(&s);
                res.push(')');
            },
            None => {
                if self.0 < 256 {
                    panic!("All Core Attribute should have names")
                } else if self.0 < 65536 {
                    res.push_str(&format!("{:#06X}", self.0));
                    res.push(')');
                } else {
                    res.push_str(&format!("Attribute({:#018X})", self.0));
                    res.push(')');
                }
            }
        }
        res
    }


    /// Creates an attribute from a str
    /// if the string has the form "Index(0x1234)" or "Attribute(0x12345678)"
    /// the attribute will be the corresponding attribute
    /// if the string has the form "Index('name')" or "Attribute('name')"
    /// the attribute name will be added to the list of known names
    /// will check if the str is a known attribute such as DELTA, TIME_ON_FRAME, FRAMES
    pub fn from_str(from: &str) -> Attribute {
        let from = from.trim();
        if from.starts_with("Index(") {
            if from.len() > 7 {
            if let Some(c) = from.get(6..from.len() - 1) {
                return if c.starts_with(|x: char| {x.is_digit(10)}) {
                    Attribute::from_digit(c)
                }
                else {
                    Attribute::new_index(c)
                }
            }
            }
            panic!("Invalid Index(...)")
        }
        if from.starts_with("Attribute(") {
            if from.len() > 11 {
            if let Some(c) = from.get(10..from.len() - 1) {
                return if c.starts_with(|x: char| {x.is_digit(10)}) {
                    Attribute::from_digit(&from[10..from.len() - 1])
                }
                else {
                    Attribute::new_attribute(&from[10..from.len() - 1])
                }
            }
            }
            panic!("Invalid Attribute(...)")
        }
        if from.starts_with("Core(") {
            return match &from[5..from.len() - 1] {
                "Delta" => {Attribute::DELTA},
                "FrameTime" => {Attribute::TIME_ON_FRAME},
                "Frames" => {Attribute::FRAMES},
                _ => panic!("Invalid Core(...)")
            }
        }
        if from.starts_with(|c: char| {c.is_digit(10)}) {
            return Attribute::from_digit(from);
        }
        match from {
            "Delta" => {Attribute::DELTA},
            "FrameTime" => {Attribute::TIME_ON_FRAME},
            "Frames" => {Attribute::FRAMES},
            _ => {Attribute::new_attribute(from)}
        }
    }

    fn from_digit(data: &str) -> Attribute {
        let data = data.trim();
        if data.starts_with("0x") || data.starts_with("0X") {
            return Attribute(u64::from_str_radix(&data[2..], 16).expect("proper hex format"));
        }
        if data.starts_with("0b") || data.starts_with("0B") {
            return Attribute(u64::from_str_radix(&data[2..], 2).expect("proper binary format"));
        }
        if data.starts_with("0o") || data.starts_with("0O") {
            return Attribute(u64::from_str_radix(&data[2..], 8).expect("proper octal format"));
        }
        Attribute(u64::from_str_radix(data, 10).expect("proper decimal format"))
    }
}