use super::core_new::AnimationState;
use super::core_new::{NodeBuild,NodeCore};
use super::core_new::NodeResult;
use super::core_new::DataType;
use super::core_new::Cell;
use super::prelude::*;

use bevy::prelude::*;

use std::collections::HashMap;

#[doc(hidden)]
pub struct TestNode{
    name : String,
    results : Vec<NodeResult>,
    mods : Vec<(String, DataType)>,
}

impl TestNode{
    pub fn new(name : &str) -> Box<dyn NodeCore>{
        Box::new(
            TestNode{
                name : format!("{}",name),
                results : vec![
                NodeResult::Test{message : format!("this is {}'s 0th path; this -> next;", name),next : format!("test_next_{}",name)},
                NodeResult::Test{message : format!("this is {}'s 1st path; this -> alt;", name),next : format!("test_alt_{}",name)},
                NodeResult::Test{message : format!("this is {}'s 2nd path; this -> temp;", name),next : format!("test_temp_{}",name)},
                NodeResult::Test{message : format!("this is {}'s 3rd path; this -> temp;", name),next : format!("test_temp_{}", name)},
                NodeResult::Test{message : format!("this is {}'s 4th path; this -> ?;", name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}'s 5th path; this -> ?;", name),next : format!("test_error")}],
                mods : vec![],
            }
        )
    }

    pub fn new_next(name : &str) -> Box<dyn NodeCore>{
        const TEST_TYPE : &'static str = "next";
        Box::new(
            TestNode{
                name : format!("test_next_{}",name),
                results : vec![
                NodeResult::Frame(Frame{index : 0, ..Default::default()}),
                NodeResult::Test{message : format!("this is {}_{}'s 1st path; this -> ?;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}_{}'s 2nd path; this -> ?;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}_{}'s 3rd path; this -> ?;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}_{}'s 4th path; this -> ?;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}_{}'s 5th path; this -> ?;",TEST_TYPE, name),next : format!("test_error")}],
                mods : vec![("alt".to_string(), true.into())],
            }
        )
    }

    pub fn new_alt(name : &str) -> Box<dyn NodeCore>{
        const TEST_TYPE : &'static str = "alt";
        Box::new(
            TestNode{
                name : format!("test_alt_{}",name),
                results : vec![
                NodeResult::Test{message : format!("this is {}_{}'s 0th path; this -> ?;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Frame(Frame{index : 1, ..Default::default()}),
                NodeResult::Frame(Frame{index : 2, ..Default::default()}),
                NodeResult::Test{message : format!("this is {}_{}'s 3rd path; this -> ?;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}_{}'s 4th path; this -> ?;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}_{}'s 5th path; this -> ?;",TEST_TYPE, name),next : format!("test_error")}],
                mods : vec![("temp".to_string(), true.into())],
            }
        )
    }

    pub fn new_temp(name : &str) -> Box<dyn NodeCore>{
        const TEST_TYPE : &'static str = "temp";
        Box::new(
            TestNode{
                name : format!("test_temp_{}",name),
                results : vec![
                NodeResult::Test{message : format!("this is {}_{}'s 0th path; this -> ?; set alt = false;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}_{}'s 1st path; this -> ?; set alt = false;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Frame(Frame{index : 3, ..Default::default()}),
                NodeResult::Test{message : format!("this is {}_{}'s 3rd path; this -> alt; set alt = false;",TEST_TYPE, name),next : format!("test_alt_{}",name)},
                NodeResult::Test{message : format!("this is {}_{}'s 4th path; this -> ?; set alt = false;",TEST_TYPE, name),next : format!("test_error")},
                NodeResult::Test{message : format!("this is {}_{}'s 5th path; this -> ?; set alt = false;",TEST_TYPE, name),next : format!("test_error")}],
                mods : vec![("alt".to_string(), false.into())],
            }
        )
    }
    pub fn error() -> Box<dyn NodeCore>{
        const TEST_TYPE : &'static str = "error";
        let name = format!("an error");
        Box::new(
            TestNode{
                name : format!("test_error"),
                results : vec![
                NodeResult::Error(format!("this is {}_{}'s 0th path",TEST_TYPE, name)),
                NodeResult::Error(format!("this is {}_{}'s 1st path",TEST_TYPE, name)),
                NodeResult::Error(format!("this is {}_{}'s 2nd path",TEST_TYPE, name)),
                NodeResult::Error(format!("this is {}_{}'s 3rd path",TEST_TYPE, name)),
                NodeResult::Error(format!("this is {}_{}'s 4th path",TEST_TYPE, name)),
                NodeResult::Error(format!("this is {}_{}'s 5th path",TEST_TYPE, name)),],
                mods : vec![],
            }
        )
    }
}

impl NodeCore for TestNode{
    fn get_name(&self) -> &str {
        &self.name
    }

    fn run(&self, animation_state : &mut AnimationState) {
        let mut res = 0;
        if animation_state.get::<bool>("alt").unwrap_or(false){
            res += 1;
        }
        if animation_state.get::<bool>("temp").unwrap_or(false){
            res += 2;
        }
        for (name, value) in self.mods.iter(){
            println!("{} is setting {}", self.name, name);
            animation_state.set(name, value.clone());
        }
        animation_state.set_next(self.results[res].clone())
    }

    fn print(&self) {
        todo!();
    }
}

impl NodeBuild for TestNode{
    fn build(self : Box<Self>, _world : &bevy::ecs::world::WorldCell) -> Box<dyn NodeCore> {
        todo!();
    }
}


///this is a node that holds an id open with a sesific name;
///was added so that nodes can be built without all the down stream nodes needing to exist when it is made
pub(crate) struct VoidNode(pub(crate) String);

impl VoidNode{
    pub(crate) fn new(name : &str) -> Box<dyn NodeCore>{
        Box::new(
            VoidNode(name.to_string())
        )
    }
}

impl NodeCore for VoidNode{
    fn get_name(&self) -> &str {
        &self.0
    }
    fn run(&self, animation_state : &mut AnimationState){
        animation_state.set_next(NodeResult::Error(format!("Tried to run the void node {}", self.0)))
    }
    fn print(&self){
        println!("void node called {}", self.0);
    }
}

/**
A simple node that just pics the Cell in its cells vec with the index of its driver in the state peramiters
# Plz Help
    this node is sort of a place holder prof of concept node that i will probably re-write at some point;
    plz provide feedback on any nodes that you want added this one is extreamly simple but also powerful simply becuse it can fully drive the state its self
    allowing for more complex builders to use this basic node and make diffrent types of node without even changing this one eg and auto_incrementing node
    would just be a basic node where the builder automaticly fills in the cell modifyers to set the driver to the next cell --Will Definitly be remaking it tho
    its way to hard to get spesific behaviers out of it without needing to hardcode every cell
    am thinging a hashmap with the cell modifyes and a vec of frames to get the same result but you would easily be able to auto generate the frames and then overrided just
    spesific hashmap modiyers instead of hardcoding all the frames just the ones you override 
*/
pub struct BasicNode{
    pub name : String,
    pub driver : String,
    pub cells : Vec<Cell>,
    pub modifyers : Vec<(String, DataType)>,
}

impl NodeCore for BasicNode{
    fn get_name(&self) -> &str {
        &self.name
    }

    fn run(&self, animation_state : &mut AnimationState) {
        let index = if let Some(index) = animation_state.get::<usize>(&self.driver) {index} else {0};
        if index >= self.cells.len() {animation_state.set_next(NodeResult::Error(format!("Index {} bigger then cells.len {}", index, self.cells.len()))); return;}
        
        for (name, value) in self.modifyers.iter(){
            animation_state.set(name, value.clone());
        }

        let res = &self.cells[index];

        for (name, value) in res.modifyers.iter(){
            animation_state.set(name, value.clone());
        }

        animation_state.set_next(NodeResult::Frame(res.frame.clone()))
    }

    fn print(&self){
        println!("basic node called {}", self.get_name());
    }
}

/**
A simple switch node that will go to node A if Condishon is true Or node B if its false Or node C If something breaks i.e. the condishon was never set
*/
pub struct SwitchNode{
    pub name : String,
    pub driver : String,
    pub true_node : NodeResult,
    pub false_node : NodeResult,
    pub fallback_node : NodeResult,
    pub modifyers : Vec<(String, DataType)>
}

impl NodeCore for SwitchNode{
    fn get_name(&self) -> &str {
        &self.name
    }

    fn run(&self, animation_state : &mut AnimationState) {
        if let Some(drive) = animation_state.get::<bool>(&self.driver){
            if drive {
                animation_state.set_next(self.true_node.clone())
            }
            else{
                animation_state.set_next(self.false_node.clone())
            }
        }else{
            animation_state.set_next(self.fallback_node.clone())
        }
        for (name, value) in self.modifyers.iter(){
            animation_state.set(name, value.clone());
        }
    }
    fn print(&self){
        println!("switch node called {}", self.get_name());
    }
}

/**
 A simple spritesheetnodebuilder will create a basic node out of the given infromation about the sprite sheet
 # Arguments
    `name` -String the name of the node<br>
    `driver` -String the peramiter used by the resulting node to pic a cell<br>
    `path` -String the path given to the AssetServer to load the sprite sheet<br>
    `tile_size` -Vec2 the size or the tiles give to the TextureAtles::from_grid()<br>
    `sprite_sheet_size` -(usize,usize) the number of colums and rows given to the TextureAtles::from_grid()<br>
    `start_index` -u32 the index of the first frame on the TextureAtles<br>
    `end_index` -u32 the index of the last fream on the TextureAtles<br>
    `auto_inc` -bool wether or not to make each cell point to the next cell automaticly<br>
    `cell_mods` -HashMap<usize,Vec<(String,DataType)>> a hashmap with the modiyers for each cell Key is the cells index and the value is a Vec<(String,DataType)> that is added to the coresponding cell<br>
 # Example
 ```
 SpriteSheetNodeBuilder{
        name : IDLENODE.to_string(),
        driver : "idle".to_string(),
        path : "test.png".to_string(),
        tile_size : Vec2::splat(60.0),
        sprite_sheet_size : (3,3),
        start_index : 0,
        end_index : 2,
        auto_inc : true,
        cell_mods : HashMap::new(),
        node_mods : Vec::new()
    }
    ```
    */
pub struct SpriteSheetNodeBuilder{
    pub name : String,
    pub driver : String,
    pub path : String,
    pub tile_size : Vec2,
    pub sprite_sheet_size : (usize,usize),
    pub start_index : u32,
    pub end_index : u32,
    pub auto_inc : bool,
    pub cell_mods : HashMap<usize,Vec<(String,DataType)>>,
    pub node_mods : Vec<(String,DataType)>,
}

impl NodeBuild for SpriteSheetNodeBuilder{
    fn build(mut self : Box<Self>, world : &bevy::ecs::world::WorldCell) -> Box<dyn NodeCore> {
        let mut cells = Vec::new();
        let asset_server = world.get_resource::<AssetServer>().expect("failed to get AssetServer");
        let texture = asset_server.load(&self.path[..]);
        let texture_atlas = world.get_resource_mut::<Assets<TextureAtlas>>().expect("failed to get Assets<TextureAtlas>").add(
            TextureAtlas::from_grid(texture, self.tile_size, self.sprite_sheet_size.0, self.sprite_sheet_size.1)
        );
        let mut i = 0;
        let cell_count = self.end_index - self.start_index;
        for index in self.start_index..=self.end_index {
            let mut cell = Cell{
                frame : Frame{
                    index,
                    sprite_sheet : texture_atlas.clone(),
                    ..Default::default()
                },
                modifyers : if let Some(mods) = self.cell_mods.remove(&i){
                    mods
                } else { Vec::new() },
            };
            if self.auto_inc { 
                if i == cell_count as usize {
                    cell.modifyers.push((self.driver.clone(), DataType::Usize(0)));
                }
                else {cell.modifyers.push((self.driver.clone(), DataType::Usize(i+1)))}
            }
            //println!("count = {}; \ncell = {:?}", i, cell);
            cells.insert(i,cell);
            i += 1;
        }

        Box::new(
            BasicNode{
                name : self.name,
                driver : self.driver,
                cells,
                modifyers : self.node_mods,
            }
        )
    }
}