use bevy::prelude::*;
use node_core::NodeLoader;
use crate::error::BevySpriteAnimationError as Error;
use std::{collections::HashMap, marker::PhantomData};
use crate::prelude::*;

mod error;

pub mod prelude;

pub mod attributes;
pub mod node_core;
pub mod nodes;
pub mod state;

pub struct AnimationPlugin<Flag>{
    marker: PhantomData<Flag>
}

impl<F: 'static + Send + Sync> Default for AnimationPlugin<F> {
    fn default() -> AnimationPlugin<F>{
        AnimationPlugin { marker: PhantomData::default() }
    }
}

impl<F:'static + Send + Sync + Component> Plugin for AnimationPlugin<F> {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationNodes::<F>::default());
        app.add_plugin(bevy_inspector_egui::InspectorPlugin::<AnimationNodes<F>>::default());
        app.add_system(animation_system::<F>.label("AnimationUpdate"));
        app.add_system(state::update_delta::<F>.before("AnimationUpdate"));
        app.add_system_to_stage(CoreStage::First, state::clear_changed);
        app.add_system_to_stage(CoreStage::Last, state::clear_unchanged_temp);
    }
}

#[derive(Component)]
pub struct StartNode(node_core::NodeID);

impl StartNode {
    pub fn from_str(name: &str) -> StartNode {
        StartNode(name.into())
    }
    pub fn from_u64(id: u64) -> StartNode {
        StartNode(NodeID::from_u64(id))
    }
    pub fn from_hex(hex: &str) -> Result<StartNode, std::num::ParseIntError> {
        let hex = if hex.to_lowercase().starts_with("0x") {
            u64::from_str_radix(&hex[2..], 16)?
        } else {u64::from_str_radix(hex, 16)?};
        Ok(StartNode(NodeID::from_u64(hex)))
    }
}

pub struct AnimationNodes<F> {
    nodes: HashMap<node_core::NodeID, Box<dyn node_core::AnimationNode>>,
    #[cfg(feature = "serialize")]
    loaders: HashMap<String, Box<dyn NodeLoader>>,
    marker: PhantomData<F>,
}

impl<F> Default for AnimationNodes<F> {
    fn default() -> AnimationNodes<F> {
        AnimationNodes {
            nodes: HashMap::new(),
            #[cfg(feature = "serialize")]
            loaders: default_loaders(),
            marker: PhantomData::default()
        }
    }
}

#[cfg(feature = "serialize")]
fn default_loaders() -> HashMap<String, Box<dyn NodeLoader>> {
    let mut map: HashMap<String, Box<dyn NodeLoader>> = HashMap::new();
    map.insert("IndexNode".to_string(),IndexNode::loader());
    map.insert("FPSNode".to_string(), FPSNode::loader());
    map
}

impl<F> bevy_inspector_egui::Inspectable for AnimationNodes<F> {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _: Self::Attributes, context: &mut bevy_inspector_egui::Context) -> bool {
        for (name, node) in self.nodes.iter_mut() {
        ui.vertical(|ui| {
            ui.label(name.to_string());
            node.ui(ui, context);
        });}
        true
    }
}

impl<F> AnimationNodes<F> {
    pub fn get_node(&self, id: impl Into<node_core::NodeID>) -> Result<&Box<dyn node_core::AnimationNode>, Error> {
        let id = id.into();
        match self.nodes.get(&id) {
            Some(n) => {Ok(n)},
            None => {Err(Error::NodeNotFound(id))}
        }
    }

    #[inline]
    pub fn add_node(&mut self, node: Box<dyn node_core::AnimationNode>) -> NodeID {
        let id = node.id();
        self.insert_node(id.clone(), node);
        id
    }

    #[inline]
    fn insert_node(&mut self, id: NodeID, node: Box<dyn AnimationNode>) {
        self.nodes.insert(id, node);
    }

    #[cfg(feature = "serialize")]
    pub fn serialize(&self, asset_server: &AssetServer) -> Result<String, Error> {
        let mut data = String::new();
        data.push('[');
        data.push('\n');
        for (id, node) in self.nodes.iter() {
            data.push('\t');
            data.push_str(&format!("NodeID(\"{:#020X}\")", id.as_u64()));
            data.push(':');
            node.serialize(&mut data, asset_server)?;
        }
        data.push(']');
        Ok(data)
    }

    #[cfg(feature = "serialize")]
    pub fn registor_node(&mut self, loader: Box<dyn NodeLoader>) {
        if loader.can_load().len() != 1 {
            todo!("Change this so that AnimationNodes has a map of node_type -> Loader so one loader can load more then one type of node and share sate")
        }
        let can_load = loader.can_load()[0];
        info!("registoring {} loader", can_load);
        if self.loaders.contains_key(can_load) {warn!("A loader for {} was alreadey registored", can_load)};
        self.loaders.insert(can_load.into(), loader);
        //this does nothing for now but my become a memory leak in the futer if i make loader extentions point to a shaired loader;
        //this would allow a single loader to share a state between multiple nodes of diffrent types being loaded but my allow a loader
        //to have no type left relying on it because the are all now registored lesswere this becomes an implmentaion issue tho
    }

    #[cfg(feature = "serialize")]
    pub fn load<P: Into<std::path::PathBuf>>(&mut self, path: P, asset_server: &AssetServer) -> Result<(), Error>{
        use std::fs;
        use std::path::PathBuf;
        let path: PathBuf = path.into();
        let tree = if let Some(ext) = path.as_path().extension() {
            let t = ext == "nodetree";
            if !(ext == "node" || t) {
                return Err(Error::InvalidExtension(ext.to_str().unwrap().to_string()));
            }
            t
        } else {false};

        let data = fs::read_to_string(path.as_path())?;

        if tree {
            info!("loaded {:?} from {:?}",self.load_tree_from_str(&data, asset_server)?,path);
        } else {
            info!("loaded {} from {:?}",self.load_node_from_str(&data, asset_server)?,path);
        }
        Ok(())
    }

    pub fn load_node_from_str(&mut self, data: &str, asset_server: &AssetServer) -> Result<NodeID, Error> {
        let (id, node) = self.load_node(data, asset_server)?;
        self.insert_node(id, node);
        Ok(id)
    }

    #[cfg(feature = "serialize")]
    pub fn load_node(&mut self, data: &str, asset_server: &AssetServer) -> Result<(NodeID, Box<dyn AnimationNode>), Error> {
        let data: &str = data.trim();
        
        let node_id: Option<NodeID> = if data.trim().starts_with("NodeID(\"") {
            let end = data.find(')').ok_or(Error::MalformedStr { message: format!("Failed to find ')' "), location: here!() })? + 1;
            //info!("data = {}", &data[..end]);
            Some(ron::from_str(&data[..end])?)
        } else {
            None
        };

        let loader = if node_id.is_some() {data.find(':').ok_or(Error::MalformedStr{
            message: format!("Failed to find NodeID : Node seperator"),
            location: here!()
        })? + 1} else {0};
        
        let start: usize = if let Some(i) = data[loader..].find('(') {i + loader} else { return Err(Error::MalformedStr{
            message: format!("Failed to Find Oppening ( in str"),
            location: here!(),
        }
        )};
        let loader = self.loaders.get_mut(&data[loader..start]).ok_or(Error::NoLoader(data[loader..start].to_string()))?;
        let node = loader.load(&data[start..], asset_server)?;
        let node_id = if node_id.is_some() {node_id.unwrap()} else {node.id()};
        Ok((node_id, node))
    }

    #[cfg(feature = "serialize")]
    pub fn load_tree_from_str(&mut self, data: &str, asset_server: &AssetServer) -> Result<Vec<NodeID>, Error> {
        let data = data.trim();
        let data = if data.starts_with('[') {data[1..].trim()} else {data};
        let mut nodes = Vec::new();
        let mut index = 0;
        loop {
        if index >= data.len() {
            break;
        }
        let data = &data[index..];
        if let Some(next) = data.chars().next() {
            if next.is_whitespace() || next == ',' || next == ']' || next == ')' || next == '}' {
                index += 1;
                trace!("skiped {} at begging of node?\n{}", next, here!());
                continue;
            }
        };
        
        let start = if data.trim().starts_with("NodeID(\"") {
            data.find("NodeID(\"").unwrap() + 30
        } else {0};

        let mut open = data[start..].match_indices('(');
        open.next();
        let mut close = data[start..].match_indices(')');
        let end = loop {
            match (open.next(), close.next()) {
                (_, None) => return Err(
                                Error::MalformedStr{
                                    message: format!("Failed to find ) "),
                                    location: here!()
                                }),
                (None, Some((end,_))) => {break end + start;},
                (Some((open,_)), Some((close,_))) => if close < open {break close + start;}
            }
        };
        let data = &data[..end + 1];
        nodes.push(self.load_node(data, asset_server)?);
        index += end + 1;
        }   
        let mut ids = Vec::new();
        for (id, node) in nodes.into_iter() {
            ids.push(id);
            self.insert_node(id, node);
        }
        Ok(ids)
    }
}

fn animation_system<Flag: Component>(
    nodes: Res<AnimationNodes<Flag>>,
    mut query: Query<(&mut state::AnimationState, &mut Handle<Image>, &StartNode), With<Flag>>
){
    for (mut state,mut handle, start) in query.iter_mut() {
        let mut next = NodeResult::Next(start.0.clone());
        trace!("Starting With: {}",start.0);
        loop {
            match next {
                NodeResult::Next(id) => match nodes.get_node(id) {
                    Ok(res) => {match res.run(&mut state) {
                        Ok(r) => {
                            next = r;
                        },
                        Err(e) => {error!("Node({}): {}", id, e); break;}
                    }},
                    Err(e) => {error!("{}",e); break;}
                },
                NodeResult::Done(h) => {*handle = h; break;},
            }
        }
    }
}