use bevy::prelude::*;
use std::collections::HashMap;

const MAXNODEDEPTH : u8 = 255;

#[derive(Debug, Clone)]
pub struct  Frame{
    pub sprite_index : usize,
    pub sprite_sheet : Handle<TextureAtlas>,
}

pub struct NodeTree{
    nodes : Vec<Box<dyn AnimationNode>>,
    node_names : HashMap<&'static str, usize>,
    node_loaders : HashMap<String,fn(&String) -> Box<dyn AnimationNode>>
}

impl NodeTree{
    pub fn build_add(&mut self, node_builder : Box<dyn NodeBuilder>) -> bool{
        if node_builder.is_loaded(){
            let node = node_builder.build(self);
            if let Some(_) = self.node_names.get(node.get_name()){
                println!("there is already a node with the name {}; use add_replace to override the old node", node.get_name());
                return false
            } else {
                let id = self.nodes.len();
                self.node_names.insert(node.get_name(), id);
                self.nodes.insert(id, node);
            }
            return true
        }
        println!("node {} was never loaded use the builders load function and try again", node_builder.get_name());
        false
    }

    pub fn add(&mut self, node : Box<dyn AnimationNode>) -> bool{
            if let Some(_) = self.node_names.get(node.get_name()){
                println!("there is already a node with the name {}; use add_replace to override the old node", node.get_name());
                return false
            } else {
                let id = self.nodes.len();
                self.node_names.insert(node.get_name(), id);
                self.nodes.insert(id, node);
            }
            return true
    }

    pub fn build_add_replace(&mut self, node_builder : Box<dyn NodeBuilder>) -> bool {
        if node_builder.is_loaded(){
            let node = node_builder.build(self);
            if let Some(id) = self.node_names.get(node.get_name()){
                self.nodes[*id] = node;
            } else {
                let id = self.nodes.len();
                self.node_names.insert(node.get_name(), id);
                self.nodes.insert(id,node);
            }
            return true
        }
        false
    }

    pub fn add_replace(&mut self, node : Box<dyn AnimationNode>) -> bool {
            if let Some(id) = self.node_names.get(node.get_name()){
                self.nodes[*id] = node;
            } else {
                let id = self.nodes.len();
                self.node_names.insert(node.get_name(), id);
                self.nodes.insert(id,node);
            }
            return true
    }

    pub fn get_id(&self, name : &str) -> Option<usize>{
        if let Some(id) = self.node_names.get(name){
            return Some(*id);
        }
        None
    }

    pub fn get_or_insert_id(&mut self, name : &'static str) -> usize{
        if let Some(id) = self.node_names.get(name){
            return *id;
        }
        let id = self.nodes.len();
        self.node_names.insert(name, id);
        self.nodes.insert(id, Box::new(VoidNode(name)));
        id
    }

    pub fn get_name(&self, id : usize) -> Option<&'static str>{
        if id < self.nodes.len(){
            return Some(self.nodes[id].get_name())
        }
        None
    }

}

impl Default for NodeTree{
    fn default() -> Self {
        NodeTree{
            nodes : Vec::new(),
            node_names : HashMap::new(),
            node_loaders : HashMap::new(),
        }
    }
}

pub struct VoidNode(pub &'static str);

impl AnimationNode for VoidNode{
    fn run(&self, _animation_state : &mut AnimationState) {
        panic!("tried to run a node called {} it was never added to the tree; make sure you added a node with this name to this tree", self.0)
    }

    fn get_name(&self) -> &'static str {
        self.0
    }

}

pub struct Animator{
    pub state : AnimationState,
    pub temp_modifyers : Vec<String>,
    pub error_frame : Frame,
    pub root_node : usize,
}

impl Animator {
    fn next_frame(&mut self, dt : f64, tree : &NodeTree) -> Frame{
        self.state.clear(&self.temp_modifyers);
        self.state.current_node = NodeResult::NodeID(self.root_node);
        let t = if let DataType::F64(t) = self.state.data.get("TIME").unwrap() {*t} else {panic!("TIME in not set to a f64")};
        self.state.set("TIME".to_string(), DataType::F64(t + dt));
        self.state.set("DTIME".to_string(), DataType::F64(dt));
        let depth = 0;
        loop {
            if depth == MAXNODEDEPTH {println!("Depth max of {} reached terminating tree with error", MAXNODEDEPTH); return self.error_frame.clone();}
            match &self.state.current_node {
                NodeResult::Frame(f) => {return f.clone();}
                NodeResult::NodeID(id) => {tree.nodes[*id].run(&mut self.state)},
                _ => panic!(),
            }
        }
    }

    #[allow(dead_code)]
    pub fn new(root_node : &str, node_tree : &NodeTree, temp_modifyers : Vec<String>, error_frame : Frame) -> Animator{
        let state = AnimationState{
            current_node : NodeResult::NodeID(0),
            data : [("TIME".to_string(), DataType::F64(0.0))].iter().cloned().collect(),
            changed : Vec::new(),
        };
        let root_node = if let Some(id) = node_tree.get_id(root_node) {id} else {println!("could not find {} is node_tree, set root node to 0", root_node); 0};
        Animator {
            root_node,
            state,
            temp_modifyers,
            error_frame
        }
    }
}

#[derive(Debug,Clone)]
pub struct AnimationState{
    data : HashMap<String,DataType>,
    current_node : NodeResult,
    changed : Vec<String>,
}

impl AnimationState {
    pub fn print(&self){
        println!("this animation state is set to");
        for (name,value) in self.data.iter(){
            println!("{} = {:?}", name, value);
        }
        println!("and these fileds where change since last clear");
        for name in self.changed.iter(){
            println!("{}",name);
        }
        println!("");
    }
}

#[derive(Debug, Clone)]
pub enum DataType{
    Usize(usize),
    Isize(isize),
    F64(f64),
    String(String),
    Bool(bool)
}

mod data_type_cast{
    use super::DataType;
    impl From<usize> for DataType {
        fn from(u : usize) -> Self {
            DataType::Usize(u)
        }
    }
    
    impl From<isize> for DataType {
        fn from(i : isize) -> Self {
            DataType::Isize(i)
        }
    }
    
    impl From<bool> for DataType {
        fn from(b : bool) -> Self {
            DataType::Bool(b)
        }
    }
    
    impl From<f32> for DataType{
        fn from(f : f32) -> Self{
            DataType::F64(f as f64)
        }
    }
    
    impl From<f64> for DataType{
        fn from(f : f64) -> Self{
            DataType::F64(f)
        }
    }
    
    impl From<String> for DataType{
        fn from(s : String) -> Self{
            DataType::String(s)
        }
    }
    
    impl From<&str> for DataType{
        fn from(s : &str) -> Self{
            DataType::String(s.to_string())
        }
    }

    impl From<DataType> for usize{
        fn from(input : DataType) -> Self {
            match input {
                DataType::Usize(u) => u,
                _ => { println!("failed to convert {:?} to usize set to 0 instrad", input); 0}
            }
        }
    }
}

impl AnimationState {
    #[allow(dead_code)]
    pub fn set(&mut self, name : String, value : DataType){
        self.data.insert(name.clone(), value);
        self.changed.push(name);
    }
    #[allow(dead_code)]
    pub fn get(&self, name : &String) -> Option<&DataType>{
        self.data.get(name)
    }

    fn clear(&mut self, to_clear : &Vec<String>){
        for modifyer in to_clear.iter(){
            if !self.changed.contains(modifyer) && self.data.contains_key(modifyer){
                println!("cleared modifyer {} it was set to {:?}",modifyer, self.data.remove(modifyer));
            }
        }
        self.changed.clear();
    }

    pub(crate) fn set_next(&mut self, next : NodeResult){
        self.current_node = next;
    }
}


pub trait AnimationNode : Send + Sync{
    fn run(&self, animation_state : &mut AnimationState);
    fn get_name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub enum NodeResult{
    Frame(Frame),
    NodeID(usize),
    NodeName(&'static str),
}

pub(super) fn animation_update(
    mut to_animate : Query<(&mut TextureAtlasSprite, &mut Animator, &mut Handle<TextureAtlas>)>,
    time : Res<Time>,
    node_tree : Res<NodeTree>,
){
    let dt = time.delta_seconds_f64();
    for (mut sprite, mut animator, mut texture) in to_animate.iter_mut(){
        let next_frame = animator.next_frame(dt, &node_tree);
        sprite.index = next_frame.sprite_index as u32;
        texture.set(Box::new(next_frame.sprite_sheet)).expect("Failed to set Handle<TextureAtlas>");
    }
}

pub struct Cell{
    pub frame : Frame,
    pub modifyers : HashMap<String, DataType>,
}

impl Cell {

    pub fn run(&self, state : &mut AnimationState){
        for (name, value) in self.modifyers.iter(){
            state.set(name.clone(),value.clone());
        }
    }

    pub fn set(&mut self, name : String, value : DataType){
        self.modifyers.insert(name, value);
    }

    pub fn get(&self, name : &String) -> Option<&DataType>{
        self.modifyers.get(name)
    }
}

pub trait NodeBuilder : Send + Sync{
    fn get_name(&self) -> &'static str;
    fn build(self : Box<Self>, node_tree : &mut NodeTree) -> Box<dyn AnimationNode>;
    fn is_loaded(&self) -> bool;
    fn deserialize(&mut self) -> bool;
}