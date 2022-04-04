# bevy_sprite_animation
A simple 2d sprite animation plugin for the bevy game engine;

anyone is welcome to make suggestion and corrections to this repositry: esspecialy my spelling :S

this is more or less a copy of **[Aarthificial's Reanimator](https://github.com/aarthificial/reanimation)** for unity but for bevy of course.

<br>

*subject to change with feedback*
## Usage
Add `AnimationPlugin<T>` and other systems to app
```rust
fn main() {
    App::build()
        .add_plugin(AnimationPlugin::<MainAnimation>::default())
        .add_startup_system(add_nodes.system())
        .add_startup_system(add_animator.system())
        .add_system(update_animator)
        .add_system(read_animator)
}
```
Add `NodeCore`s &| `NodeBuild`s to `Res<NodeTree<T>>`
```rust
fn add_nodes(
    asset_server : Res<AssetServer>,
    mut texture_atlas : ResMut<Assets<TextureAtlas>>,
    mut node_tree : ResMut<NodeTree<MainAnimation>>,
) {
    //add a node created in this system
    //hardcoded like this
    node_tree.add(
        Box::new(
            nodes::SwitchNode{
                name : "gun_switch".to_string(),
                driver : "gun".to_string(),
                true_node : NodeResult::NodeName("gun_node".to_string()),
                false_node : NodeResult::NodeName("idle_node".to_string()),
                fallback_node : NodeResult::NodeName("idle_node".to_string()),
                modifyers : vec![],
         }));
    //or using loaded assets like this
    let texture_handle = asset_server.load("test.png");
    let texture_atles = TextureAtlas::from_grid(texture_handle.clone(), Vec2::splat(60.0), 3, 3);
    let texture_atles_handle = texture_atlas.add(texture_atles);
    node_tree.add(
        Box::new(
            nodes::BasicNode{
                name : "gun_node".to_string(),
                driver : "gun_draw".to_string(),
                cells : vec![
                    Cell{
                        frame : Frame{
                            index : 3,
                            sprite_sheet : texture_atles_handle.clone(),
                            ..Default::default()},
                        modifyers : vec![("gun_draw".to_string(), DataType::Usize(1))]}, //makes it select the next cell next frame
                           //would be added automaticly by a builder with auto_inc^
                    Cell{
                        frame : Frame{
                            index : 4,
                            sprite_sheet : texture_atles_handle.clone(),
                            ..Default::default()},
                        modifyers : vec![("gun_draw".to_string(), DataType::Usize(2))]},
                    Cell{
                        frame : Frame{
                            index : 5,
                            sprite_sheet : texture_atles_handle.clone(),
                            ..Default::default()},
                        modifyers : vec![("gun_draw".to_string(), DataType::Usize(3))]},
                    Cell{
                        frame : Frame{
                            index : 6,
                            sprite_sheet : texture_atles_handle.clone(),
                            ..Default::default()},
                        modifyers : vec![("gun_draw".to_string(), DataType::Usize(4))]},
                    Cell{
                        frame : Frame{
                            index : 7,
                            sprite_sheet : texture_atles_handle.clone(),
                            ..Default::default()},
                        modifyers : vec![("gun_draw".to_string(), DataType::Usize(5)),]},
                    Cell{
                        frame : Frame{
                            index : 7,
                            sprite_sheet : texture_atles_handle.clone(),
                            ..Default::default()},
                        modifyers : vec![
                            ("gun_draw".to_string(), DataType::Usize(0)),
                            ("gun_shoot".to_string(), DataType::Bool(true)), //sets "gun_shoot" so logic else where can run
                            ("gun".to_string(), DataType::Bool(false)),]}], //sets the gun false after it shoots
                modifyers : vec![],
    }));
    
    //add a node builder that will be made at the beging of the next frame like this
    node_tree.add_build(Box::new(nodes::SpriteSheetNodeBuilder{
        name : "idle_node".to_string(),
        driver : "idle_loop".to_string(),
        path : "test.png".to_string(),
        tile_size : Vec2::splat(60.0),
        sprite_sheet_size : (3,3),//(colums, rows)
        start_index : 0,
        end_index : 2,
        auto_inc : true,
        cell_mods : HashMap::new(),
        node_mods : Vec::new()
    }));
}
```
Create an entity with an `Animatior` on it that uses `NodeTree<T>` to pick its next frame
```rust
fn add_animator(
    mut commands: Commands,
) {
    commands.spawn_bundle(SpriteSheetBundle::default())
    .insert(Animator::new(
            vec!["gun_draw".to_string(),// is set temp so the shoot animation always starts from the beggining; if it was interupted half way.
                 "gun_shoot".to_string()],//is set temp so the gun will shoot once without the programer needing to 'set' the "gun_shoot" to false allowing for non mut access
            Frame::default(), //put the frame the the animator would display in an error situation could be a big red "ERROR" for debuging or just the fist frame of the idel animation *is subject to change how this works with feedback*
            "gun_switch",
            10,
        )
    ).insert(MainAnimation);
}
```
Change the state of the `Animatior` to controle what frame is picked next update
```rust
fn update_animator(
    mut animatiors : Query<&mut Animator>,
    input : Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space){
    for mut animatior in animatiors.iter(){
      animatior.set("gun",true)
    }}
}
```
get a parameter from an `Animatior` to create logic that happens only on special frames
```rust
fn read_animator(
    animatiors : Query<(Entity, &mut Animator)>,
) {
    for (entity, animatior) in animatiors.iter(){
      if animatior.get::<bool>("gun_shoot") {
        println!("{} is on a frame where they would shoot", entity);
      }
    }
}
```
