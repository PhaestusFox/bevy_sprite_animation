use bevy::{prelude::*, asset::AssetLoader};
use crate::error::LoadError;
use crate::error::BevySpriteAnimationError as Error;
use crate::prelude::*;
struct NodeLoader(AppTypeRegistry);

impl AssetLoader for NodeLoader {
    fn extensions(&self) -> &[&str] {
        &["node", "nodetree"]
    }
    fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            load_context: &'a mut bevy::asset::LoadContext,
        ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
            Box::pin(async move {
                match load_context.path().extension().ok_or(Error::NoExtension)?.to_str().ok_or(Error::ExtensionNotOsString)? {
                "node" => load_node(&self.0, bytes, load_context).await,
                "nodetree" => load_tree(&self.0, bytes, load_context).await,
                }
            }
        )
    }
}

async fn load_node<'a, 'b: 'a>(type_registry: &AppTypeRegistry, bytes: &'a [u8], load_context: &'a mut bevy::asset::LoadContext<'b>) -> Result<(), bevy::asset::Error> {
    let data = String::from_utf8_lossy(bytes);
    let clean = data.trim();
    let mut line = 0;
    let mut column = 0;
    let type_registry = type_registry.read();
    let mut id = None;
    if clean.starts_with("NodeId(") {
        let len = clean.find(')').ok_or(LoadError::NoClosingParentheses { file: load_context.path().to_string_lossy(), line, column })?;
        let val = clean[7..=len].parse::<u64>()?;
        id = Some(NodeId::U64(val));
    } else if clean.starts_with("NodeName(") {
        
    }

    //type_registry.get_with_name(type_name);
    Ok(())
}

async fn load_tree<'a, 'b>(type_registry: &AppTypeRegistry, bytes: &'a [u8], load_context: &'a mut bevy::asset::LoadContext<'b>) -> Result<(), bevy::asset::Error> {
    Ok(())
}