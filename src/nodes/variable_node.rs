use crate::error::LoadError;
use crate::prelude::*;
use crate::serde::LoadNode;
use crate::serde::ReflectLoadNode;
use bevy::asset::AssetPath;
use bevy::prelude::Handle;
use bevy::prelude::Image;
use bevy::reflect::Reflect;
use serde::Deserializer;

#[derive(Debug, Reflect)]
#[reflect(LoadNode)]
pub struct VariableNode {
    id: Option<NodeId<'static>>,
    name: String,
    frames: Vec<(Handle<Image>, f32)>,
    is_loop: bool,
    index: Attribute,
}

impl VariableNode {
    #[inline(always)]
    pub fn new(name: &str, frames: &[(Handle<Image>, f32)], is_loop: bool) -> VariableNode {
        VariableNode {
            id: None,
            name: name.to_string(),
            frames: frames.to_vec(),
            is_loop,
            index: Attribute::IndexId(0),
        }
    }

    #[inline(always)]
    pub fn new_with_index(
        name: &str,
        frames: &[(Handle<Image>, f32)],
        is_loop: bool,
        index: Attribute,
    ) -> VariableNode {
        VariableNode {
            id: None,
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

    fn id(&self) -> NodeId {
        if let Some(id) = &self.id {
            id.to_static()
        } else {
            NodeId::from_name(&self.name)
        }
    }

    fn set_id(&mut self, id: NodeId<'_>) {
        self.id = Some(id.to_static())
    }

    #[cfg(feature = "dot")]
    fn dot(&self, this: NodeId<'_>, out: &mut String, asset_server: &bevy::prelude::AssetServer) {
        this.dot(out);
        out.push_str(&format!(" [label=\"{}\"];\n", self.name));
        for (i, (index, len)) in self.frames.iter().enumerate() {
            this.dot(out);
            out.push_str(" -> ");
            let h = handle_to_node(index.id());
            h.dot(out);
            out.push_str(&format!(" [label=\"({}, {})\"];\n", i, len));
            if let Some(path) = asset_server.get_handle_path(index) {
                h.dot(out);
                out.push_str(&format!(" [label={:?}];\n", path.path()));
                h.dot(out);
                out.push_str(" [color=green];\n");
            }
        }
    }
}

impl LoadNode for VariableNode {
    fn load<'b>(
        s: &str,
        load_context: &mut bevy::asset::LoadContext<'b>,
        dependencies: &mut Vec<AssetPath<'static>>,
    ) -> Result<AnimationNode, crate::error::LoadError> {
        let mut node = ron::de::Deserializer::from_str(s)?;
        match node.deserialize_struct("IndexNode", &[], VariableLoader(load_context, dependencies))
        {
            Ok(ok) => Ok(AnimationNode::new(ok)),
            Err(e) => Err(LoadError::Ron(ron::de::SpannedError {
                code: e,
                position: ron::de::Position { line: 0, col: 0 },
            })),
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

struct VariableLoader<'de, 'b: 'de>(
    &'de mut bevy::asset::LoadContext<'b>,
    &'de mut Vec<AssetPath<'static>>,
);

impl<'de, 'b: 'de> serde::de::Visitor<'de> for VariableLoader<'de, 'b> {
    type Value = VariableNode;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Ron String or a IndexNode")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
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
            id: None,
            frames: handles,
            name,
            is_loop,
            index,
        })
    }
}
