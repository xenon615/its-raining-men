
use bevy::prelude::*;
use crate::{
    animator::{AllAnimations, AnimationKey, CurrentAnimation}, 
    camera_target::{CameraTarget, SetCameraTarget}, 
    env::{History, Humidity, Seat}, 
    movement::{MovementPath, MovementPathDone}, music::MusicEvent, GameState
};

use crate::shared::random_pos;

// ---

#[derive(Component)]
pub struct Girl;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GirlsState {
    #[default]
    Idle,
    Moving
}

// ---

pub struct  GirlsPlugin;
impl Plugin for GirlsPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<GirlsState>()
        .add_systems(Startup, spawn)
        .add_systems(OnEnter(GameState::Intro), enter_walking)
        .add_systems(OnEnter(GirlsState::Idle), enter_idle)
        .add_systems(OnEnter(GameState::Portal), enter_running)
        
        .add_systems(Update, move_done.run_if(on_event::<MovementPathDone>()))
        .add_systems(OnEnter(GameState::Sing), enter_sing)
        .add_systems(OnEnter(GameState::Raining), enter_raining)
        .add_event::<MovementPathDone>()
        .observe(music_event)
        ;
    }
} 

const GIRLS_COUNT : usize = 64;
pub const GIRL_VELOCITY: f32 = 2.;

// ---

fn spawn (
    mut commands: Commands,
    assets: ResMut<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut all_animations: ResMut<AllAnimations>,

) {
    all_animations.add(AnimationKey::Girl, "models/girls/girl.glb", 13, &mut graphs, &assets);
    let gh = assets.load("models/girls/girl.glb#Scene0");
    let far = 60.;
    for bp  in [ Vec3::new(0., 0., far), Vec3::new(0., 0., -far), Vec3::new(far, 0., 0.), Vec3::new(-far, 0.,0.)] {
        for _j in 0 .. GIRLS_COUNT / 4 {
            commands.spawn((
                SceneBundle {
                    scene: gh.clone(),
                    transform: Transform::from_translation(random_pos(bp, 10.)),
                    ..default()
                },
                Girl,
                AnimationKey::Girl
            ));
        }        
    }
}

// ---

fn enter_walking(
    mut girls_q: Query<(&mut Transform, Entity), With<Girl>>,
    seats_q: Query<&GlobalTransform, (With<Seat>, Without<Girl>)>,
    mut ccc: Commands,
    mut next : ResMut<NextState<GirlsState>>
) {
    let mut seats: Vec<(Vec3, Dir3)> = seats_q.iter().map(|t| {
        (t.translation(), t.forward())
    }).collect();
    
    for (t, entity) in girls_q.iter_mut() {

        let idx = seats
            .iter()
            .enumerate().min_by(|(_ ,a), (_, b)|  a.0.distance_squared(t.translation).total_cmp(&b.0.distance_squared(t.translation)))
            .map(|(idx, _)| idx)
            .unwrap();
        let seat  = seats.swap_remove(idx);
        let velocity =  fastrand::f32() +  GIRL_VELOCITY;
        let points = vec![
            (seat.0 - seat.1 * 4.0, velocity * 2., Some(1), None),
            (seat.0, velocity, Some(5), None)
        ];
        ccc.entity(entity).insert(
            MovementPath::new(&points, fastrand::i32(9..=10) as usize, points[0].0)
        );
    }
    next.set(GirlsState::Moving);
}

// ---

fn enter_idle(
    state: Res<State<GameState>>,
    mut girls_q: Query<&mut CurrentAnimation, With<Girl>>,
) {
    let idxs = [4, 6];
    if *state.get() == GameState::Speak {

        for mut ca in girls_q.iter_mut() {
            ca.0 = idxs[fastrand::usize(0 ..idxs.len())];
        }
    }
}

// ----

fn enter_running(
    mut girls_q: Query<(&Transform, Entity), With<Girl>>,
    mut cc: Commands,
    mut next : ResMut<NextState<GirlsState>>
) {

    let step = 360.0 / GIRLS_COUNT as f32; 
    let idxs = [2,3,7,8];
    for (idx, (t, entity)) in girls_q.iter_mut().enumerate() {
        let radius = 6. * if idx % 2 == 0 {1.} else {0.75};

        let delta  = t.forward() *  (1.0  +  fastrand::f32());
        let velocity =  fastrand::f32() +  GIRL_VELOCITY;
        let points = vec![
            (t.translation + delta, velocity * 2., Some(1), None),
            (-delta * 4.0, velocity * 2., None, None),
            (Vec3::new((radius - fastrand::f32() * 1.0 ) * (idx as f32 * step).cos(), 0.0, radius * (idx  as f32 * step).sin()), velocity * 2. , None, None)
        ];
        
        cc.entity(entity).insert(
            MovementPath::new(&points,idxs[fastrand::usize(0 .. idxs.len())], Vec3::ZERO)
        );
    }
    next.set(GirlsState::Moving);
}

// ---

fn move_done(
    mut ev_r : EventReader<MovementPathDone>,
    girls_q: Query<Entity, With<Girl>>,
    check_q: Query<Entity, (With<Girl>, With<MovementPath>)>,
    mut next: ResMut<NextState<GirlsState>>
) {
    for e in ev_r.read() {
        if girls_q.get(e.0).is_ok() {
            if check_q.is_empty() {
                next.set(GirlsState::Idle);
            }
        }
    }
}

// ---

fn enter_sing (
    mut girls_q: Query<&mut CurrentAnimation, With<Girl>>,
) {
    girls_q.iter_mut().for_each(|mut ca| ca.0 = 6);
}

// ---

fn enter_raining(
    mut girls_q: Query<&mut CurrentAnimation, With<Girl>>,
) {
    girls_q.iter_mut().for_each(|mut ca| ca.0 = 4);
}

//  ---

fn music_event (
    trigger: Trigger<MusicEvent>,
    mut cmd: Commands,
    h_q: Query<Entity, With<Humidity>>,
    hi_q: Query<Entity, With<History>>
) {
    let MusicEvent(GameState::Sing, count) = *trigger.event() else  {
        return;
    };
    if count == 1 {
        let he = h_q.get_single().unwrap();
        cmd.trigger(SetCameraTarget(
            CameraTarget::from_entity(he).with_translation_bias(Vec3::new(0.,0. , 2.)),
            1
        ));        
    } else if count == 2 {
        cmd.trigger(SetCameraTarget(
            CameraTarget::from_position(Vec3::new(60., 10., 0.)).with_direction(-Dir3::X), 0)
        );

    } else if count == 3  {
        cmd.trigger(SetCameraTarget(
            CameraTarget::from_position(Vec3::new(0., 2., -5.)).with_direction(Dir3::Z), 0)
        );
    } else if count == 4  {
        cmd.trigger(SetCameraTarget(
            CameraTarget::from_position(Vec3::new(-50., 40., 0.)).with_direction(-Dir3::X), 0)
        );
    } else if count == 5  {
        let hie = hi_q.get_single().unwrap();
        cmd.trigger(SetCameraTarget(
            CameraTarget::from_entity(hie)
            .with_translation_bias(Vec3::new(0., 3., 5.))
            .with_rotation_bias(Vec3::new(0., 3., 0.))
            ,
            2
        ));
    }
}
