use bevy::prelude::SystemSet;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
pub enum AnimationSet {
    PreUpdate,
    Update,
    PostUpdate,
}
