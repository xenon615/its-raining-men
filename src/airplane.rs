use bevy::prelude::*;
use crate::GameState;

pub struct AirplanePlugin;
impl Plugin for AirplanePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::Raining), spawn)
        .add_systems(Update, fly.run_if(in_state(GameState::Raining)))
        .enable_state_scoped_entities::<GameState>()
        ;
    }
}

// --

#[derive(Component)]
pub struct AirPlane;

// --

fn spawn(
    mut commands: Commands, 
    assets: ResMut<AssetServer>
) {
    commands.spawn((
        SceneBundle {
            scene: assets.load("models/other/airplane.glb#Scene0"),
            transform: Transform::from_xyz(0., 120., 0.),
            ..default()
        },
        AirPlane,
        StateScoped(GameState::Raining)
    ));
}

// ---

fn fly (
    mut jet_q : Query<&mut Transform, With<AirPlane>>,
    time: Res<Time>
) {
    if let Ok(mut t) = jet_q.get_single_mut() {
        let m = t.forward() * time.delta_seconds() * 20.;
        t.translation += m;
    }
}