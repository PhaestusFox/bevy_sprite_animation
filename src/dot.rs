use bevy::{prelude::*, asset::HandleId};
use crate::prelude::*;

use std::{
    io::Write,
    process::{Command, Stdio},
};

pub struct KeyChain([KeyCode; 3]);
impl Default for KeyChain {
    fn default() -> Self {
        KeyChain([KeyCode::E; 3])
    }
}

impl KeyChain {
    fn test(&self) -> bool {
        self.0 == [KeyCode::D, KeyCode::O, KeyCode::T]
    }
    fn add(&mut self, key: KeyCode) {
        if key == KeyCode::NumpadAdd {return;}
        self.0[0] = self.0[1];
        self.0[1] = self.0[2];
        self.0[2] = key;
    }
}

pub fn write_dot(
    input: Res<Input<KeyCode>>,
    mut local: Local<KeyChain>,
    nodes: Res<Assets<AnimationNode>>,
    roots: Query<(Entity, &StartNode)>,
    asset_server: Res<AssetServer>,
) {
    for key in input.get_just_pressed() {local.add(*key);};
    if !input.just_pressed(KeyCode::NumpadAdd) || !local.test() {return;}
    println!("run dot");
    let mut string = String::from("digraph graphname {\n");
    for (root, node) in &roots {
        string.push_str(&format!("e_{} -> ", root.index()));
        node.dot(&mut string);
        string.push(';');
        string.push('\n');
    }
    for (id, node) in nodes.iter() {
        let id = handle_to_node(id);
        node.dot(id, &mut string, &asset_server);
    }
    string.push('}');
    open_dot(&string, "AnimationNodeTree").unwrap()
}

fn execute_dot(dot: &str, format: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut child = Command::new("dot")
        .arg("-T")
        .arg(format)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child.stdin.as_mut().unwrap().write_all(dot.as_bytes())?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    Ok(output.stdout)
}

fn open_dot(dot: &str, path: &str) -> Result<(), std::io::Error> {
    let format = "svg";
    let rendered = execute_dot(dot, format)?;
    let path = std::env::temp_dir().join(path).with_extension(format);
    std::fs::write(&path, rendered)?;
    opener::open(path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}