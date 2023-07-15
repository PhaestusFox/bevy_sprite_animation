use bevy::prelude::*;
use crate::error::BevySpriteAnimationError as Error;
use std::fmt::Debug;
use crate::prelude::*;

pub(crate) mod utils {
    use std::hash::Hasher;

    pub fn get_hasher() -> bevy::utils::AHasher {
        use std::hash::BuildHasher;
        bevy::utils::RandomState::with_seeds(42, 23, 13, 8).build_hasher()
    }

    pub fn get_hash<T: std::hash::Hash>(name: &T) -> u64 {
        let mut hasher = get_hasher();
        name.hash(&mut hasher);
        hasher.finish()
    }
}

mod error;

pub mod serde;

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

/// The plugin that adds all you need for the Animation sytem
/// The const is the max number of nodes that are to be run per entity per frame
/// This is to stop infinity looping, you should be abel to see this high if you have no nodes that loop
/// This will only report as a warning when the max depth is reached so please dont set it too high if there is a potental to loop
/// start small get bigger, keep it as small as you can whithout rist of breaking early
pub struct SpriteAnimationPlugin<const MAXDEPTH: usize>;

impl<const MAX: usize> Plugin for SpriteAnimationPlugin<MAX> {
    fn build(&self, app: &mut App) {
        app.add_asset::<AnimationNode>();
        #[cfg(feature = "serialize")]
        app.add_plugins(crate::serde::AnimationNodeSerdePlugin);
        app.add_systems(First, state::clear_changed);
        app.add_systems(Update, state::update_delta.in_set(AnimationSet::PreUpdate));
        app.add_systems(Update, animation_system::<MAX>.in_set(AnimationSet::Update));
        app.add_systems(Update, state::flip_update.in_set(AnimationSet::PostUpdate));
        app.add_systems(Last, state::clear_unchanged_temp);
        app.configure_sets(Update, (AnimationSet::PreUpdate, AnimationSet::Update, AnimationSet::PostUpdate).chain());
        nodes::type_registration::registor_nodes(app);
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

    pub fn downcast_ref<T: std::any::Any>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref()
    }
}

impl Debug for AnimationNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.debug(f)
    }
}

impl<'a> AnimationNodeTrait for AnimationNode {
    fn run(&self, state: &mut crate::state::AnimationState) -> Result<NodeResult, RunError> {
        self.0.run(state)
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
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.debug(f)
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
        StartNode(NodeId::from_name(name))
    }
    pub fn from_handle(handle: Handle<AnimationNode>) -> StartNode {
        StartNode(NodeId::Handle(handle))
    }
}

fn animation_system<const MAX: usize>(
    nodes: Res<Assets<AnimationNode>>,
    mut query: Query<(&mut state::AnimationState, &mut Handle<Image>, &StartNode)>
){
    query.par_iter_mut().for_each_mut(|(mut state,mut image, start)| {
        let mut next = NodeResult::Next(start.0.clone());
        trace!("Starting With: {:?}", start.0);
        for _ in 0..MAX {
            match next {
                NodeResult::Next(id) => if let Some(node) = nodes.get(&Handle::weak(id.to_static().into())) {
                    trace!("Running Node: {:?}",id);
                    next = match node.run(&mut state) {
                        Ok(ok) => ok,
                        Err(e) => {error!("{}", e); break;},
                    }
                } else {
                    error!("Node not found: {:?}", id);
                    break;
                },
                NodeResult::Done(h) => {*image = h; break;},
            }
        }
    })
}