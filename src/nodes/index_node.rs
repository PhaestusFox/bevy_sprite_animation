use crate::node_core::CanLoad;
use crate::prelude::*;
use bevy::prelude::Handle;
use bevy::prelude::Image;
use bevy::reflect::Reflect;
use crate::error::BevySpriteAnimationError as Error;

#[cfg(test)]
mod test {
    use crate::test::test_asset_server;

    #[test]
    #[cfg(feature = "serialize")]
    fn deserialize_clean_str() {
        use super::IndexNode;
        use crate::node_core::AnimationNodeTrait;
        use crate::node_core::NodeLoader;
        use super::IndexNodeLoader;
        let asset_server = test_asset_server();
        let mut handles = Vec::new();
        for i in 0..3 {
            handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
        }
        let mut loader = IndexNodeLoader;
        let test_node = loader.load("name: \"Zombie1_Idle\",
        frames: [
        Zombie1/zombie1_00000.png,
        Zombie1/zombie1_00001.png,
        Zombie1/zombie1_00002.png,
        ]", &asset_server).unwrap();
    let true_node = Box::new(IndexNode::new("Zombie1_Idle", &handles[..3], true));
    assert_eq!(test_node.hash(), true_node.hash());
    }

    #[test]
    #[cfg(feature = "serialize")]
    fn deserialize_capped_str() {
        use super::IndexNode;
        use crate::node_core::AnimationNodeTrait;
        use crate::node_core::NodeLoader;
        use super::IndexNodeLoader;
        let asset_server = test_asset_server();
        let mut handles = Vec::new();
        for i in 0..3 {
            handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
        }
        let mut loader = IndexNodeLoader;
        let test_node = loader.load("
            (
            name: \"Zombie1_Idle\",
            frames: [
            Zombie1/zombie1_00000.png,
            Zombie1/zombie1_00001.png,
            Zombie1/zombie1_00002.png,
            ],
            ),
        ", &asset_server).unwrap();
        let true_node: Box<dyn AnimationNodeTrait> = Box::new(IndexNode::new("Zombie1_Idle", &handles[..3], true));
        assert_eq!(test_node.hash(), true_node.hash());
    }

    #[test]
    #[cfg(feature = "serialize")]
    fn deserialize_full_str() {
        use super::IndexNode;
        use crate::node_core::AnimationNodeTrait;
        use crate::node_core::NodeLoader;
        use super::IndexNodeLoader;
        let asset_server = test_asset_server();
        let mut handles = Vec::new();
        for i in 0..3 {
            handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
        }
        let mut loader = IndexNodeLoader;
        let test_node = loader.load("
            IndexNode(
            name: \"Zombie1_Idle\",
            frames: [
            Zombie1/zombie1_00000.png,
            Zombie1/zombie1_00001.png,
            Zombie1/zombie1_00002.png,
            ],
            is_loop: true,
            ),
        ", &asset_server).unwrap();
        let true_node: Box<dyn AnimationNodeTrait> = Box::new(IndexNode::new("Zombie1_Idle", &handles[..3], true));
        assert_eq!(test_node.hash(), true_node.hash());
    }

    #[test]
    #[cfg(feature = "serialize")]
    fn serialize_str_pretty() {
        use super::IndexNode;
        use crate::node_core::AnimationNodeTrait;
        let asset_server = test_asset_server();
        let mut handles = Vec::new();
        for i in 0..3 {
            handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
        }
        let true_node: Box<dyn AnimationNodeTrait> = Box::new(IndexNode::new("Zombie1_Idle", &handles[..3], true));
        let mut res = String::new();
        let ser_res = true_node.serialize(&mut res, &asset_server);
        assert!(ser_res.is_ok(), "{}", ser_res.err().unwrap());
        assert!(res == "IndexNode(\n\tname: \"Zombie1_Idle\",\n\tframes: [\n\tZombie1/zombie1_00000.png,\n\tZombie1/zombie1_00001.png,\n\tZombie1/zombie1_00002.png,\n\t],\n\tis_loop: true,\n\tindex: IndexID(256),\n\t),\n")
    }

    #[test]
    #[cfg(feature = "serialize")]
    fn serialize_deserialize() {
        use super::IndexNode;
        use crate::node_core::NodeLoader;
        use crate::node_core::AnimationNodeTrait;
        use super::IndexNodeLoader;
        let asset_server = test_asset_server();
        let mut handles = Vec::new();
        for i in 0..3 {
            handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
        }
        let true_node: Box<dyn AnimationNodeTrait> = Box::new(IndexNode::new("Zombie1_Idle", &handles[..3], true));
        let mut res = String::new();
        assert!(true_node.serialize(&mut res, &asset_server).is_ok());
        let loader = IndexNodeLoader;
        let test_node = loader.load(&res, &asset_server);
        assert!(test_node.is_ok(), "{}", test_node.err().unwrap());
        let test_node = test_node.unwrap();
        assert_eq!(test_node.hash(), true_node.hash())
    }
}

#[derive(Debug, Reflect, std::hash::Hash)]
pub struct IndexNode{
    name: String,
    frames: Vec<Handle<Image>>,
    is_loop: bool,
    index: Attribute,
}

#[cfg(feature = "bevy-inspector-egui")]
impl bevy_inspector_egui::Inspectable for IndexNode {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _options: Self::Attributes, _context: &mut bevy_inspector_egui::Context) -> bool {
        let mut edit = false;
        ui.collapsing("IndexNode", |ui| {
        ui.horizontal(|ui| {
            ui.label("Name: ");
            if ui.text_edit_singleline(&mut self.name).changed() {edit = true;}
        });
        if ui.checkbox(&mut self.is_loop, "loop").changed() {edit = true;};
        });
        edit
    }
}

impl IndexNode {
    #[inline(always)]
    pub fn new(name: &str, frames: &[Handle<Image>], is_loop: bool) -> IndexNode{
        IndexNode { 
            name: name.to_string(),
            frames: frames.to_vec(),
            is_loop,
            index: Attribute::INDEX,
        }
    }

    #[inline(always)]
    pub fn new_with_index(name: &str, frames: &[Handle<Image>], is_loop: bool, index: Attribute) -> IndexNode {
        IndexNode { 
            name: name.to_string(),
            frames: frames.to_vec(),
            is_loop,
            index,
        }
    }
}

#[cfg(feature = "serialize")]
impl CanLoad for IndexNode {
    fn loader() -> Box<dyn NodeLoader> {
        Box::new(IndexNodeLoader)
    }
}

impl AnimationNodeTrait for IndexNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn node_type(&self) -> String {
        "IndexNode".to_string()
    }

    fn run(&self, state: &mut AnimationState) -> NodeResult {
        assert!(self.frames.len() != 0);
        let mut index = state.try_get_attribute::<usize>(&self.index).unwrap_or(0);
        let frames = state.get_attribute::<usize>(&Attribute::FRAMES);
        index += frames;
        if index >= self.frames.len() {
            if self.is_loop {
                index %= self.frames.len();
            } else {
                index = self.frames.len() - 1;
            }
        }
        state.set_attribute(self.index, index);
        NodeResult::Done(self.frames[index].clone())
    }

    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, context: &mut bevy_inspector_egui::Context) -> bool{
        bevy_inspector_egui::Inspectable::ui(self, ui, (), context)
    }

    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, asset_server: &bevy::prelude::AssetServer) -> Result<(), Error>
    {
        data.push_str("IndexNode(\n\t");
        data.push_str("name: \"");
        data.push_str(&self.name);
        data.push_str("\",\n\tframes: [\n\t");
        for frame in self.frames.iter() {
            if let Some(path) = asset_server.get_handle_path(frame) {
                data.push_str(path.path().to_str().unwrap())
            } else {
                return Err(Error::AssetPathNotFound(frame.clone_weak()));
            }
            data.push_str(",\n\t");
        }
        data.push_str("],\n\t");
        data.push_str(&format!("is_loop: {},\n\t",self.is_loop));
        data.push_str("index: ");
        data.push_str(&ron::to_string(&self.index)?);
        data.push_str(",\n\t),\n");
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

    fn id(&self) -> NodeId {
        NodeId::Name((&self.name).into())
    }
}

#[cfg(feature = "serialize")]
pub use loader::IndexNodeLoader;

#[cfg(feature = "serialize")]
mod loader {
use crate::{node_core::NodeLoader, prelude::Attribute};
use std::collections::HashMap;
use super::IndexNode;

use crate::prelude::{AnimationNodeTrait, BevySpriteAnimationError as Error};
pub struct IndexNodeLoader;

impl NodeLoader for IndexNodeLoader {
    fn load(&self, data: &str, asset_server: &bevy::prelude::AssetServer) -> Result<Box<dyn AnimationNodeTrait>, Error> {
        let data = data.trim();
        let data = if data.starts_with("IndexNode(") {&data[10..]} else {data};
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
                    //info!("add {} : {}", key, data[start..start+len].trim());
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
        let mut frames = Vec::new();
        for path in if let Some(paths) = map.get("frames") {
            paths[1..paths.len() - 1].split_terminator(',')
        } else {
            return Err(Error::DeserializeError{
                node_type: "IndexNode",
                message: "Failed to find frames".to_string(),
                loc: crate::here!(),
                raw: ron::de::SpannedError {code: ron::Error::MissingStructField { field: "Frames", outer: None }, position: ron::de::Position{line: 0, col: 0}},
            });
        } {
            if path.trim().len() == 0 {
                continue;
            }
            frames.push(asset_server.load(path[0..path.len()].trim()))
        }

        let index = match map.get("index") {
            Some(v) => {ron::from_str(v)?},
            None => {Attribute::INDEX}
        };
        
        let is_loop = match map.get("is_loop") {
            Some(v) => {!v.trim().starts_with("f")},
            None => {true}
        };

        let name = if let Some(v) = map.get("name") {
            v[1..v.len() - 1].to_string()
        } else {
            return Err(Error::DeserializeError{
                node_type: "IndexNode",
                message: "Failed to find name".to_string(),
                loc: crate::here!(),
                raw: ron::de::SpannedError {code: ron::Error::MissingStructField { field: "Name", outer: None }, position: ron::de::Position{line: 0, col: 0}},
                });
        };
        Ok(Box::new(IndexNode {
            name,
            frames,
            index,
            is_loop
        }))
    }

    fn can_load(&self) -> &[&str] {
        &["IndexNode"]
    }
}

}