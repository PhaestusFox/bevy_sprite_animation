# bevy_sprite_animation

A simple 2d sprite animation plugin for the Bevy game engine.

Anyone is welcome to make suggestion and corrections to this repository, typographic and otherwise!

This is more or less a copy of **[Aarthificial's Reanimator](https://github.com/aarthificial/reanimation)** for Unity but for Bevy of course.

**[Here](https://youtu.be/6fuo8jm7wlM)** is a video explaining how the example works.
**[Here](https://youtu.be/5UIqGe2P7YU)** is a video going over what is new in 0.4

*subject to change with feedback*

## Version
0.3 = Bevy 0.7 avalable as 0.7 branch<br>
0.3.1 = Bevy 0.8 avalable as 0.8 branch<br>
0.3.1 = Bevy 0.10 avalable as 0.10 branch<br>
0.3.2 = Bevy 0.11 avalable as master branch<br>
0.4 = Bevy 0.11 avalable as v0.4 branch<br>
## Usage

### Add `AnimationPlugin::<usize>` and other systems to app

the usize is the max nodes a single path can take
this is to stop loops from locking up frames indefinitly

```rust
fn main() {
    App::build()
        .add_plugin(AnimationPlugin::<10>)
        .add_startup_system(add_nodes.system())
        .add_startup_system(add_animator.system())
        .add_system(update_animator)
        .add_system(read_animator)
        // registure any custom nodes so thay can be loaded
        // these nodes should reflect LoadNode inorder to work correctly
        .register_type::<MatchNode<ZState>>()
        .run()
}
```

### Add nodes to `Assets<AnimationNode>` resurce

```rust
fn add_nodes(
    asset_server : Res<AssetServer>,
    mut nodes : ResMut<Assets<AnimationNode>>,
) {
    // make some image handles
    let mut handles = Vec::new();
    for i in 0..10 {
        handles.push(asset_server.load(format!("SomeSprite_{}", i));
    }

    // add a node created in this system
    // hardcoded like this
    let node = Box::new(IndexNode::new("New Node", &handles));

    // this will return a handle to the node
    let node_handle = nodes.add_node(node);
    // this converts our handle into a NodeId
    let node_id = NodeId::from_handle(node_handle);

    // this can be used to make common nodes easy to refrence
    // or to make it easy to refrece from a node loaded from a file

    //with a given name
    nodes.set(NodeId::from_name("Node Name"), node);
    //with a given id
    nodes.set(NodeId::from_u64("Node Name"), node);
    
    // load a node from a file
    // this will return a handle to a Refrence Node, this is so the node can have a Handle<AnimationNode> diffrent from its FilePath
    let from_file = asset_server.load("example.node");
    
    // load a node_tree from a file
    // this will return a handle to a Refrence Node, this is also the nodes can have diffrent Handle<AnimationNode> diffrent from its FilePath and so they dont unload if the nodes that my point into the tree where to unload
    node_tree.load("example.nodetree");

    // Refrence Nodes will run the first Node in there list if you call them so it it ok the use them as start nodes, as long as you are using .node or the first node in the file is correct
}
```

### Create an entity with an `AnimationState` and `StartNode`
the start node it used to pick the entry point each frame

```rust
fn add_animator(
    mut commands: Commands,
) {
    // create a default state
    let mut state = AnimationState::default();
    // set starting Attributes
    start.set_attribute(Attribute::FlipX, true);
    // Attributes data can be any time that derives Reflect
    // Attributes can be made from any time that impls  `Into<Cow<'static, str>>`
    // Attributes made this way will dispay this name when they are debugged
    start.set_attribute(Attribute::new_attribute("custom_attribute"), "cat");
    // there is a more relaxed way to get Attributes that only requies it impl Hash
    // Attributes made this way will **Not** dispay a name when they are debugged
    // Atttibute both methodes of getting an attribute are Eq, if T.into::<Cow>().hash() and T.hash()
    // are also Eq
    start.set_attribute(Attribute::new_attribute_id("specil_attribute"), 5);

    // set temporary attribute
    // these will be removed if they are not changed each frame
    // you can also get index attributes, they follow the same rules but can only be used to get and set usize into the AnimationState
    state.set_temporary(Attribute::new_index("Idel"));

    // remove temporary attribute
    // by default all attributes are persistent
    // Index Attributes do not conflict with Custom Attributes
    // Attribute::new_index("Idel") != Attribute::new_attribute("Idel")
    state.set_persistent(Attribute::new_index_id("Idel"));

    // spwan the entity
    commands.spwan((
        // we need this to see it
        SpriteBundle::default(),
        // the state the nodes can use so multiple entitys can use the same nodes and get diffrent results
        state,
        // the first node the entity should run to work out its final sprite
        // this can be from a u64, anything that impls Cow<'_, str>, or a Handel<AnimationNode>
        StartNode::from_u64(0),
    ))
}
```

### Change the state of the `AnimationState` to control what frame is picked next update

```rust
fn update_animation_state(
    mut animatiors : Query<&mut AnimationState>,
    input : Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space){
    for mut animatior in animatiors.iter(){
      start.set_attribute(Attributes::new_attribute("custom_attribute"), "dog");
    }}
}
```

### Get an attribute from an `AnimationState` to create logic that happens only on special frames

```rust
fn read_animation_state(
    animatiors : Query<(Entity, &AnimationState)>,
) {
    for (entity, animatior) in animatiors.iter(){
      if let Ok(ground_type) = animatior.get_attribute::<GroundType>(Attributes::new_attribute("step")) {
        println!("{} is on a frame where you should play the sound of someone stepping on {}", entity, ground_type);
      }
    }
}
```

### Check if an attribute from `AnimationState` changed this frame

```rust
fn read_animation_change(
    animatiors : Query<(Entity, &AnimationState)>,
    dogs: Query<&mut Dogs>,
) {
    for (entity, animatior) in animatiors.iter(){
        // assuming barke is temporary it will only change when set to true.
        // use `changed` for logic where you dont care what the attribute
        if animatior.changed(Attributes::new_attribute("barke")) {
            println!("{} is on a frame where you should play a barke sound effect", entity);
        }
    }

    for (entity, animatior) in animatiors.iter(){
        if animatior.changed(Attributes::new_attribute("dog_breed")) {
            let dog = dogs.get(animatior.get_attribute::<Entity>(Attributes::new_attribute("dog_breed")));
            // do something to the state based on the dog's breed
            println!("{} is on a frame where you should play a barke sound effect", entity);
        }
    }
}
```