use std::{collections::HashMap, usize};

use bevy::prelude::*;
use super::core::*;

pub struct BasicNode{
    pub name : &'static str,
    pub driver : String,
    pub auto_increment : bool,
    pub cells : Vec<Cell>,
    pub modifyers : HashMap<String, DataType>,
    pub frame_rate : usize,
}

impl AnimationNode for BasicNode{
    fn run(&self, animation_state : &mut AnimationState) {
        let mut index = match animation_state.get(&self.driver.clone()) {
            Some(i) => match i {
                DataType::Usize(i) => {*i},
                _ => 0
            },
            None => {animation_state.set(self.driver.clone(), DataType::Usize(0)); 0}
        };
        for (name,value) in self.modifyers.iter(){
            animation_state.set(name.clone(),value.clone());
        }
        if self.auto_increment {
            let t = if let DataType::F64(t) = animation_state.get(&"TIME".to_string()).unwrap(){
                *t
            } else {panic!("TIME feld in animation_state not set");};
            let frames = (self.frame_rate as f64 * t).floor();
            animation_state.set("TIME".to_string(),DataType::F64(t - 1.0/self.frame_rate as f64 * frames));
            index = (index + frames as usize) % self.cells.len();

            animation_state.set(self.driver.clone(), DataType::Usize(index));
        }
        if index < self.cells.len() {
            self.cells[index].run(animation_state);
            animation_state.set_next(NodeResult::Frame(self.cells[index].frame.clone()));
        } else {
            self.cells[0].run(animation_state);
            animation_state.set_next(NodeResult::Frame(self.cells[index].frame.clone()));
        }
    }
    fn get_name(&self) -> &'static str{
        self.name
    }
}

pub struct BasicNodeBuilder{
    pub name : &'static str,
    pub driver : String,
    pub sprite_sheet_path : String,
    pub auto_increment : bool,
    pub start_frame : usize,
    pub last_frame : usize,
    pub modifyers : Vec<(String, DataType)>,
    pub frame_rate : usize,
    pub sprite_size : Vec2,
    pub sprite_sheet_size : (usize,usize),
    sprite_sheet_handel : Option<Handle<TextureAtlas>>
}

impl NodeBuilder for BasicNodeBuilder{
    fn build(self : Box<Self>, _node_tree : &mut NodeTree) -> Box<dyn AnimationNode> {
        let modifyers = self.modifyers.iter().cloned().collect();
        let mut cells = Vec::new();
        let (true_start, true_end) = 
        if self.start_frame > self.last_frame {println!("Start({}) is begger then end({}) inverting", self.start_frame, self.last_frame); (self.last_frame,self.start_frame)}
        else {(self.start_frame,self.last_frame)};
        if let Some(texture_atles_handle) = self.sprite_sheet_handel {
            for i in true_start..=true_end{
                cells.push(
                    Cell{
                        frame : Frame {sprite_index : i, sprite_sheet : texture_atles_handle.clone()},
                        modifyers : HashMap::new()
                    }
                );
            }
        } else {panic!("the node was never loaded and you are now trying to build it; use self.load() and try again")}
        Box::new(BasicNode{
            name : self.name,
            driver : self.driver,
            auto_increment : self.auto_increment,
            cells,
            modifyers,
            frame_rate : self.frame_rate
        })
    }

    fn is_loaded(&self) -> bool{
        if let Some(_) = self.sprite_sheet_handel{
            return true
        }
        false
    }

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn deserialize(&mut self) -> bool {
        todo!()
    }
}

impl BasicNodeBuilder{
    pub fn new(name : &'static str, driver : String, auto_increment : bool, frame_rate : usize, start_index : usize, end_index : usize, sprite_sheet_path : String, sprite_size : Vec2, sprite_sheet_size : (usize,usize), modifyers : Vec<(String, DataType)>) -> Self
    {
        BasicNodeBuilder{
            name,
            driver,
            auto_increment,
            frame_rate,
            start_frame : start_index,
            last_frame : end_index,
            sprite_sheet_path,
            sprite_size,
            sprite_sheet_size,
            modifyers,
            sprite_sheet_handel : None,
        }
    }
}

struct SwitchNode{
    pub name : &'static str,
    pub driver : String,
    pub true_node : usize,
    pub false_node : usize,
    pub modifyers : HashMap<String, DataType>,
    pub fallback_node : usize,
}

impl AnimationNode for SwitchNode{
    fn run(&self, animation_state : &mut AnimationState) {
        if let Some(DataType::Bool(g)) = animation_state.get(&self.driver) {
            if *g { animation_state.set_next(NodeResult::NodeID(self.true_node));}
            else { animation_state.set_next(NodeResult::NodeID(self.false_node));}
        } else {
            animation_state.set_next(NodeResult::NodeID(self.fallback_node));
        }
    }

    fn get_name(&self) -> &'static str {
        self.name
    }
}

pub struct SwitchNodeBuilder{
    pub name : &'static str,
    pub driver : String,
    pub true_node : &'static str,
    pub false_node : &'static str,
    pub modifyers : Vec<(String, DataType)>,
    pub fallback_node : &'static str
}

impl NodeBuilder for SwitchNodeBuilder{
    fn build(self : Box<Self>, node_tree : &mut NodeTree) -> Box<dyn AnimationNode> {
        let true_node = node_tree.get_or_insert_id(self.true_node);
        let false_node = node_tree.get_or_insert_id(self.false_node);
        let fallback_node = node_tree.get_or_insert_id(self.fallback_node);
        let modifyers = self.modifyers.iter().cloned().collect();
        Box::new(SwitchNode{
            name : self.name,
            driver : self.driver,
            true_node,
            false_node,
            modifyers,
            fallback_node
        })
    }
    fn is_loaded(&self) -> bool {
        true
    }

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn deserialize(&mut self) -> bool {
        todo!()
    }
}

impl BasicNodeBuilder{
    pub fn load(mut self, asset_server : &Res<AssetServer>, texture_atlas : &mut ResMut<Assets<TextureAtlas>>) -> Box<dyn NodeBuilder>{
        let texture_handle = asset_server.load(&self.sprite_sheet_path[..]);
        let texture_at = TextureAtlas::from_grid(texture_handle, self.sprite_size, self.sprite_sheet_size.0, self.sprite_sheet_size.1);
        let texture_atlas_handle = texture_atlas.add(texture_at);
        self.sprite_sheet_handel = Some(texture_atlas_handle);
        Box::new(self)
    }
}