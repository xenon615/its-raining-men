use bevy::prelude::* ;
use crate::GameState;


pub struct LiftPlugin;
impl Plugin for LiftPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn)
        .add_systems(Update, rotate.run_if(in_state(ScrewState::Moving)))
        .add_systems(OnEnter(GameState::Portal), enter_portal)
        .register_type::<Screw>()
        .init_state::<ScrewState>()
        ;
    }
}

// --

#[derive(Component)]
pub struct Lift;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Screw;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum ScrewState {
    #[default]
    Idle,
    Moving
}


// --

fn spawn(
    mut commands: Commands, 
    assets: ResMut<AssetServer>
) {
    let lh = assets.load("models/other/lift.glb#Scene0");
    let dist = 40.;
    for (x, z) in [(1., 1.), (1., -1.), (-1., 1.), (-1., -1.)] {
        commands.spawn((
            SceneBundle {
                scene: lh.clone(),
                transform: Transform::from_xyz(x * dist, 0., z * dist).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            Lift
        ));
    }
}

// ---

fn rotate (
    mut screw_q : Query<&mut Transform, With<Screw>>,
    time: Res<Time>
) {

    for mut t in &mut screw_q {
        t.rotate_y(time.delta_seconds() * 5.0);        
    }

}

// ---

fn enter_portal(
    mut next: ResMut<NextState<ScrewState>>
) {
    next.set(ScrewState::Moving);
}