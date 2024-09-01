use bevy::{prelude::*, window::{
    // WindowResolution, 
    WindowMode
}};
use bevy_gltf_components::ComponentsFromGltfPlugin;
use bevy_registry_export::ExportRegistryPlugin;
use music::MusicEvent;
mod camera;
mod env;
mod girls;
mod camera_target;
mod shared;
mod leader;
mod men;
mod airplane;
mod lift;
mod music;
mod intro;
mod animator;

mod movement;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Intro,
    Thunder,
    Portal,
    Speak,
    Sing,
    Raining,
    TakeOff,
    Finish
}

#[derive(Component)]
pub struct NotReady;

// ---

fn main() {
    App::new()
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins((
        DefaultPlugins.set(
            WindowPlugin {
                primary_window : Some(Window {
                    // resolution : WindowResolution::new(1400., 900.),
                    mode: WindowMode::BorderlessFullscreen,
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            }
        ),
        ComponentsFromGltfPlugin{legacy_mode: false},
        ExportRegistryPlugin::default(),
        camera::CameraPlugin,
        env::EnvPlugin,
        girls::GirlsPlugin,
        leader::LeaderPlugin,
        men::MenPlugin,
        airplane::AirplanePlugin,
        lift::LiftPlugin,
        music::MusicPlugin,
        movement::MovementPlugin,
    ))
    .add_plugins((
        camera_target::CameraTargetPlugin, 
        intro::IntroPlugin,
        animator::AnimatorPlugin
    ))
    .init_state::<GameState>()
    .add_systems(Update, check_ready.run_if(in_state(GameState::Loading)))
    .observe(the_end)
    .run();
}

// ---

fn check_ready(
    mut next: ResMut<NextState<GameState>>,
    not_ready_q: Query<Entity, With<NotReady>>
) {
    if not_ready_q.is_empty() {
        next.set(GameState::Intro);
        // next.set(GameState::Finish);
    }
} 

// --

fn the_end(
    trigger: Trigger<MusicEvent>,
    mut exit: EventWriter<AppExit>
) {
    if  MusicEvent(GameState::Finish, 1) == *trigger.event() {
        exit.send(AppExit::Success);
    }
}