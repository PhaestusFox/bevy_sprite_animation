use bevy::prelude::SystemLabel;

#[derive(Debug, Clone, Copy, std::hash::Hash, PartialEq, Eq)]
#[derive(SystemLabel)]
pub enum AnimationLabel {
    PreUpdate,
    Update,
    PostUpdate,
}