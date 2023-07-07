use bevy::prelude::Handle;
use bevy::prelude::Image;
use thiserror::Error;

use crate::prelude::NodeID;
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
    },
    #[cfg(feature = "ron")]
    #[error("RonError: {0}")]
    RonError(#[from] ron::Error),
    #[cfg(feature = "ron")]
    #[error("RonError: {0}")]
    RonDeError(#[from] ron::de::SpannedError),
    #[error("{0} Not Found")]
    NodeNotFound(NodeID),
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
}

#[macro_export]
macro_rules! here {
    () => {
        format!("{}:{}:{}",file!(), line!(), column!())
    };
}