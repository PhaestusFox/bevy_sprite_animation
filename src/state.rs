use crate::error::StateError;

use super::prelude::*;
use super::error::BevySpriteAnimationError as Error;
use bevy::{utils::{HashMap, HashSet}, reflect::TypeData};
use serde::{Serialize, de::DeserializeOwned};

use bevy::prelude::*;
use thiserror::Error;

use std::any::{TypeId, Any};

pub trait AnimationStateObj: Any + Send + Sync + Reflect {
    fn get_registration(&self) -> bevy::reflect::TypeRegistration;
}

impl<T: Any + Send + Sync + Reflect + bevy::reflect::GetTypeRegistration> AnimationStateObj for T  {
    fn get_registration(&self) -> bevy::reflect::TypeRegistration {
        T::get_type_registration()
    }
}

#[derive(Component)]
pub struct AnimationState {
    data: HashMap<Attribute, Box<dyn AnimationStateObj>>,
    pub(crate) changed: HashSet<Attribute>,
    pub(crate) temp: HashSet<Attribute>,
}

impl std::fmt::Debug for AnimationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationState")
        // .field("data", &self.data)
        .field("changed", &self.changed)
        .field("temp", &self.temp)
        .finish()
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        let mut data: HashMap<Attribute, Box<dyn AnimationStateObj>> = HashMap::default();
        data.insert(Attribute::Delta,  Box::new(0.0f32));
        data.insert(Attribute::Frames, Box::new(0));
        data.insert(Attribute::FlipX, Box::new(false));
        data.insert(Attribute::FlipY, Box::new(false));
        let s = Self { data, changed: HashSet::new(), temp: HashSet::new()};
        s
    }
}

impl AnimationState {
    /// will return `D` for attribute panics if the attribute is not set
    /// or `D` is the wrong type
    /// use try_get_attribute() if you are unsure if the attribute exists
    #[inline(always)]
    pub fn attribute<D: 'static>(&self, key: &Attribute) -> &D {
        self.get_attribute(key).expect(&format!("get Attribute {} failed", key))
    }

    /// will return an `Result<D, StateError>`
    /// # Errors 
    /// * WrongType - the type used to get is not the same as the one used to set
    /// * NotFound - there is no data set for the Attribute
    #[inline(always)]
    pub fn get_attribute<D: 'static>(&self, key: &Attribute) -> Result<&D, StateError> {
        if let Some(data) = self.data.get(key) {
            data.as_any().downcast_ref::<D>().ok_or(StateError::WrongType)
        } else {
            Err(StateError::NotFound)
        }
    }

    /// sets an Attribute to a specific type and val
    pub fn set_attribute<D: AnimationStateObj>(&mut self, key: Attribute, val: D) {
        self.change(key.clone());
        self.data.insert(key, Box::new(val));
    }

    /// Will stop this Attribute being cleared after a frame it is not set
    pub fn set_persistent(&mut self, temp: &Attribute) -> bool {
        self.temp.remove(temp)
    }

    /// Will clear this Attribute if it is not set each frame
    pub fn set_temporary(&mut self, temp: Attribute) -> bool {
        self.temp.insert(temp)
    }

    /// retrun a bool based on if the Attribute has changed this frame
    pub fn changed(&self, attribute: &Attribute) -> bool {
        self.changed.contains(attribute)
    }

    #[inline]
    fn change(&mut self, attribute: Attribute) {
        self.changed.insert(attribute);
    }

    /// removes the data from an Attribute and forgets its type
    pub fn clear_attribute(&mut self, attribute: &Attribute) {
        self.data.remove(attribute);
    }

    /// get the usize for an index panics if given something other then Index or IndexId
    /// return 0 if index does not exist or is wrong type
    pub fn index(&self, index: &Attribute) -> usize {
        self.get_index(index).expect("Attribute to be Index or IndexId")
    }

    /// try get the usize for an index return None if given something other then Index or IndexId
    /// return 0 if index does not exist or is wrong type
    pub fn get_index(&self, index: &Attribute) -> Option<usize> {
        if !index.is_index() {return None;}
        Some(self.get_attribute::<usize>(index).cloned().unwrap_or_default())
    }

    #[cfg(feature = "ron")]
    pub(crate) fn set_from_ron(&mut self, attribute: &Attribute, s: &str) -> Result<(), StateError> {
        let Some(main) = self.data.get_mut(attribute) else {return Err(StateError::NotFound);};
        let data = main.get_registration();
        let Some(data) = data.data::<ReflectDeserialize>() else {return Err(StateError::NotRegistered(data.type_name()));};
        let mut deserializer = ron::Deserializer::from_str(s).expect("Ron To Work");
        let val = data.deserialize(&mut deserializer)?;
        main.set(val).expect("Same Type");
        Ok(())
    }

}

pub(crate) fn update_delta(
    time: Res<Time>,
    mut states: Query<&mut AnimationState>,
){
    for mut state in states.iter_mut() {
        state.set_attribute(Attribute::Delta, time.delta_seconds());
    }
}

pub(crate) fn clear_unchanged_temp(
    mut states: Query<&mut AnimationState>,
) {
    for mut state in states.iter_mut() {
        let state = state.as_mut();
        let mut to_clear = Vec::with_capacity(state.temp.len());
        for temp in state.temp.iter() {
            if !state.changed(temp) {
                to_clear.push(temp.clone());
            }
        }
        for clear in to_clear.iter() {
            state.clear_attribute(clear)
        }
    }
}

pub(crate) fn clear_changed(
    mut states: Query<&mut AnimationState>
) {
    for mut state in states.iter_mut() {
        state.changed.clear();
    }
}

pub(crate) fn flip_update(
    mut sprites: Query<(&AnimationState, &mut Sprite)>,
){
    for (state, mut sprite) in sprites.iter_mut() {
        sprite.flip_x = *state.attribute(&Attribute::FlipX);
        sprite.flip_y = *state.attribute(&Attribute::FlipY);
    }
}