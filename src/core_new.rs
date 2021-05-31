use bevy::{ecs::world::WorldCell, prelude::*};
use super::nodes::VoidNode;
use std::{collections::HashMap, marker::PhantomData};

/// Used to stop infinite loops if an animation node loops back to a parent node but the state has not changed in a way to branch the path
const MAXNODEDEPTH : u8 = 255;

/**
This is the tree that contains all the nodes for an animation of try T
it is add as a resource when sprite_animation::AnimationPlugin<T> is added to your app
# Example
 ```
    use bevy::prelude::*;
    use sprite_animation::prelude::*;
    use nodes::TestNode;
    struct TestFlag;
    fn main{
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(AnimationPlugin::<TestFlag>::default())
    .add_startup_system(start.system())
    .add_system(update.system())
    .run();
    }

    fn start(
     mut commands : Commands,
     mut node_tree : ResMut<NodeTree<MainAnimation>>,
    ){
        commands.spawn_bundle(OrthographicCameraBundle::new_2d());
        let error_frame = Frame{
            index : 0,
            sprite_sheet : Default::default
        }
        commands.spawn_bundle(SpriteSheetBundle).insert(
        Animator::new(
            vec!["test_temp".to_string()],
            Frame{index : 0, sprite_sheet : Default::default},
            "root_node",
            10,
        ))
        .insert(TestFlag); //the animator will use this nodetree

        node_tree.add(TestNode::new_next("root_node", "end_node")); //adds a node called "root_node" to the tree that just points to the next node called "end_node"
        node_tree.add(TestNode::new("end_node")); //adds a node called "end_node" that just prints the error "This is a blank test node called end_node"

 }

 fn update(test_animators : Query<&Animator, With<TestFlag>>){
    //do stuff to set the animators state for next frame
 }
 ```
 # Plz Help
 I beleve setting an animatior to use two flags will make it tick twice as fast but as long as both trees contain a node that can be its root it should not panic?
 plz give me ideas of a better way; dont want to flag the animators so and entitys tag can be changed and it will change the tree without needing a new animator
*/
pub struct NodeTree<T : Send + Sync>{
    /// A vec of node builders that will be built and added to the tree at then begining of the next frame
    to_build : Vec<Box<dyn NodeBuild>>,
    /// A vec of nodes that this tree contains
    nodes : Vec<Box<dyn NodeCore>>,
    /// A map of node names to there coresponding index into NodeTree.nodes
    node_names : HashMap<String, usize>,
    /// The flag for animation group
    marker : PhantomData<T>
}
impl<T : Send + Sync> Default for NodeTree<T>{
    fn default() -> Self{
        NodeTree{
            to_build : Vec::new(),
            nodes : Vec::new(),
            node_names : HashMap::new(),
            marker : PhantomData::default(),
        }
    }
}
impl<T: 'static + Send + Sync> NodeTree<T>{
    /// Adds a node to this tree returning false if it failed
    /// # Arguments
    /// * `node` - A Box to be added to the tree
    /// # Example
    /// ```
    /// use sprite_animation::prelude::*;
    /// use nodes::TestNode;
    /// struct TestFlag;
    /// let duplicate_name = "duplicate";
    /// let unique_name = "unique";
    /// let mut nodetree = NodeTree::<TestFlag>::default();
    /// let node = TestNode::new(duplicate_name);
    /// nodetree.add(node);
    /// //returns true if no node with that name is in the tree
    /// let node_unique = TestNode::new(unique_name);
    /// assert!(nodetree.add(node_unique));
    /// //returns false if node is already in the tree
    /// let node_duplicate = TestNode::new(duplicate_name);
    /// assert!(!nodetree.add(node_duplicate));
    /// ```
    pub fn add(&mut self, node : Box<dyn NodeCore>) -> bool{
        if let Some(_) = self.node_names.get(node.get_name()){
            println!("there is already a node with the name {}; use add_replace to override the old node", node.get_name());
            return false
        } else {
            let id = self.nodes.len();
            self.node_names.insert(node.get_name().to_string(), id);
            self.nodes.insert(id, node);
        }
        return true;
    }

    /// Adds a node to this tree replacing a prexisting node if it exists  
    /// > `Always returns true because its just a copy of add but with the false outcome corrected;
    /// > I might change this to a Reslut<(),AnimationErr> but I didn't think/am to lazy now to add this to the fist public build but if it is need I will do so`
    /// # Arguments
    /// * `node` - A Box to be added to the tree
    /// # Example
    /// ```
    ///     use sprite_animation::prelude::*;
    ///     use nodes::TestNode;
    ///     struct TestFlag;
    ///     let duplicate_name = "duplicate";
    ///     let unique_name = "unique";
    ///     let mut nodetree = NodeTree::<TestFlag>::default();
    ///     let node = TestNode::new(duplicate_name);
    ///     nodetree.add(node);
    ///     //returns true if no node with that name is in the tree
    ///     let node_unique = TestNode::new(unique_name);
    ///     assert!(nodetree.add_replace(node_unique));
    ///     //returns true even if the node exist
    ///     let node_duplicate = TestNode::new(duplicate_name);
    ///     assert!(nodetree.add_replace(node_duplicate));
    /// ```
    pub fn add_replace(&mut self, node : Box<dyn NodeCore>) -> bool{
        if let Some(id) = self.node_names.get(node.get_name()){
            self.nodes[*id] = node;
        } else {
            let id = self.nodes.len();
            self.node_names.insert(node.get_name().to_string(), id);
            self.nodes.insert(id, node);
        }
        return true;
    }

    /// Returns the node ID as `Some(usize)` if a node with givin name is in this tree else returns `None`
    /// # Arguments
    /// `name` - The name coresponding to the node Id you are looking for
    /// # Return
    /// >`Some(usize)` if that node is in the tree
    /// >`None` if that node is not in the tree
    pub fn get_id(&self, name : &str) -> Option<usize>{
        if let Some(id) = self.node_names.get(name){
            return Some(*id);
        }
        None
    }
    
    /// Returns the node ID `usize` if a node with givin name is in this tree else insets a VoidNode with that name and returns its ID
    /// # Arguments
    /// `name` - The name coresponding to the node Id you are looking for
    /// # Return
    /// will return a `usize` representing that nodes location in the tree
    pub fn get_or_create_id(&mut self, name : &str) -> usize{
        if let Some(id) = self.node_names.get(name){
            return *id;
        }
        let id = self.nodes.len();
        self.node_names.insert(name.to_string(), id);
        self.nodes.insert(id, VoidNode::new(name));
        id
    }
    
    /// Returns the name of a node at a given ID
    /// # Arguments
    /// `id` - a `usize` that represents the nodes location in the tree
    /// # Return
    /// >`Some(&str)` of the node with that ID in the tree
    /// >`None` if that ID is not in the tree
    pub fn get_name(&self, id : usize) -> Option<&str>{
        if id < self.nodes.len(){
            return Some(self.nodes[id].get_name());
        }
        None
    }
    
    /// Calls print on all nodes in the tree and appends there id to the frount
    pub fn print(&self){
        println!("NodeTree Nodes = ");
        for (i, node) in self.nodes.iter().enumerate(){
            format!("node {} = {:?}",i, node.print());
        }
    }

    /// the function called at the begining of each frame to build and nodes on the to build list
    /// this function returns as soon as len == 0;
    /// # Plz Help 
    /// I have no idea how costly this is since it callse world.cell && world_cell.get_resource_mut befor it can even check if len == 0 each frame.<br>
    /// let me know if there is a better way to do this proferably one that would atleast let all NodeTrees run in parallel?
    /// It took me like a week just to find out how to get dynamic access to the world so build nodes did not have restriction on what they could use when initalizing them self
    pub fn build(world : &mut World){
        let world_cell = world.cell();
        let mut node_tree = world_cell.get_resource_mut::<NodeTree<T>>().expect("Failed to get nodetree for {}");
        let len = node_tree.to_build.len();
        if len == 0 {return;}
        for _ in 0..len{
            let node = node_tree.to_build.pop().expect("Poped more then len");
            node_tree.add(node.build(&world_cell));
        }
    }

    /// Add a node builder to this trees to build list to be built at the begining of the next frame
    /// # Arguments
    /// `node` - a Boxed NodeBuild
    /// # Plz Help
    /// this is the best i could come upwith that will allow me give a node builder world access `by storing in and using it in the fn build latter`
    pub fn add_build(&mut self, node : Box<dyn NodeBuild>){
        self.to_build.push(node);
    }
}

/**
this is the animatior Component;
attach this to the entity you want it to animate
# Example
 ```
 struct PlayerData { is_falling : bool, /*some other stuff*/ };
 struct Player;
 fn update(player_animator : Query<&mut Animator, With<Player>>,
    player_data : Res<PlayerData>,){
    //do stuff to set the animators state for next frame
    player_animator.single_mut().expect("failed to get player").set("is_falling", player_data.is_falling);
 }
 ```
*/
pub struct Animator{
    animation_state : AnimationState,
    pub temp_drivers : Vec<String>,
    pub error_frame : Frame,
    pub root_node : NodeResult,
    pub frame_rate : usize,
    pub time_since_last_frame : f32,
}
impl Animator{
    /**
    This will resolve the animator with the given nodetree and returns a frame;
    # Arguments
    * `frames` - the number of frames run - this is done so that if one frame relise on a state set in a previuse frame instead just skiping over it
    * `node_tree` - the node tree use to resolve the frame
    # Example
    ```
    fn update<T:'static + Send + Sync>(
    mut animators : Query<&mut Animator>,
    node_tree : Res<NodeTree<T>>,){
        for animator in animators.iter_mut(){
            let next_frame = animator.next_frame(frames as usize, &node_tree);
            //do something with the frame
        }
    }
    ```
    # Plz Help
    Should this return Result instead of the animators error frame
    */
    pub fn next_frame<T :'static + Send + Sync>(&mut self, frames : usize, node_tree : &NodeTree<T>) -> Frame{
        let mut _ret = self.error_frame.clone();
        for _ in 0..frames {
            self.animation_state.clear(&self.temp_drivers);
            self.animation_state.set_next(self.root_node.clone());
            let mut depth = 0;
            loop{
                if depth == MAXNODEDEPTH {println!("Depth max of {} reached terminating tree with error", MAXNODEDEPTH); return self.error_frame.clone();}
                match self.animation_state.get_next_node() {
                    NodeResult::Frame(f) => {_ret = f.clone(); break;}
                    NodeResult::NodeID(id) => {node_tree.nodes[*id].run(&mut self.animation_state)},
                    NodeResult::NodeName(name) => {node_tree.nodes[node_tree.get_id(&name).expect("Tried to get node that does not exist")].run(&mut self.animation_state)},
                    NodeResult::Error(e) => {println!("{}",e); return self.error_frame.clone()},
                    NodeResult::Test{message: messages, next} => {println!("{}", messages); node_tree.nodes[node_tree.get_id(&next).expect("Tried to get node that does not exist")].run(&mut self.animation_state)},
                    NodeResult::Null => {println!("Root Node was never set");return self.error_frame.clone();}
                }
                depth += 1;
            }
        }
        _ret
    }
    /**
    Sets a peramiter to a value in the animators state
    # Arguments
    * `name` - the name of the peramiter to set
    * `to` - an datatype that impl `Into<DataType>`
    # Example
    ```
        struct PlayerData { is_falling : bool, /*some other stuff*/ };
        struct Player;
        fn update(player_animator : Query<&mut Animator, With<Player>>,
            player_data : Res<PlayerData>,){
            //do stuff to set the animators state for next frame
            player_animator.single_mut().expect("failed to get player").set("is_falling", player_data.is_falling);
    }
    ```
    */
    pub fn set<D : Into<DataType>>(&mut self, name : &str, to : D){
        self.animation_state.set(name, to.into())
    }
    /**
    This will return the value of a peramiter in the animators state
    # Arguments
    * `name` - the name of the peramiter to get
    # Return
        `Some(D)` when that peramiter exists and can be cast from DataType
        `None` if the peramiter does not exist
    # Example
    ```
        struct Player;
        fn update(player_animator : Query<&Animator, With<Player>>,){
            let step = player_animator.single_mut().expect("failed to get player").get::<bool>("step");
            //do stuff to set the animators state for next frame
            if fun {
                println!("a step was taken last turn so you could play a step sound here");
            }
        }
    ```
    # Plz Help
    how would I change this to an event system? this would not get replaced (it has other uses beyond event trigers),
    but it does seem optuse and expense to set a peramiter only as an event flag
    */
    pub fn get<D : From<DataType> + Default>(&self, name : &str) -> Option<D>{
        if let Some(res) = self.animation_state.get::<D>(name){
            return Some(res);
        }
        None
    }
    /**
    Create a new animator
    # Arguments
    * `temp_drivers` - a vec of string that represent drivers that will only last one frame after the last time they were set
    * `error_frame` - the frame that is returned if an error occures when resolving a frame
    * `root_node` - the Node to use as the root node can be any thing that impl `Into<NodeResult>`
    * `frame_rate` - the frame rate of this animator allows for things like animation on dubbles by setting it to half the game frame rate
    */
    pub fn new(temp_drivers : Vec<String>, error_frame : Frame, root_node : impl Into<NodeResult>, frame_rate : usize) -> Animator{
        Animator{
            animation_state : AnimationState::new(),
            temp_drivers,
            error_frame,
            root_node : root_node.into(),
            frame_rate,
            time_since_last_frame : 0.0,
        }
    }
    ///Prints the animators state
    pub fn print(&self){
        println!("");
        self.animation_state.print();
    }

    ///this really needs to be fixed but I just want to get the public build done so I can start getting feed back
    pub(crate) fn get_string_for_test(&self) -> String{
        match &self.animation_state.get_next_node() {
            NodeResult::NodeName(name) => {name.clone()}
            NodeResult::Error(message) => {message.clone()}
            NodeResult::Frame(frame) => {format!("Frame:{}",frame.index)}
            _ => "Not and error or name".to_string(),
        }
    }

    ///prints the path down the tree that was taken, `verbose` will print out error mesages and test data
    #[cfg(feature = "node_trace")]
    pub fn path_to_string(&self, verbose : bool) -> String{
        self.animation_state.path_to_string(verbose)
    }
}

///A single frame of an animation
///will add fliping x and y in here at some point
#[derive(Debug, Clone,Default)]
pub struct Frame{
    ///The index of the sprite sheet this frame is at
    pub index : u32,
    ///The handle to the TextureAtlas this frame is on
    pub sprite_sheet : Handle<TextureAtlas>,
    pub flip_x : bool,
    pub flip_y : bool,
}

#[cfg(feature = "node_trace")]
pub struct AnimationState{
    data : HashMap<String, DataType>,
    path : Vec<NodeResult>,
    changed : Vec<String>,
}

#[cfg(feature = "node_trace")]
impl AnimationState{
    fn clear(&mut self, to_clear : &[String]){
        for driver in to_clear.iter(){
            if !self.changed.contains(driver) && self.data.contains_key(driver){
                println!("{} && {} = Delete {}", !self.changed.contains(driver), self.data.contains_key(driver), driver);
                self.data.remove(driver);
            }
        }
        self.changed.clear();
        self.path.clear();
    }
    pub fn set_next(&mut self, node : NodeResult){
        self.path.push(node)
    }
    pub fn set<D : Into<DataType>>(&mut self,name : &str, to : D){
        self.changed.push(name.to_string());
        self.data.insert(name.to_string(), to.into());
    }
    pub fn get<D : From<DataType>>(&self,name : &str) -> Option<D>{
        if let Some(data) = self.data.get(name){
            return Some(D::from(data.clone()))
        }
        None
    }

    fn new() -> AnimationState{
        AnimationState {
            data : HashMap::new(),
            path : vec![NodeResult::Null],
            changed : Vec::new()
        }
    }

    fn get_next_node(&self) -> &NodeResult{
        self.path.last().expect("path is empty")
    }

    ///Prints all peramiters and there values in this state along with a list of all values changed since last frame and the current next node set
    fn print(&self){
        println!("the peramiters are : ");
        for (name, value) in self.data.iter(){
            println!("{} : {:?}", name, value);
        }
        println!("{:-<10}","");
        println!("\nThese peramiters have changed since last frame");
        for changed in self.changed.iter(){
            println!("{}", changed);
        }
        println!("{:-<10}","");
        println!("\nthe path was : \n{:-<10}\n","");
        for node in self.path.iter(){
            print!("{:?} -> ", node);
        }
        print!("Out")
    }

    #[cfg(feature = "node_trace")]
    fn path_to_string(&self, verbose : bool) -> String{
        let mut res = "start -> ".to_string();
        for node in self.path.iter(){
            match node{
                NodeResult::Null => {res.push_str("null -> ")}
                NodeResult::NodeID(id) => {res.push_str(&format!("{} -> ",id))}
                NodeResult::NodeName(name) => {res.push_str(&format!("{} -> ",name))}
                NodeResult::Frame(frame) => {if !verbose {res.push_str("Frame -> ")} else {res.push_str(&format!("{:?} -> ",frame))}}
                NodeResult::Error(error) => {if !verbose {res.push_str("Error -> ")} else {res.push_str(&format!("Error({:?}) -> ",error))}}
                NodeResult::Test { message, next } => {if !verbose {res.push_str(&format!("{} -> ", next))} else {res.push_str(&format!("{}({}) -> ",next,message))}}
            }
        }
        res.push_str("End");
        res
    }
}

#[cfg(not(feature = "node_trace"))]
pub struct AnimationState{
    data : HashMap<String, DataType>,
    next_node : NodeResult,
    changed : Vec<String>,
}

#[cfg(not(feature = "node_trace"))]
impl AnimationState{
    fn clear(&mut self, to_clear : &[String]){
        for driver in to_clear.iter(){
            if !self.changed.contains(driver) && self.data.contains_key(driver){
                println!("{} && {} = Delete {}", !self.changed.contains(driver), self.data.contains_key(driver), driver);
                self.data.remove(driver);
            }
        }
        self.changed.clear()
    }

    pub fn set_next(&mut self, node : NodeResult){
        self.next_node = node
    }
    pub fn set<D : Into<DataType>>(&mut self,name : &str, to : D){
        self.changed.push(name.to_string());
        self.data.insert(name.to_string(), to.into());
    }
    pub fn get<D : From<DataType>>(&self,name : &str) -> Option<D>{
        if let Some(data) = self.data.get(name){
            return Some(D::from(data.clone()))
        }
        None
    }

    fn new() -> AnimationState{
        AnimationState {
            data : HashMap::new(),
            next_node : NodeResult::Null,
            changed : Vec::new()
        }
    }

    fn get_next_node(&self) -> &NodeResult{
        &self.next_node
    }

    ///Prints all peramiters and there values in this state along with a list of all values changed since last frame and the current next node set
    fn print(&self){
        println!("the peramiters are : ");
        for (name, value) in self.data.iter(){
            println!("{} : {:?}", name, value);
        }
        println!("{:-<10}","");
        println!("\nThese peramiters have changed since last frame");
        for changed in self.changed.iter(){
            println!("{}", changed);
        }
        println!("{:-<10}","");
        println!("\nthe next node to check is \n{:-<10}\n{:?}\n{:-<10}","",self.next_node,"")
    }
}

///is a cell of animation it holds the Frame that will get set and also the peramiters that will be set because this frame was selected
///# Plz Help
///may abstract this away to just being a frame and making the node that pics the cell responsable for applying the modifyers for a spesific Frame getting picked
///would make more sense for a node to decide how to modiy the state on a case by case then to add a Type that the node just reeds data out of.
///for example could just have a HashMap in the BasicNode that applies spsific peramiters hashed as the index of the Frame instead of storing a vec of cell most of with
///dont actualy have and modifyers beyond changing the driver of the node witch is set up by a for loop in the auto_inc builder
#[derive(Debug)]
pub struct Cell{
    pub frame : Frame,
    pub modifyers : Vec<(String, DataType)>
}

/// a enum that represents common rust types.  
/// is used so one HashMap can store all types of peramiters and it is up to the user to know what type of data they expext will
/// just return default data value if they try and get something that cant be cast back and forth e.g. trying to get an f32 from a string
/// # Plz Help
/** may add parsing of sting to float or something to that vain in the futre just didnt thing about it at the time
    may add a byte[] DataType value in the futer but that is outsize my experiance and also seems like it could open up exploits casting raw bytes all over the place
*/
#[derive(Debug,Clone)]
pub enum DataType {
    F32(f32),
    String(String),
    Usize(usize),
    Isize(isize),
    Bool(bool),
}

///A now apparently catch all enum for passing around node identification
///# Plz Help
/**origenly this was how I told the animator if I the node pointed to a diffrent node or was the last node and was now giving a Frame as output.
it has now devolved into a sudo catch all for passing node states around E.g. can set as a string or usize depending on how you know what node you are going to next,
I beleve usize is faster because you just index direcly into the node trees node vector, where as the string needs to go thru the node trees node name hashmap first
it also holds error messages and return frames.
should this be public almost sertinly not; why is it? because i would need to make a public enum maybe two to take its place and i dont want to break my code not knowing
exactly how and where i would want to expose things, e.g. if i impl Into<NodeResult> for &str and usize can i leave it privet and let users pass in &str and usize thure
a fn(node : impl Into<NodeResult>) or does node result need to be public for that to work?
*/
#[derive(Debug,Clone)]
pub enum NodeResult{
    Null,///This what is set when an object is iniulized with a noderesult
    NodeID(usize),
    NodeName(String),
    Frame(Frame),
    Error(String),
    Test{message : String, next : String}
}

/**
The trait used to say that a struct can be run as a node in a node tree
*/
pub trait NodeCore : Send + Sync{
    ///returns the name of this node
    fn get_name(&self) -> &str;
    ///is called by an animator that provide its state the node then modifys the state and sets a NodeResult for the next step
    fn run(&self, animation_state : &mut AnimationState);
    ///prints a node basicly just a debug output to console
    fn print(&self);
}
/**
The trait used to say that a struct can be built in a node by a node tree
*/
pub trait NodeBuild : Send + Sync{
    ///is called by a node tree at the beging of a frame
    fn build(self : Box<Self>, world : &WorldCell) -> Box<dyn NodeCore>;
}