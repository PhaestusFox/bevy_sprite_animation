use bevy::reflect::Reflect;
use bevy::reflect::ReflectDeserialize;
use bevy::reflect::ReflectSerialize;
use crate::error::BevySpriteAnimationError as Error;

use crate::node_core::CanLoad;
use crate::prelude::*;

#[derive(serde::Serialize, serde::Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub struct FPSNode {
    name: String,
    fps: u32,
    then: NodeId<'static>,
}

#[cfg(feature = "bevy-inspector-egui")]
impl bevy_inspector_egui::Inspectable for FPSNode {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, _options: Self::Attributes, _context: &mut bevy_inspector_egui::Context) -> bool {
        let mut edit = false;
        ui.collapsing("FPSNode", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name: ");
                if ui.text_edit_singleline(&mut self.name).changed() {edit = true;};
            });
        });
        edit
    }
}

impl std::hash::Hash for FPSNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.fps.hash(state);
        self.then.hash(state);
    }
}

impl FPSNode {
    pub fn new(name: &str, fps: u32, next: impl Into<NodeId<'static>>) -> FPSNode{
        FPSNode{
            name: name.to_string(),
            fps,
            then: next.into(),
        }
    }

    fn frame_time(&self) -> f32 {
        1. / self.fps as f32
    }
}

#[cfg(feature = "serialize")]
impl CanLoad for FPSNode {
    fn loader() -> Box<dyn NodeLoader> {
        Box::new(FPSNodeLoader)
    }
}
impl AnimationNodeTrait for FPSNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, state: &mut AnimationState) -> NodeResult {
        let delta = state.get_attribute::<f32>(&Attribute::DELTA);
        let rem_time = state.try_get_attribute_or_error::<f32>(&Attribute::TIME_ON_FRAME).unwrap_or(0.);
        let time = delta + rem_time;
        let frames = (time / self.frame_time()).floor();
        let rem_time = time - self.frame_time() * frames;
        state.set_attribute(Attribute::FRAMES, frames as usize);
        state.set_attribute(Attribute::TIME_ON_FRAME, rem_time);
        state.set_attribute(Attribute::LAST_FPS, self.frame_time());
        NodeResult::Next(self.then.to_static())
    }

    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, context: &mut bevy_inspector_egui::Context) -> bool{
        bevy_inspector_egui::Inspectable::ui(self, ui, (), context)
    }

    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, _asset_server: &bevy::prelude::AssetServer) -> Result<(), Error>
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

    fn node_type(&self) -> String {
        "FPSNode".to_string()
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
pub use loader::FPSNodeLoader;

#[cfg(feature = "serialize")]
mod loader {
    use crate::node_core::NodeLoader;

    use super::FPSNode;

    pub struct FPSNodeLoader;

    impl NodeLoader for FPSNodeLoader {
        fn load(&mut self, data: &str, _asset_server: &bevy::prelude::AssetServer) -> Result<Box<dyn crate::prelude::AnimationNodeTrait>, crate::error::BevySpriteAnimationError> {
        let node: FPSNode = ron::from_str(data)?;
        Ok(Box::new(node))
    }

    fn can_load(&self) -> &[&str] {
        &["FPSNode"]
    }
    }
}