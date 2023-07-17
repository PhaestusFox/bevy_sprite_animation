use std::collections::HashMap;
use std::fmt::Debug;

use crate::prelude::*;
use crate::error::{BevySpriteAnimationError as Error, LoadError};
use crate::serde::{LoadNode, ReflectLoadNode};

#[cfg(not(feature = "serialize"))]
pub trait MatchType:'static + Send + Sync + Eq + std::hash::Hash + Debug {}
#[cfg(not(feature = "serialize"))]
impl<T> MatchType for T
where T:'static + Send + Sync + Eq + std::hash::Hash + Debug
{
    
}

#[cfg(feature = "serialize")]
pub trait MatchType:'static + Send + Sync + Eq + serde::de::DeserializeOwned + serde::Serialize + Debug + std::hash::Hash{}
#[cfg(feature = "serialize")]
impl<T> MatchType for T
where T:'static + Send + Sync + Eq + serde::de::DeserializeOwned + serde::Serialize + Debug + std::hash::Hash {}

#[derive(Reflect, Debug)]
#[reflect(LoadNode)]
pub struct MatchNode<T: MatchType> {
    name: String,
    #[reflect(ignore)]
    pairs: HashMap<T, NodeId<'static>>,
    check: Attribute,
    default: NodeId<'static>,
}

impl<T:MatchType> MatchNode<T> {
    pub fn new(name: &str, set: Vec<(T, NodeId<'static>)>, check: Attribute, default: NodeId<'static>) -> MatchNode<T> {
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

impl<T> AnimationNodeTrait for MatchNode<T>
where T:MatchType + serde::de::DeserializeOwned + serde::Serialize + std::any::Any
{
    fn run(&self, state: &mut crate::state::AnimationState) -> Result<NodeResult, RunError> {

        let val = match state.get_attribute::<T>(&self.check) {
            Ok(x) => x,
            Err(e) => return Err(RunError::Custom(format!("Match: {}: {:?}", e, self.check))),
        };

        Ok(if let Some(next) = self.pairs.get(&val) {
            NodeResult::Next(next.clone())
        } else {
            NodeResult::Next(self.default.clone())
        })
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

    fn id(&self) -> NodeId {
        NodeId::from_name(&self.name)
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, _: &bevy::prelude::AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name));
        for (index, next) in self.pairs.iter() {
            this.dot(out);
            out.push_str(" -> ");
            next.dot(out);
            out.push_str(&format!("[label=\"{:?}\"];\n", index));
        }
    }
}

use bevy::reflect::Reflect;
use ron::error::SpannedError;

#[cfg(feature = "serialize")]
impl<T: MatchType + serde::de::DeserializeOwned> LoadNode for MatchNode<T> {
    fn load<'b>(s: &str, _: &mut bevy::asset::LoadContext<'b>, _dependencies: &mut Vec<bevy::asset::AssetPath<'static>>) -> Result<AnimationNode, crate::error::LoadError> {
        let data = s.trim();
        let data = &data[1..data.len()-1];
        let mut chars = crate::serde::InputIter::new(data);
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
        let check = map.get("check").ok_or(LoadError::Ron(SpannedError{code: ron::Error::MissingStructField { field: "Check", outer: Some(std::any::type_name::<Self>().to_string()) }, position: chars.file_position()}))?;
        let pairs = map.get("pairs").ok_or(LoadError::Ron(SpannedError{code: ron::Error::MissingStructField { field: "Pairs", outer: Some(std::any::type_name::<Self>().to_string()) }, position: chars.file_position()}))?;
        let default = map.get("default").ok_or(LoadError::Ron(SpannedError{code: ron::Error::MissingStructField { field: "Default", outer: Some(std::any::type_name::<Self>().to_string()) }, position: chars.file_position()}))?;
        let name = map.get("name").ok_or(LoadError::Ron(SpannedError{code: ron::Error::MissingStructField { field: "Name", outer: Some(std::any::type_name::<Self>().to_string()) }, position: chars.file_position()}))?;
        let check: Attribute = ron::from_str(check).or_else(|e| Err(LoadError::Ron(e).add_offset(chars.file_position())))?;
        let pairs: Vec<(T, NodeId)> = ron::from_str(pairs).or_else(|e| Err(LoadError::Ron(e).add_offset(chars.file_position())))?;
        let default: NodeId = ron::from_str(default).or_else(|e| Err(LoadError::Ron(e).add_offset(chars.file_position())))?;
        let name = name[1..name.len()-1].to_string();
        let pairs: HashMap<T, NodeId> = pairs.into_iter().collect();
        Ok(AnimationNode::new(MatchNode {
            name,
            pairs,
            default,
            check
        }))
    }
}