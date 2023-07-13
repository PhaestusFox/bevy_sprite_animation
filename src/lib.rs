use bevy::prelude::*;
use node_core::NodeLoader;
use node_core::CanLoad;
use crate::error::BevySpriteAnimationError as Error;
use crate::error::LoadError;
use std::collections::HashMap;
use crate::prelude::*;

mod error;

//pub mod serde;

pub mod prelude;

pub mod attributes;
pub mod node_core;
pub mod nodes;
pub mod state;
pub mod system_set;

pub mod node_id;

#[cfg(test)]
mod test{
    pub(crate) fn test_asset_server() -> bevy::asset::AssetServer {
        use bevy::core::TaskPoolOptions;
        TaskPoolOptions::default().create_default_pools();
        bevy::asset::AssetServer::new(bevy::asset::FileAssetIo::new("assets", &None))
    }
}

pub struct SpriteAnimationPlugin;

impl Plugin for SpriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<AnimationNode>();
        app.insert_resource(AnimationNodeTree::default());
        app.add_systems(Update, animation_system.in_set(AnimationSet::Update));
        app.add_systems(Update, state::update_delta.before(AnimationSet::Update).in_set(AnimationSet::PreUpdate));
        app.add_systems(First, state::clear_changed);
        app.add_systems(PostUpdate, state::flip_update.in_set(AnimationSet::PostUpdate));
        app.add_systems(Last, state::clear_unchanged_temp);
        #[cfg(feature = "bevy-inspector-egui")]
        bevy_inspector_egui::RegisterInspectable::register_inspectable::<StartNode>(app);
    }
}

#[derive(bevy::reflect::TypeUuid, bevy::reflect::TypePath)]
#[uuid="b30eb8be-06db-4d7c-922d-22767a539ad6"]
pub struct AnimationNode(pub Box<dyn AnimationNodeTrait>);
impl AnimationNode {
    pub fn new(node: impl AnimationNodeTrait) -> AnimationNode {
        AnimationNode(Box::new(node))
    }
}

impl<'a> AnimationNodeTrait for AnimationNode {
    fn run(&self, state: &mut crate::state::AnimationState) -> NodeResult {
        self.0.run(state)
    }
    fn hash(&self) -> u64 {
        self.0.hash()
    }
    fn id(&self) -> NodeId<'_> {
        self.0.id()
    }
    fn name(&self) -> &str {
        self.0.name()
    }
    fn node_type(&self) -> String {
        self.0.node_type()
    }
    fn serialize(&self, data: &mut String, asset_server: &AssetServer) -> Result<(), Error> {
        self.0.serialize(data, asset_server)
    }
}

#[derive(Component)]
pub struct StartNode(pub NodeId<'static>);

#[cfg(feature = "bevy-inspector-egui")]
impl bevy_inspector_egui::Inspectable for StartNode {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _options: Self::Attributes, _context: &mut bevy_inspector_egui::Context) -> bool {
        let mut edit = false;
        ui.horizontal(|ui|{
            let mut name = self.0.name_or_id();
            ui.label("Start Node: ");
            if ui.text_edit_singleline(&mut name).changed() {
                self.0 = NodeId::from_str(&name);
                edit = true;
            }
        });
        edit
    }
}

impl std::str::FromStr for StartNode {
    type Err = <NodeId<'static> as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StartNode(NodeId::from_str(s)?))
    }
}

impl StartNode {
    pub fn from_u64(id: u64) -> StartNode {
        StartNode(NodeId::U64(id))
    }
    pub fn from_name(name: impl Into<std::borrow::Cow<'static, str>>) -> StartNode {
        StartNode(NodeId::Name(name.into()))
    }
}

#[derive(Resource)]
pub struct AnimationNodeTree {
    nodes: Vec<Handle<AnimationNode>>,
    nodes: HashSet<Handle<AnimationNode>>,
    #[cfg(feature = "serialize")]
    loaders: std::sync::Arc<std::sync::RwLock<HashMap<String, Box<dyn NodeLoader>>>>,
}

impl Default for AnimationNodeTree {
    fn default() -> AnimationNodeTree {
        AnimationNodeTree {
            nodes: HashSet::new(),
            #[cfg(feature = "serialize")]
            loaders: default_loaders(),
        }
    }
}

#[cfg(feature = "serialize")]
fn default_loaders() -> std::sync::Arc<std::sync::RwLock<HashMap<String, Box<dyn NodeLoader>>>> {
    let mut map: HashMap<String, Box<dyn NodeLoader>> = HashMap::new();
    map.insert("IndexNode".to_string(),IndexNode::loader());
    map.insert("FPSNode".to_string(), FPSNode::loader());
    map.insert("ScriptNode".to_string(), ScriptNode::loader());
    map.insert("ScaleNode".to_string(), ScaleNode::loader());
    std::sync::Arc::new(std::sync::RwLock::new(map))
}

impl AnimationNodeTree {

    pub fn add_node(&mut self, node: Handle<AnimationNode>) {
        self.nodes.insert(node);
    } 

    // #[cfg(feature = "serialize")]
    // pub fn serialize(&self, asset_server: &AssetServer) -> Result<String, Error> {
    //     let mut data = String::new();
    //     data.push('[');
    //     data.push('\n');
    //     for (id, node) in self.nodes.iter() {
    //         data.push('\t');
    //         data.push_str(&format!("{}", id));
    //         data.push(':');
    //         node.serialize(&mut data, asset_server)?;
    //     }
    //     data.push(']');
    //     Ok(data)
    // }

    #[cfg(feature = "serialize")]
    pub fn registor_node<T: CanLoad>(&mut self) {
        let loader = T::loader();
        if loader.can_load().len() != 1 {
            todo!("Change this so that AnimationNodes has a map of node_type -> Loader so one loader can load more then one type of node and share sate")
        }
        let can_load = loader.can_load()[0];
        info!("registoring {} loader", can_load);
        let Ok(mut loaders) = self.loaders.write() else {error!("Failed to add loader {:?}", LoadError::RwLockPoisoned); return;};
        if loaders.contains_key(can_load) {warn!("A loader for {} was alreadey registored", can_load)};
        loaders.insert(can_load.into(), loader);
        //this does nothing for now but my become a memory leak in the futer if i make loader extentions point to a shaired loader;
        //this would allow a single loader to share a state between multiple nodes of diffrent types being loaded but my allow a loader
        //to have no type left relying on it because the are all now registored lesswere this becomes an implmentaion issue tho


    }

    #[cfg(feature = "serialize")]
    pub fn load<P: AsRef<std::path::Path>>(&mut self, path: P, asset_server: &AssetServer, assets: &mut Assets<AnimationNode>) -> Result<(), Error>{
        let io = asset_server.asset_io().load_path(path.as_ref());
        
        let tree = if let Some(ext) = path.as_ref().extension() {
            let t = ext == "nodetree";
            if !(ext == "node" || t) {
                return Err(Error::InvalidExtension(ext.to_str().unwrap().to_string()));
            }
            t
        } else {false};
        let data = String::from_utf8(futures_lite::future::block_on(io)?)?;

        if tree {
            let nodes = self.load_tree_from_str(&data, asset_server, assets)?;
            info!("loaded {} from {:?}",nodes.len(), path.as_ref());
            for node in nodes {
                self.add_node(node)
            }
        } else {
            let node = self.load_node_from_str(&data, asset_server, assets)?;
            info!("loaded 1 from {:?}", path.as_ref());
            self.add_node(node);
        }
        Ok(())
    }

    #[cfg(feature = "serialize")]
    pub fn load_node_from_str(&mut self, data: &str, asset_server: &AssetServer, assets: &mut Assets<AnimationNode>) -> Result<Handle<AnimationNode>, Error> {
        let (id, node) = self.load_node(data, asset_server)?;
        Ok(assets.set(id, AnimationNode(node)))
    }

    #[cfg(feature = "serialize")]
    pub fn load_node(&mut self, data: &str, asset_server: &AssetServer) -> Result<(NodeId, Box<dyn AnimationNodeTrait>), Error> {
        let mut input = InputIter::new(data);
        let mut sym = String::new();
        let mut node_id = None;
        while let Some(char) = input.next() {
            if char.is_whitespace() {
                sym.clear();
            } else {
                sym.push(char);
            }
            if char == '(' {
                if sym.starts_with("Id(") || sym.starts_with("Name(") {
                    ext_to(&mut sym, &mut input, ')')?;
                    println!("Node Id Data: {:?}", &sym);
                    node_id = Some(ron::from_str(&sym).unwrap());
                    ext_to(&mut sym, &mut input, ':')?;
                    sym.clear();
                } else {
                    sym.pop();
                    break;
                }
            }
        }

        let loaders = self.loaders.read().or(Err(LoadError::RwLockPoisoned))?;
        let loader = loaders.get(&sym).ok_or(Error::NoLoader(sym))?;
        
        let node = match loader.load(&data[input.next_index-1..], asset_server) {
            Ok(ok) => ok,
            Err(e) => return Err(match e {
                Error::RonDeError(mut e) => {e.position.line += input.line;
                e.position.col += input.col;
                Error::RonDeError(e)},
                Error::DeserializeError { node_type, message, loc, mut raw } => {
                    raw.position.line += input.line;
                    raw.position.col += input.col;
                    Error::DeserializeError { node_type, message, loc, raw }
                },
                e => {e}
            }),
        };
        let node_id = if node_id.is_some() {node_id.unwrap()} else {node.id().to_static()};
        Ok((node_id, node))
    }

    #[cfg(feature = "serialize")]
    pub fn load_tree_from_str(&mut self, data: &str, asset_server: &AssetServer, assets: &mut Assets<AnimationNode>) -> Result<Vec<Handle<AnimationNode>>, Error> {
        let mut input = InputIter::new(data);
        let mut sym = String::new();
        let mut nodes = Vec::new();
        let mut has_started = false;
        let mut id = None;
        while let Some(char) = input.next() {
            if char.is_whitespace() {
                sym.clear();
            } else if !has_started && char.is_ascii_punctuation() {
                continue;
            } else {
                has_started = true;
                sym.push(char);
            }
            if char == '(' {
                if sym.starts_with("Id(") || sym.starts_with("Name(") {
                    ext_to(&mut sym, &mut input, ')')?;
                    id = Some(ron::from_str::<NodeId>(&sym).map_err(|mut e| {
                        e.position.line += input.line;
                        e.position.col += input.col;
                        e
                    })?);
                    ext_to(&mut sym, &mut input, ':')?;
                    sym.clear();
                } else {
                    ext_till_close(&mut sym, &mut input, '(', ')')?;
                    println!("Found Node With Data: {}", sym);
                    let node = self.load_node(&sym, asset_server)?;
                    if let Some(id) = id {
                        nodes.push((id, node.1));
                    } else {
                        nodes.push((node.0.to_static(), node.1));
                    }
                    sym.clear();
                    has_started = false;
                    id = None;
                }
            }
        }
        let mut ids = Vec::new();
        for (id, node) in nodes.into_iter() {
            ids.push(assets.set(id, AnimationNode(node)));
        }
        Ok(ids)
    }
}

fn ext_to(
    word: &mut String,
    chars: &mut InputIter,
    target: char,
) -> Result<(), ron::de::SpannedError> {
    while let Some(char) = chars.next() {
        word.push(char);
        if char == target {
            return Ok(());
        }
    }
    Err(ron::de::SpannedError{code: ron::Error::Eof, position: chars.file_position()})
}

fn ext_till_close(
    word: &mut String,
    chars: &mut InputIter,
    open: char,
    close: char,
) -> Result<(), ron::de::SpannedError>{
    let mut depth = 1;
    while let Some(char) = chars.next() {
        word.push(char);
        if char == close {
            depth -= 1;
        } else if char == open {
            depth += 1;
        }
        if depth == 0 {
            return Ok(());
        }
    }
    Err(ron::de::SpannedError{code: ron::Error::Eof, position: chars.file_position()})
}

struct InputIter<'a>{
    input: std::str::Chars<'a>,
    next_index: usize,
    line: usize,
    col: usize
}

impl<'a> InputIter<'a> {
    fn new(s: &'a str) -> InputIter {
        Self{
            
            input: s.chars(),
            next_index: 0,
            line: 0,
            col: 0,
        }
    }

    fn file_position(&self) -> ron::de::Position {
        ron::de::Position{line: self.line + 1, col: self.col + 1}
    }
}

impl Iterator for InputIter<'_> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.input.next() {
            self.next_index += 1;
            self.col += 1;
            if next == '\n' {
                self.col = 0;
                self.line += 1;
            }
            Some(next)
        } else {
            None
        }
    }
}

fn animation_system(
    nodes: Res<Assets<AnimationNode>>,
    mut query: Query<(&mut state::AnimationState, &mut Handle<Image>, &StartNode)>
){
    for (mut state,mut handle, start) in query.iter_mut() {
        let mut next = NodeResult::Next(start.0.clone());
        trace!("Starting With: {:?}", start.0);
        loop {
            match next {
                NodeResult::Next(id) => if let Some(node) = nodes.get(&Handle::weak(id.to_static().into())) {
                    trace!("Running Node: {:?}",id);
                    next = node.run(&mut state);
                } else {
                    error!("Node not found: {:?}", id);
                    break;
                },
                NodeResult::Error(e) => {error!("{}",e); break;}
                NodeResult::Done(h) => {*handle = h; break;},
            }
        }
    }
}