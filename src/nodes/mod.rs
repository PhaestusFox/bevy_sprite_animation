pub use fps_node::FPSNode;
pub use index_node::IndexNode;
pub use match_node::MatchNode;
pub use reference_node::ReferenceNode;
pub use scale_node::ScaleNode;
pub use script_node::ScriptNode;
pub use variable_node::VariableNode;

pub mod fps_node;
pub mod index_node;
pub mod match_node;
pub mod reference_node;
pub mod scale_node;
pub mod script_node;
pub mod variable_node;

pub(crate) mod type_registration {
    use super::*;
    use bevy::prelude::App;
    pub(crate) fn registor_nodes(app: &mut App) {
        app.register_type::<FPSNode>()
            .register_type::<IndexNode>()
            .register_type::<ScriptNode>()
            .register_type::<ScaleNode>()
            .register_type::<VariableNode>();
    }
}
