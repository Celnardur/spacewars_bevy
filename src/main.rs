use bevy::prelude::*;
use bevy::render::texture::{SamplerDescriptor, FilterMode};

const SHIP_ROTATION: f32 = std::f32::consts::PI;
const SHIP_ACCELERATION: f32 = 25.0;
const SPRITE_HEIGHT: f32 = 34.0;
const SPRITE_WIDTH: f32 = 16.0;
const SPRITE_DELAY: f32 = 0.2;

struct Materials {
    ship_material: Handle<ColorMaterial>,
}

struct ShipSprites {
    needle: Handle<TextureAtlas>,
    needle_accelerating: Handle<TextureAtlas>,
    wedge: Handle<TextureAtlas>,
    wedge_accelerating: Handle<TextureAtlas>,
}

struct SpriteTimer(Timer);

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum ShipStage {
    Input,
    Physics,
    Update,
    Collisions,
    Render,
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum MissleStage {
    Update,
    Collisions,
    Render,
}

#[derive(Default)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Default)]
struct Rotation(f32);

#[derive(Default)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Default)]
struct ShipAnimations {
    coasting: Handle<TextureAtlas>,
    accelerating: Handle<TextureAtlas>,
}

enum ShipState {
    Coasting,
    Accelerating,
}

struct Ship;

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let mut load_atlas = |file, width, height, frames| {
        let texture_handle = server.load(file);
        let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(width, height), frames, 1);
        atlases.add(atlas)
    };

    // create texture atlases
    let needle = load_atlas("needle_blur.png", SPRITE_WIDTH, SPRITE_HEIGHT, 1);
    let needle_accelerating = load_atlas("needle_acc_blur.png", SPRITE_WIDTH, SPRITE_HEIGHT, 4);
    let wedge = load_atlas("wedge_blur.png", SPRITE_WIDTH, SPRITE_HEIGHT, 1);
    let wedge_accelerating = load_atlas("wedge_acc_blur.png", SPRITE_WIDTH, SPRITE_HEIGHT, 4);
    commands.insert_resource(ShipSprites {
        needle,
        needle_accelerating,
        wedge,
        wedge_accelerating,
    });

    commands.insert_resource(SpriteTimer(Timer::from_seconds(SPRITE_DELAY, true)));
}

fn sprite_tick(
    time: Res<Time>,
    mut timer: ResMut<SpriteTimer>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}

fn add_ships(mut commands: Commands, ships: Res<ShipSprites>) {
    commands
        .spawn()
        .insert(Ship)
        .insert(Position::default())
        .insert(Rotation::default())
        .insert(Velocity::default())
        .insert(ShipAnimations {
            coasting: ships.needle.clone(),
            accelerating: ships.needle_accelerating.clone(),
        })
        .insert(ShipState::Coasting)
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: ships.needle.clone(),
            ..Default::default()
        });
}

fn movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut ship: Query<(&mut Rotation, &mut Velocity), With<Ship>>,
) {
    for (mut rot, mut vel) in ship.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            rot.0 += SHIP_ROTATION * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::D) {
            rot.0 -= SHIP_ROTATION * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::W) {
            let acc = SHIP_ACCELERATION * time.delta_seconds();
            vel.x += rot.0.cos() * acc;
            vel.y += rot.0.sin() * acc;
        }
    }
}

fn ship_switch_sprite_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut ship: Query<(&mut TextureAtlasSprite, &mut Handle<TextureAtlas>, &ShipAnimations)>,
) {
    for (mut sprite, mut atlas, animations) in ship.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::W) {
            atlas.id = animations.accelerating.id.clone();
            sprite.index = 0;
        }
        if keyboard_input.just_released(KeyCode::W) {
            atlas.id = animations.coasting.id.clone();
            sprite.index = 0;
        }
    }
}

fn update_system(time: Res<Time>, mut obj: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in obj.iter_mut() {
        pos.x += vel.x * time.delta_seconds();
        pos.y += vel.y * time.delta_seconds();
    }
}

fn set_position(mut q: Query<(&Position, &mut Transform)>) {
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(pos.x, pos.y, 0.0);
        transform.scale = Vec3::new(4.0, 4.0, 1.0);
    }
}

fn set_rotation(mut q: Query<(&Rotation, &mut Transform)>) {
    for (rot, mut transform) in q.iter_mut() {
        transform.rotation = Quat::from_rotation_z(rot.0 - (std::f32::consts::PI / 2.0));
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(add_ships.system()))
        .add_system(movement_system.system().label(ShipStage::Input))
        .add_system(ship_switch_sprite_system.system().label(ShipStage::Input))
        .add_system(
            update_system
                .system()
                .label(ShipStage::Update)
                .after(ShipStage::Input),
        )
        .add_system(set_position.system().after(ShipStage::Update))
        .add_system(set_rotation.system().after(ShipStage::Update))
        .add_system(sprite_tick.system())
        .run();
}
