use bevy::reflect::Reflect;
use bevy::reflect::ReflectDeserialize;
use bevy::reflect::ReflectSerialize;
use crate::serde::ReflectLoadNode;
use crate::prelude::*;

#[derive(serde::Serialize, serde::Deserialize, Reflect)]
#[reflect(Serialize, Deserialize, LoadNode)]
pub struct FPSNode {
    #[serde(default)]
    id: Option<NodeId<'static>>,
    name: String,
    fps: u32,
    then: NodeId<'static>,
}
impl crate::serde::LoadNode for FPSNode {
    fn load<'b>(s: &str, _load_context: &mut bevy::asset::LoadContext<'b>, _dependencies: &mut Vec<bevy::asset::AssetPath<'static>>) -> Result<AnimationNode, crate::error::LoadError> {
        let node = ron::from_str::<FPSNode>(s)?;
        Ok(AnimationNode::new(node))
    }
}

impl FPSNode {
    pub fn new(name: &str, fps: u32, next: impl Into<NodeId<'static>>) -> FPSNode{
        FPSNode{
            id: None,
            name: name.to_string(),
            fps,
            then: next.into(),
        }
    }

    fn frame_time(&self) -> f32 {
        1. / self.fps as f32
    }
}

impl AnimationNodeTrait for FPSNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, state: &mut AnimationState) -> Result<NodeResult, RunError> {
        let delta = state.attribute::<f32>(&Attribute::Delta);
        let rem_time = state.get_attribute::<f32>(&Attribute::TimeThisFrame).cloned().unwrap_or(0.);
        let time = delta + rem_time;
        let frames = (time / self.frame_time()).floor();
        let rem_time = time - self.frame_time() * frames;
        state.set_attribute(Attribute::Frames, frames as usize);
        state.set_attribute(Attribute::TimeThisFrame, rem_time);
        state.set_attribute(Attribute::LastFPS, self.frame_time());
        Ok(NodeResult::Next(self.then.to_static()))
    }

    fn id(&self) -> NodeId {
        if let Some(id) = &self.id {
            id.to_static()
        } else {
            NodeId::from_name(&self.name)
        }
    }

    fn set_id(&mut self, id: NodeId<'_>) {
        self.id = Some(id.to_static());
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, _: &bevy::prelude::AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name));
        this.dot(out);
        out.push_str(" -> ");
        self.then.dot(out);
        out.push_str(";\n");
    }
}