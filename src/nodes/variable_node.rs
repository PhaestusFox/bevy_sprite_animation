use crate::error::LoadError;
use crate::prelude::*;
use crate::serde::LoadNode;
use crate::serde::ReflectLoadNode;
use bevy::asset::AssetPath;
use bevy::prelude::Handle;
use bevy::prelude::Image;
use bevy::reflect::Reflect;
use serde::Deserializer;
use crate::error::BevySpriteAnimationError as Error;

#[derive(Debug, Reflect)]
#[reflect(LoadNode)]
pub struct VariableNode{
    name: String,
    frames: Vec<(Handle<Image>, f32)>,
    is_loop: bool,
    index: Attribute,
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

impl VariableNode {
    #[inline(always)]
    pub fn new(name: &str, frames: &[(Handle<Image>, f32)], is_loop: bool) -> VariableNode {
        VariableNode { 
            name: name.to_string(),
            frames: frames.to_vec(),
            is_loop,
            index: Attribute::IndexId(0),
        }
    }

    #[inline(always)]
    pub fn new_with_index(name: &str, frames: &[(Handle<Image>, f32)], is_loop: bool, index: Attribute) -> VariableNode {
        VariableNode { 
            name: name.to_string(),
            frames: frames.to_vec(),
            is_loop,
            index,
        }
    }
}

impl AnimationNodeTrait for VariableNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn node_type(&self) -> String {
        "VariableNode".to_string()
    }

    fn run(&self, state: &mut AnimationState) -> Result<NodeResult, RunError> {
        assert!(self.frames.len() != 0);
        let mut index = state.index(&self.index);
        let rem_time = state.attribute::<f32>(&Attribute::TimeThisFrame);
        let frames = *state.attribute::<usize>(&Attribute::Frames);
        let mut frame_time = state.attribute::<f32>(&Attribute::LastFPS) * frames as f32 + rem_time;
        let mut current: &(Handle<Image>, f32) = &self.frames[index % self.frames.len()];
        while frame_time > current.1 {
            frame_time -= current.1;
            index += 1;
            if index >= self.frames.len() {
                if self.is_loop {
                    index %= self.frames.len();
                } else {
                    index = self.frames.len() - 1;
                }
            }
            current = &self.frames[index];
        }
        state.set_attribute(Attribute::TimeThisFrame, frame_time);
        state.set_attribute(self.index.clone(), index);
        Ok(NodeResult::Done(current.0.clone()))
    }

    #[cfg(feature = "bevy-inspector-egui")]
    fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, context: &mut bevy_inspector_egui::Context) -> bool{
        bevy_inspector_egui::Inspectable::ui(self, ui, (), context)
    }

    #[cfg(feature = "serialize")]
    fn serialize(&self, data: &mut String, asset_server: &bevy::prelude::AssetServer) -> Result<(), Error>
    {
        data.push_str("VariableNode(\n\t");
        data.push_str("name: \"");
        data.push_str(&self.name);
        data.push_str("\",\n\tframes: [\n\t");
        for (frame, time) in self.frames.iter() {
            data.push_str("(");
            if let Some(path) = asset_server.get_handle_path(frame) {
                data.push_str(path.path().to_str().unwrap())
            } else {
                return Err(Error::AssetPathNotFound(frame.clone_weak()));
            }
            data.push_str(&format!(", {})", time));
            data.push_str(",\n\t");
        }
        data.push_str("],\n\t");
        data.push_str(&format!("is_loop: {},\n\t",self.is_loop));
        data.push_str("index: ");
        data.push_str(&ron::to_string(&self.index)?);
        data.push_str(",\n\t),\n");
        Ok(())
    }

    fn id(&self) -> NodeId {
        NodeId::from_name(&self.name)
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl LoadNode for VariableNode {
    fn load<'b>(s: &str, load_context: &mut bevy::asset::LoadContext<'b>, dependencies: &mut Vec<AssetPath<'static>>) -> Result<AnimationNode, crate::error::LoadError> {
        let mut node = ron::de::Deserializer::from_str(s)?;
        match node.deserialize_struct("IndexNode", &[], VariableLoader(load_context, dependencies)) {
            Ok(ok) => Ok(AnimationNode::new(ok)),
            Err(e) => Err(LoadError::Ron(ron::de::SpannedError{code: e, position: ron::de::Position{line: 0, col: 0}})),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Fileds {
    Name,
    Frames,
    IsLoop,
    Index,
}

struct VariableLoader<'de, 'b: 'de>(&'de mut bevy::asset::LoadContext<'b>, &'de mut Vec<AssetPath<'static>>);

impl<'de, 'b: 'de> serde::de::Visitor<'de> for VariableLoader<'de, 'b> {
    type Value = VariableNode;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Ron String or a IndexNode")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        use serde::de::Error;
            let mut name = None;
            let mut frames = None;
            let mut is_loop = false;
            let mut index = Attribute::IndexId(0);
        while let Some(key) = map.next_key::<Fileds>()? {
            match key {
                Fileds::Name => name = Some(map.next_value::<String>()?),
                Fileds::Frames => frames = Some(map.next_value::<Vec<(String, f32)>>()?),
                Fileds::IsLoop => is_loop = map.next_value::<bool>()?,
                Fileds::Index => index = map.next_value::<Attribute>()?,
            }
        }
        let Some(frames) = frames else {return Err(Error::missing_field("Frames"));};
        let Some(name) = name else {return Err(Error::missing_field("Name"));};
        let mut handles = Vec::with_capacity(frames.len());
        for frame in frames {
            handles.push((self.0.get_handle::<_, Image>(&frame.0), frame.1));
            self.1.push(frame.0.into());
        }
        Ok(VariableNode {
            frames: handles,
            name,
            is_loop,
            index
        })
    }
}