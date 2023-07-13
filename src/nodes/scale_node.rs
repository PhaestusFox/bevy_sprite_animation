use crate::node_core::CanLoad;
use crate::prelude::*;
use serde::{Serialize, Deserialize};
use crate::error::BevySpriteAnimationError as Error;

#[cfg(test)]
mod test {
    // use crate::test::test_asset_server;
    // use super::ScaleNode;
    // use super::ScaleNodeLoader;
    // use crate::node_core::AnimationNode;
    // use crate::node_core::NodeLoader;

    #[test]
    #[cfg(feature = "serialize")]
    fn deserialize_clean_str() {
        todo!("add tests back")
    }
    //     let asset_server = test_asset_server();
    //     let mut handles = Vec::new();
    //     for i in 0..3 {
    //         handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
    //     }
    //     let mut loader = ScaleNodeLoader;
    //     let test_node = loader.load("name: \"Zombie1_Idle\",
    //     frames: [
    //     (Zombie1/zombie1_00000.png, 0.1),
    //     (Zombie1/zombie1_00001.png, 0.2),
    //     (Zombie1/zombie1_00002.png, 0.3),
    //     ]", &asset_server).unwrap();
    // let true_node = Box::new(ScaleNode::new("Zombie1_Idle", &handles[..3], true));
    // assert_eq!(test_node.hash(), true_node.hash());
    // }

    // #[test]
    // #[cfg(feature = "serialize")]
    // fn deserialize_capped_str() {
    //     let asset_server = test_asset_server();
    //     let mut handles = Vec::new();
    //     for i in 0..3 {
    //         handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
    //     }
    //     let mut loader = ScaleNodeLoader;
    //     let test_node = loader.load("
    //         (
    //         name: \"Zombie1_Idle\",
    //         frames: [
    //         (Zombie1/zombie1_00000.png, 0.1),
    //         (Zombie1/zombie1_00001.png, 0.2),
    //         (Zombie1/zombie1_00002.png, 0.3),
    //         ],
    //         ),
    //     ", &asset_server).unwrap();
    //     let true_node: Box<dyn AnimationNode> = Box::new(ScaleNode::new("Zombie1_Idle", &handles[..3], true));
    //     assert_eq!(test_node.hash(), true_node.hash());
    // }

    // #[test]
    // #[cfg(feature = "serialize")]
    // fn deserialize_full_str() {
    //     let asset_server = test_asset_server();
    //     let mut handles = Vec::new();
    //     for i in 0..3 {
    //         handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
    //     }
    //     let mut loader = ScaleNodeLoader;
    //     let test_node = loader.load("
    //             VariableNode(
    //             name: \"Zombie1_Idle\",
    //             frames: [
    //             (Zombie1/zombie1_00000.png, 0.1),
    //             (Zombie1/zombie1_00001.png, 0.2),
    //             (Zombie1/zombie1_00002.png, 0.3),
    //             ],
    //             ),
    //     ", &asset_server).unwrap();
    //     // let node: &dyn Any = &test_node;
    //     // let node = node.downcast_ref::<VariableNode>().unwrap();
    //     // println!("{:#?}", node);
    //     let true_node: Box<dyn AnimationNode> = Box::new(ScaleNode::new("Zombie1_Idle", &handles[..3], true));
    //     assert_eq!(test_node.hash(), true_node.hash());
    // }

    // #[test]
    // #[cfg(feature = "serialize")]
    // fn serialize_str_pretty() {
    //     let asset_server = test_asset_server();
    //     let mut handles = Vec::new();
    //     for i in 0..3 {
    //         handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
    //     }
    //     let true_node: Box<dyn AnimationNode> = Box::new(ScaleNode::new("Zombie1_Idle", &handles[..3], true));
    //     let mut res = String::new();
    //     let ser_res = true_node.serialize(&mut res, &asset_server);
    //     assert!(ser_res.is_ok(), "{}", ser_res.err().unwrap());
    //     println!("{}", res);
    //     assert!(res == "VariableNode(\n\tname: \"Zombie1_Idle\",\n\tframes: [\n\t(Zombie1/zombie1_00000.png, 0.1),\n\t(Zombie1/zombie1_00001.png, 0.2),\n\t(Zombie1/zombie1_00002.png, 0.3),\n\t],\n\tis_loop: true,\n\tindex: IndexID(256),\n\t),\n")
    // }

    // #[test]
    // #[cfg(feature = "serialize")]
    // fn serialize_deserialize() {
    //     let asset_server = test_asset_server();
    //     let mut handles = Vec::new();
    //     for i in 0..3 {
    //         handles.push(asset_server.load(&format!("Zombie1/zombie1_{:05}.png", i)));
    //     }
    //     let true_node: Box<dyn AnimationNode> = Box::new(ScaleNode::new("Zombie1_Idle", &handles[..3], true));
    //     let mut res = String::new();
    //     assert!(true_node.serialize(&mut res, &asset_server).is_ok());
    //     let mut loader = ScaleNodeLoader;
    //     let test_node = loader.load(&res, &asset_server);
    //     assert!(test_node.is_ok(), "{}", test_node.err().unwrap());
    //     let test_node = test_node.unwrap();
    //     assert_eq!(test_node.hash(), true_node.hash())
    // }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScaleNode{
    name: String,
    index: Attribute,
    scale: Attribute,
    next: NodeId<'static>,
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

impl ScaleNode {
    #[inline(always)]
    pub fn new(name: &str, scale: Attribute, next: NodeId<'static>) -> ScaleNode {
        ScaleNode { 
            name: name.to_string(),
            index: Attribute::INDEX,
            scale,
            next
        }
    }

    #[inline(always)]
    pub fn new_with_index(name: &str, index: Attribute, scale: Attribute, next: NodeId<'static>) -> ScaleNode {
        ScaleNode { 
            name: name.to_string(),
            index,
            scale,
            next,
        }
    }
}

#[cfg(feature = "serialize")]
impl CanLoad for ScaleNode {
    fn loader() -> Box<dyn NodeLoader> {
        Box::new(ScaleNodeLoader)
    }
}

impl AnimationNodeTrait for ScaleNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn node_type(&self) -> String {
        "ScaleNode".to_string()
    }

    fn run(&self, state: &mut AnimationState) -> NodeResult {
        let mut index = state.try_get_attribute::<usize>(&self.index).unwrap_or(0);
        let rem_time = state.get_attribute::<f32>(&Attribute::TIME_ON_FRAME);
        let frames = state.get_attribute::<usize>(&Attribute::FRAMES);
        let last = state.get_attribute::<f32>(&Attribute::LAST_FPS);
        let scale = state.try_get_attribute::<f32>(&self.scale).unwrap_or(1.);
        let mut frame_time = last * frames as f32 + rem_time;
        let width = last * scale;
        let frames = (frame_time / width).floor();
        frame_time -= frames * width;
        index += frames as usize;

        state.set_attribute(Attribute::LAST_FPS, last * scale);
        state.set_attribute(Attribute::TIME_ON_FRAME, frame_time);
        state.set_attribute(Attribute::FRAMES, frames as usize);
        state.set_attribute(self.index, index);
        NodeResult::Next(self.next.to_static())
    }

    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, context: &mut bevy_inspector_egui::Context) -> bool{
        bevy_inspector_egui::Inspectable::ui(self, ui, (), context)
    }

    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, _: &bevy::prelude::AssetServer) -> Result<(), Error>
    {
        let mut buf =  Vec::new();
        let pretty = ron::ser::PrettyConfig::default().new_line("\n\t".to_string());
        let mut serializer = ron::Serializer::new(&mut buf, Some(pretty))?;
        serde::Serialize::serialize(self, &mut serializer)?;
        data.push_str(&String::from_utf8_lossy(&buf));
        data.push(',');
        data.push('\n');
        Ok(())
    }

    #[cfg(feature = "hash")]
    fn hash(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        self.name.hash(&mut hasher);
        self.index.hash(&mut hasher);
        self.scale.hash(&mut hasher);
        self.next.hash(&mut hasher);
        hasher.finish()
    }

    fn id(&self) -> NodeId {
        NodeId::Name((&self.name).into())
    }
}

#[cfg(feature = "serialize")]
pub use loader::ScaleNodeLoader;

#[cfg(feature = "serialize")]
mod loader {
use crate::node_core::NodeLoader;
use super::ScaleNode;

use crate::prelude::{AnimationNodeTrait, BevySpriteAnimationError as Error};
pub struct  ScaleNodeLoader;

impl NodeLoader for ScaleNodeLoader {
    fn load(&self, data: &str, _: &bevy::prelude::AssetServer) -> Result<Box<dyn AnimationNodeTrait>, Error> {
        Ok(Box::new(ron::from_str::<ScaleNode>(data)?))
    }

    fn can_load(&self) -> &[&str] {
        &["ScaleNode"]
    }
}

}