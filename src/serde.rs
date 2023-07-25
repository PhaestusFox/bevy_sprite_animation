use crate::error::LoadError;
use crate::prelude::*;
use bevy::asset::{AssetPath, LoadedAsset};
use bevy::{asset::AssetLoader, prelude::*};

pub struct AnimationNodeSerdePlugin;

impl Plugin for AnimationNodeSerdePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, add_nodes_to_assets)
            .init_non_send_resource::<NodeWorldChannel>();
    }
}

fn add_nodes_to_assets(
    node_channel: NonSend<NodeWorldChannel>,
    mut nodes: ResMut<Assets<AnimationNode>>,
) {
    for node in node_channel.receiver.try_iter() {
        // the refrence node should have a strong handle
        let _ = nodes.set(node.id().to_static(), node);
    }
}

struct NodeWorldChannel {
    receiver: std::sync::mpsc::Receiver<AnimationNode>,
}

impl FromWorld for NodeWorldChannel {
    fn from_world(world: &mut World) -> Self {
        let (sender, receiver) = std::sync::mpsc::sync_channel(10);
        let reg = world.resource::<AppTypeRegistry>().clone();
        world
            .resource::<AssetServer>()
            .add_loader(BevyNodeLoader(reg, sender));
        NodeWorldChannel { receiver }
    }
}

pub(crate) struct BevyNodeLoader(
    pub AppTypeRegistry,
    pub std::sync::mpsc::SyncSender<AnimationNode>,
);

impl AssetLoader for BevyNodeLoader {
    fn extensions(&self) -> &[&str] {
        &["node", "nodetree"]
    }
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move { load_tree(self, bytes, load_context).await })
    }
}

fn load_node<'a, 'b: 'a>(
    type_registry: &AppTypeRegistry,
    input: &mut InputIter<'a>,
    load_context: &mut bevy::asset::LoadContext<'b>,
    dependencies: &mut Vec<AssetPath<'static>>,
) -> Result<AnimationNode, LoadError> {
    let type_registry = type_registry.read();
    let node_type = input.extract_type()?;
    let data = input.extract_data_min_one_open('(', ')')?;
    let type_registration = if let Some(e) = type_registry.get_with_short_name(&node_type) {
        e
    } else if let Some(e) = type_registry.get_with_name(&node_type) {
        e
    } else {
        return Err(LoadError::NotRegistered(node_type));
    };
    let Some(loader) = type_registration.data::<ReflectLoadNode>() else {return Err(LoadError::NoLoadRegistered(node_type));};
    loader
        .load(&data, load_context, dependencies)
        .map_err(|e| e.add_offset(input.file_position()))
}

async fn load_tree<'a, 'b: 'a>(
    loader: &BevyNodeLoader,
    bytes: &'a [u8],
    load_context: &'a mut bevy::asset::LoadContext<'b>,
) -> Result<(), bevy::asset::Error> {
    let data = String::from_utf8_lossy(bytes);
    let mut dependencies = Vec::new();
    let mut input = InputIter::new(&data);
    input.trim();
    let is_tree = match load_context
        .path()
        .extension()
        .ok_or(LoadError::NoExtension)?
        .to_str()
        .ok_or(LoadError::ExtensionNotOsString)?
    {
        "node" => false,
        "nodetree" => true,
        _ => return Err(LoadError::WrongExtension)?,
    };
    if is_tree
        && '['
            != input.next().ok_or(LoadError::MissingChar {
                ch: '[',
                pos: input.file_position(),
            })?
    {
        return Err(LoadError::MissingChar {
            ch: '[',
            pos: input.file_position(),
        })?;
    }
    let mut reference = crate::nodes::ReferenceNode(Vec::new(), load_context.path().to_path_buf());
    input.trim();
    while input.peek().is_some() {
        let id = input.extract_id()?;
        input.trim();
        let mut node = match load_node(&loader.0, &mut input, load_context, &mut dependencies) {
            Ok(node) => node,
            Err(e) => {
                error!("{}", e);
                let _ = input.extract_till(',');
                input.trim();
                continue;
            }
        };
        let id = if let Some(id) = id {
            node.set_id(id.to_static());
            id
        } else {
            node.id()
        };
        reference.0.push(load_context.get_handle(id));
        loader.1.send(node).or(Err(LoadError::ChannelError))?;
        let _ = input.extract_till(',');
        input.trim();
    }
    load_context.set_default_asset(
        LoadedAsset::new(AnimationNode::new(reference)).with_dependencies(dependencies),
    );
    Ok(())
}

#[derive(Clone)]
pub struct ReflectLoadNode(
    fn(
        &str,
        &mut bevy::asset::LoadContext<'_>,
        &mut Vec<AssetPath<'static>>,
    ) -> Result<AnimationNode, LoadError>,
);

impl ReflectLoadNode {
    fn load(
        &self,
        data: &str,
        ctx: &mut bevy::asset::LoadContext<'_>,
        dependencies: &mut Vec<AssetPath<'static>>,
    ) -> Result<AnimationNode, LoadError> {
        (self.0)(data, ctx, dependencies)
    }
}

impl<T: Reflect + LoadNode> bevy::reflect::FromType<T> for ReflectLoadNode {
    fn from_type() -> Self {
        ReflectLoadNode(T::load)
    }
}

pub trait LoadNode {
    /// The Method used to load the node from a string
    /// # Arguments
    /// * `s` - this is the &str you have to load your node from. it will start `(` and end `)` and total `(` == total `)`
    /// * 'ctx' - This gives you access to the `LoadContext`<br>
    /// **Use to**
    /// * make strong handles
    /// * add labeled sub assets
    /// * add labeled sub nodes<br>
    /// **Dont use to**
    /// * set the default asset
    /// * set your primary node with a lable<br>
    /// **Your primary node must be returned as `Ok((id, node))`**<br>
    /// **Do not add your primary node to the context**
    /// * `dependencies` - this is where you can add `AssetPath` dependencies for you node<br>
    /// use this if you have things like paths to images
    ///
    /// # Examples
    /// ```no_run
    /// fn load<'b>(
    ///     s: &str,
    ///     _load_context: &mut bevy::asset::LoadContext<'b>,
    ///     _dependencies: &mut Vec<bevy::asset::AssetPath<'static>>
    /// ) -> Result<AnimationNode, LoadError> {
    ///     let node = ron::from_str::<FPSNode>(s)?;
    ///     Ok(AnimationNode::new(node))
    /// }
    /// ```
    /// see the [IndexNode] for an example of loading images as part of a node
    fn load<'b>(
        s: &str,
        ctx: &mut bevy::asset::LoadContext<'b>,
        dependencies: &mut Vec<AssetPath<'static>>,
    ) -> Result<AnimationNode, LoadError>;
}

#[derive(Clone)]
pub struct InputIter<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
    next_index: usize,
    line: usize,
    col: usize,
}

impl<'a> InputIter<'a> {
    pub fn new(s: &'a str) -> InputIter {
        Self {
            input: s.chars().peekable(),
            next_index: 0,
            line: 0,
            col: 0,
        }
    }

    pub fn file_position(&self) -> ron::de::Position {
        ron::de::Position {
            line: self.line,
            col: self.col,
        }
    }

    fn extract_till(&mut self, last: char) -> Result<String, LoadError> {
        let mut out = String::new();
        while let Some(char) = self.next() {
            out.push(char);
            if char == last {
                return Ok(out);
            }
        }
        Err(LoadError::Eof)
    }

    fn extract_type(&mut self) -> Result<String, LoadError> {
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if *ch == '(' {
                return Ok(out);
            }
            out.push(*ch);
            self.next();
        }
        Err(LoadError::Eof)
    }

    fn extract_id(&mut self) -> Result<Option<NodeId<'static>>, LoadError> {
        let backup = self.clone();
        self.trim();
        let id_val = self.extract_till(')')?;
        if id_val.starts_with("Id(") || id_val.starts_with("Name(") {
            self.extract_till(':')?;
            match ron::from_str(&id_val) {
                Ok(val) => Ok(Some(val)),
                Err(mut error) => {
                    error.position.line += self.line;
                    error.position.col += self.col;
                    Err(LoadError::Ron(error))
                }
            }
        } else {
            *self = backup;
            return Ok(None);
        }
    }

    pub fn trim(&mut self) {
        while let Some(char) = self.peek() {
            if !char.is_whitespace() {
                return;
            };
            self.next();
        }
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn extract_data_min_one_open(&mut self, open: char, close: char) -> Result<String, LoadError> {
        let mut depth = 0;
        let mut opened = false;
        let mut out = String::new();
        while let Some(ch) = self.next() {
            out.push(ch);
            if ch == close {
                depth -= 1;
            }
            if ch == open {
                depth += 1;
                opened = true;
            }
            if depth == 0 && opened {
                return Ok(out);
            }
        }
        Err(LoadError::Eof)
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
