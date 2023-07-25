use std::borrow::Cow;

use ron::error::Position;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BevySpriteAnimationError {
    #[error("Error Runing Node")]
    RunError(#[from] RunError),
    #[error("Error Interacting With State")]
    StateError(#[from] StateError),
    #[cfg(feature = "serialize")]
    #[error("Error Saving Node To Str")]
    SaveError(#[from] SaveError),
    #[cfg(feature = "serialize")]
    #[error("Error Loading Node From Str")]
    LoadError(#[from] LoadError),
}

#[cfg(feature = "serialize")]
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("No closing parentheses '{ch}' {file}::{pos}")]
    NoClosing {
        ch: char,
        file: Cow<'static, str>,
        pos: Position,
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
    MissingChar { ch: char, pos: Position },
    #[error("Error Reading Id with Ron: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("Channel Not Working")]
    ChannelError,
    #[error("Type '{0}' not in AppTypeRegistry; use app.register_type::<T>()")]
    NotRegistered(String),
    #[error("Type '{0}' dose not have LoadNode in AppTypeRegistry; impl LoadNode for T then use #[reflect(LoadNode)]")]
    NoLoadRegistered(String),
}

impl LoadError {
    pub fn add_offset(self, offset: Position) -> Self {
        match self {
            LoadError::NoClosing { ch, file, mut pos } => {
                pos.line += offset.line;
                pos.col += offset.col;
                LoadError::NoClosing { ch, file, pos }
            }
            LoadError::MissingChar { ch, mut pos } => {
                pos.line += offset.line;
                pos.col += offset.col;
                LoadError::MissingChar { ch, pos }
            }
            LoadError::Ron(mut e) => {
                e.position.line += offset.line;
                e.position.col += offset.col;
                LoadError::Ron(e)
            }
            e => e,
        }
    }
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Attribute not in state")]
    NotFound,
    #[error("Attribute has a diffrent type")]
    WrongType,
    #[cfg(feature = "ron")]
    #[error("Failed To set by ron")]
    SetByRon(#[from] ron::de::Error),
    #[cfg(feature = "ron")]
    #[error("{0} dose not #[reflect(Deserialise)]")]
    NotRegistered(&'static str),
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("{0}")]
    StateError(#[from] StateError),
    #[error("{0}")]
    Custom(String),
}

#[cfg(feature = "serialize")]
#[derive(Debug, Error)]
pub enum SaveError {}
