use std::borrow::Cow;

use bevy::prelude::Handle;
use bevy::prelude::Image;
use ron::error::Position;
use thiserror::Error;

use crate::prelude::NodeId;
use crate::prelude::Attribute;

type Location = String;

#[derive(Debug,Error)]
pub enum BevySpriteAnimationError {
    #[error("SerializeError")]
    SerializeError,
    #[error("Filed to Deserialize {node_type}: {message}\n{loc}")]
    DeserializeError{
        node_type: &'static str,
        message: String,
        loc: Location,
        raw: ron::de::SpannedError,
    },
    #[cfg(feature = "ron")]
    #[error("RonError: {0}")]
    RonError(#[from] ron::Error),
    #[cfg(feature = "ron")]
    #[error("RonError: {0}")]
    RonDeError(#[from] ron::de::SpannedError),
    #[error("{0} Not Found")]
    NodeNotFound(NodeId<'static>),
    #[error("{} Not Found", .0.name_or_id())]
    AttributeNotFound(Attribute),
    #[error("a BincodeError orccored")]
    BincodeError(#[from] bincode::Error),
    #[error("Node Error: {0}")]
    NodeError(String),
    #[cfg(feature = "serialize")]
    #[error("Invalid Extension .{0}")]
    InvalidExtension(String),
    #[cfg(feature = "serialize")]
    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[cfg(feature = "serialize")]
    #[error("No Loader Registered for: {0}")]
    NoLoader(String),
    #[cfg(feature = "serialize")]
    #[error("Input Data Is Malformed: {message}\n{location}")]
    MalformedStr{
        message: String,
        location: Location
    },
    #[cfg(feature = "serialize")]
    #[error("No AssetPath found for {:?}: only loaded asses are suported for now",.0)]
    AssetPathNotFound(Handle<Image>),
    #[cfg(feature = "serialize")]
    #[error("Failed to parse int: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[cfg(feature = "serialize")]
    #[error("Failed to parse float: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[cfg(feature = "ron")]
    #[error("Failed to find typeid for: {0};\n you must set a attribute once before a script node can set it")]
    NoTypeId(Attribute),
    #[error("asset IO")]
    AssetIo(#[from]bevy::asset::AssetIoError),
    #[error("bytes to string err")]
    StringErr(#[from]std::string::FromUtf8Error),
    #[cfg(feature = "serialize")]
    #[error("Error Loading Node: {0}")]
    LoadError(#[from] LoadError),


}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("No closing parentheses '{ch}' {file}::{pos}")]
    NoClosing {
        ch: char,
        file: Cow<'static, str>,
        pos: Position
    },
    #[error("The Loader RwLock is Poisened")]
    RwLockPoisoned,
    #[error("The Input data has ended")]
    Eof,
    #[error("Node Trees must start '[' and end ']'")]
    NotTree,
    #[error("No extension given; this is probabley a bug with Bevy")]
    NoExtension,
    #[error("Extension can be converted to str")]
    ExtensionNotOsString,
    #[error("Wrong extension given; this is probabley a bug with Bevy")]
    WrongExtension,
    #[error("Missing Char: expected {ch} at {pos}")]
    MissingChar{ch: char, pos: Position},
    #[error("Error Reading Id with Ron: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("Channel Not Working")]
    ChannelError,
    #[error("Type '{0}' not in AppTypeRegistry; use app.register_type::<T>()")]
    NotRegistered(String),
    #[error("Type '{0}' dose not have LoadNode in AppTypeRegistry; impl LoadNode for T then use #[reflect(LoadNode)]")]
    NoLoadRegistered(String)
}

impl LoadError {
    pub fn add_offset(self, offset: Position) -> Self {
        match self {
            LoadError::NoClosing {ch, file,mut pos} => {
                pos.line += offset.line;
                pos.col += offset.col;
                LoadError::NoClosing {ch, file, pos }
            },
            LoadError::MissingChar { ch, mut pos } => {
                pos.line += offset.line;
                pos.col += offset.col;
                LoadError::MissingChar { ch, pos }
            },
            LoadError::Ron(mut e) => {
                e.position.line += offset.line;
                e.position.col += offset.col;
                LoadError::Ron(e)
            }
            e => e
        }
    }
}

#[macro_export]
macro_rules! here {
    () => {
        format!("{}:{}:{}",file!(), line!(), column!())
    };
}