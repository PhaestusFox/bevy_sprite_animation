use std::collections::HashMap;

use crate::prelude::*;
use crate::error::BevySpriteAnimationError as Error;

pub trait MatchType:'static + Send + Sync + Eq + std::hash::Hash {}

impl<T> MatchType for T
where T:'static + Send + Sync + Eq + std::hash::Hash
{
    
}

pub struct MatchNode<T:'static + Send + Sync> {
    name: String,
    pairs: HashMap<T, NodeID>,
    check: Attributes,
    default: NodeID,
}

impl<T:MatchType + Ord> std::hash::Hash  for MatchNode<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        let mut pairs: Vec<(&T, &NodeID)> = self.pairs.iter().collect();
        pairs.sort_by(|a,b| { a.0.partial_cmp(b.0).unwrap()});
        for pair in pairs.iter() {
            pair.hash(state);
        }
        self.check.hash(state);
        self.default.hash(state);
    }
}

impl<T:MatchType> MatchNode<T> {
    pub fn new(name: &str, set: Vec<(T, NodeID)>, check: Attributes, default: NodeID) -> MatchNode<T> {
        let mut pairs = HashMap::default();
        for (k,v) in set.into_iter() {
            pairs.insert(k, v);
        }
        
        MatchNode {
            name: name.to_string(),
            pairs,
            check,
            default
        }
    }

}

#[cfg(feature = "serialize")]
impl<T: MatchType + serde::Serialize + serde::de::DeserializeOwned + Ord> MatchNode<T> {
    pub fn loader() -> Box<dyn NodeLoader> {
        Box::new(MatchNodeLoader::<T>::default())
    }
}

impl<T> AnimationNode for MatchNode<T>
where T:MatchType + serde::de::DeserializeOwned + serde::Serialize + std::any::Any + Ord
{
    fn run(&self, state: &mut crate::state::AnimationState) -> NodeResult {

        let val = match state.try_get_attribute_or_error::<T>(&self.check) {
            Ok(x) => x,
            Err(e) => return NodeResult::Error(e),
        };

        if let Some(next) = self.pairs.get(&val) {
            NodeResult::Next(*next)
        } else {
            NodeResult::Next(self.default)
        }
    }

    fn node_type(&self) -> String {
        format!("MatchNode<{}>", std::any::type_name::<T>())
    }

    fn name(&self) -> &str {
        &self.name
    }

    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _context: &mut bevy_inspector_egui::Context) -> bool {
        ui.label(format!("MatchNode<{}>({})",std::any::type_name::<T>(), self.name));
        return false;
    }

    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, _asset_server: &bevy::prelude::AssetServer) -> Result<(), Error> {
        let pretty = ron::ser::PrettyConfig::new();
        data.push_str("MatchNode<");
        data.push_str(std::any::type_name::<T>());
        data.push_str(">(\n\t");
        data.push_str("name: \"");
        data.push_str(self.name());
        data.push_str("\",\n\tcheck: ");
        data.push_str(&ron::ser::to_string_pretty(&self.check, pretty.clone())?);
        data.push_str(",\n\tdefault: ");
        data.push_str(&ron::ser::to_string_pretty(&self.default, pretty.clone())?);
        data.push_str(",\n\tpairs: [\n\t");
        for pair in self.pairs.iter() {
            data.push_str(&ron::ser::to_string_pretty(&pair, pretty.clone())?);
            data.push_str(",\n\t");
        }
        data.push_str("],\n),");
        Ok(())
    }

    #[cfg(feature = "hash")]
    fn hash(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        Hash::hash(self,&mut hasher);
        hasher.finish()
    }
}

#[cfg(feature = "serialize")]
pub use loader::MatchNodeLoader;

#[cfg(feature = "serialize")]
mod loader {
    #[test]
    fn match_node_name_offset() {
        let mnl = MatchNodeLoader::<u32>::default();
        assert_eq!("MatchNode<u32>", mnl.can_load()[0]);
    }

    use core::marker::PhantomData;

    use crate::node_core::NodeLoader;

    use crate::error::BevySpriteAnimationError as Error;
    use crate::prelude::{NodeID, Attributes};

    use super::{MatchNode, MatchType};

    pub struct MatchNodeLoader<T>{
        can_load: Vec<&'static str>, 
        marker: PhantomData<T>,
    }

    impl<T:MatchType> Default for MatchNodeLoader<T>{
        fn default() -> Self {
            MatchNodeLoader {
                can_load: vec![&std::any::type_name::<MatchNode<T>>()[42..]],
                marker: PhantomData::default()
            }
        }
    }
    //012345678901234567890123456789012345678901
    //bevy_sprite_animation::nodes::match_node::
    impl<T> NodeLoader for MatchNodeLoader<T> where T:MatchType + std::any::Any + serde::de::DeserializeOwned + serde::Serialize + Ord {
        fn load(&mut self, data: &str, _asset_server: &bevy::prelude::AssetServer) -> Result<Box<dyn crate::prelude::AnimationNode>, crate::error::BevySpriteAnimationError> {
        use std::collections::HashMap;
        let data = data.trim();
        let data = if data.starts_with(&format!("{}(", self.can_load[0])) {&data[self.can_load[0].len()..].trim()} else {data};
        let mut chars = data.chars().peekable();
        let mut map: HashMap<&str, &str> = HashMap::new();
        let mut start = 0;
        let mut len = 0;
        let mut key = "";
        let mut is_key = true;
        if let Some(c) = chars.peek() {if *c == '(' {start += 1; chars.next();}}
        while let Some(c) = chars.next() {
            match c {
                ':' => {if is_key {
                    key = &data[start..start+len].trim();
                    start += len + 1;
                    len = 0;
                    is_key = false;
                    }
                },
                ',' => if !is_key {
                    //bevy::log::info!("add {} : {}", key, data[start..start+len].trim());
                    map.insert(key, data[start..start+len].trim());
                    start += len + 1;
                    len = 0;
                    is_key = true;
                    key = "";
                }
                '[' => {
                    while let Some(c) = chars.next() {
                        len += 1;
                        if c == ']' {
                            len += 1;
                            break;
                        }
                    }
                }
                _ => {
                    len += 1;
                }
            }
        }
        if len > 0 {
            map.insert(key, data[start..start+len].trim());
        }
        //bevy::prelude::info!("{:?}", map);
        let check = map.get("check").ok_or(Error::DeserializeError { node_type: "MatchNode", message: format!("Failed to find check: Attribute"), loc: crate::here!() })?;
        let pairs = map.get("pairs").ok_or(Error::DeserializeError { node_type: "MatchNode", message: format!("Failed to find pairs: [({},NodeID)]", std::any::type_name::<T>()), loc: crate::here!() })?;
        let default = map.get("default").ok_or(Error::DeserializeError { node_type: "MatchNode", message: format!("Failed to find default: {}]", std::any::type_name::<T>()), loc: crate::here!() })?;
        let name = map.get("name").ok_or(Error::DeserializeError { node_type: "MatchNode", message: format!("Failed to find name: String]"), loc: crate::here!() })?;
        let check: Attributes = ron::from_str(check)?;
        let pairs: Vec<(T, NodeID)> = ron::from_str(pairs).or_else(|_| {Err(Error::DeserializeError { node_type: "MatchNode", message: format!("Failed to deserialize pairs, check type matches"), loc: crate::here!() })})?;
        let default: NodeID = ron::from_str(default)?;
        let name = name[1..name.len()-1].to_string();
        let pairs: HashMap<T, NodeID> = pairs.into_iter().collect();
        Ok(Box::new(MatchNode {
            name,
            pairs,
            default,
            check
        }))
    }

    fn can_load(&self) -> &[&str] {
        debug_assert!(self.can_load[0].starts_with("MatchNode<"));
        &self.can_load
    }
    }
}