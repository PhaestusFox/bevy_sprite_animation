use crate::serde::ReflectLoadNode;
use crate::serde::LoadNode;
use crate::prelude::*;
use bevy::reflect::Reflect;
use serde::{Serialize, Deserialize, Deserializer};
use crate::error::LoadError;

#[derive(Debug, Serialize, Deserialize, Reflect)]
#[reflect(LoadNode)]
pub struct ScaleNode{
    #[serde(default)]
    id: Option<NodeId<'static>>,
    name: String,
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
            id: None,
            name: name.to_string(),
            scale,
            next
        }
    }
}

impl AnimationNodeTrait for ScaleNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, state: &mut AnimationState) -> Result<NodeResult, RunError> {
        let rem_time = state.attribute::<f32>(&Attribute::TimeThisFrame);
        let frames = *state.attribute::<usize>(&Attribute::Frames);
        let last = state.attribute::<f32>(&Attribute::LastFPS);
        let scale = state.get_attribute::<f32>(&self.scale).cloned().unwrap_or(1.);
        let mut frame_time = last * frames as f32 + rem_time;
        let width = last * scale;
        let frames = (frame_time / width).floor();
        frame_time -= frames * width;

        state.set_attribute(Attribute::LastFPS, last * scale);
        state.set_attribute(Attribute::TimeThisFrame, frame_time);
        state.set_attribute(Attribute::Frames, frames as usize);
        Ok(NodeResult::Next(self.next.to_static()))
    }

    fn set_id(&mut self, id: NodeId<'_>) {
        self.id = Some(id.to_static())
    }

    fn id(&self) -> NodeId {
        if let Some(id) = &self.id {
            id.to_static()
        } else {
            NodeId::from_name(&self.name)
        }
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, _: &bevy::prelude::AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name));
        this.dot(out);
        out.push_str(" -> ");
        self.next.dot(out);
        out.push_str(&format!("[label=\"{}\"];\n", self.scale));
    }
}

impl LoadNode for ScaleNode {
    fn load<'b>(s: &str, _: &mut bevy::asset::LoadContext<'b>, _: &mut Vec<bevy::asset::AssetPath<'static>>) -> Result<AnimationNode, crate::error::LoadError> {
        let mut node = ron::de::Deserializer::from_str(s)?;
        match node.deserialize_struct("ScaleNode", &[], ScaleLoader) {
            Ok(ok) => Ok(AnimationNode::new(ok)),
            Err(e) => Err(LoadError::Ron(ron::de::SpannedError{code: e, position: ron::de::Position{line: 0, col: 0}})),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Fileds {
    Name,
    Scale,
    Next,
}

struct ScaleLoader;

impl<'de> serde::de::Visitor<'de> for ScaleLoader {
    type Value = ScaleNode;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Ron String or a IndexNode")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        use serde::de::Error;
            let mut name = None;
            let mut scale = None;
            let mut next = None;
        while let Some(key) = map.next_key::<Fileds>()? {
            match key {
                Fileds::Name => name = Some(map.next_value::<String>()?),
                Fileds::Scale => scale = Some(map.next_value::<Attribute>()?),
                Fileds::Next => next = Some(map.next_value::<NodeId>()?),
            }
        }
        let Some(scale) = scale else {return Err(Error::missing_field("Scale"));};
        let Some(name) = name else {return Err(Error::missing_field("Name"));};
        let Some(next) = next else {return Err(Error::missing_field("Next"));};
        Ok(ScaleNode {
            id: None,
            name,
            scale,
            next
        })
    }
}