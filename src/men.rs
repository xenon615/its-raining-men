use bevy::prelude::*;
use crate::{
    animator::{AllAnimations, AnimationKey}, 
    camera::{Cam, CameraState}, 
    camera_target::{CameraTarget, SetCameraTarget}, 
    girls::Girl, lift::Lift, 
    GameState 
};
use crate::shared::random_pos;
use crate::movement:: {MovementPath, MovementPathDone, MovementStart};

// ---

#[derive(Component)]
pub struct Man;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MenState {
    #[default]
    Idle,
    Moving
}

#[derive(Component)]
pub struct Pair(Entity);

#[derive(Component)]
pub struct Paired;
// ---

pub struct  MenPlugin;
impl Plugin for MenPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn)
        .add_systems(OnEnter(CameraState::Single), enter_moving.run_if(in_state(GameState::Raining)))
        .add_systems(Update, cam_rotate.run_if(in_state(MenState::Moving)))
        .add_systems(Update, move_done.run_if(on_event::<MovementPathDone>()))
        .init_state::<MenState>()
        ;
    }
} 

const MEN_COUNT : usize = 64;
const MAN_VELOCITY: f32 = 2.;

// ---

fn spawn (
    mut commands: Commands,
    assets: ResMut<AssetServer>,
    mut all_animations: ResMut<AllAnimations>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    
) {

    all_animations.add(AnimationKey::Man, "models/men/peasant-man.glb", 5, &mut graphs, &assets);
    let gh = assets.load("models/men/peasant-man.glb#Scene0");

    for _j in 0 .. MEN_COUNT {
        commands.spawn((
            SceneBundle {
                scene: gh.clone(),
                transform: Transform::from_translation(random_pos(Vec3::new(0., 50., 0.), 5.)),
                visibility: Visibility::Hidden,
                ..default()
            },
            Man,
            AnimationKey::Man
        ));
    }        
}

// ---

fn enter_moving(
    mut men_q: Query<(&mut Visibility, Entity), With<Man>>,
    girls_q: Query<(&Transform, Entity), (With<Girl>, Without<Man>)>,
    mut ccc: Commands,
    cam_q: Query<Entity, With<Cam>>,
    mut next: ResMut<NextState<MenState>>,
) {
    for ((mut vis , man_entity), (gt, ge)) in men_q.iter_mut().zip(girls_q.iter()) {
        *vis = Visibility::Visible;
        let velocity = fastrand::f32() + MAN_VELOCITY;
        let points = vec![
            (gt.translation - gt.forward() * 10. , velocity * 2., Some(1), None),
            (gt.translation + gt.right() * 1. , velocity * 2. , Some(3), None)
        ];

        ccc.entity(man_entity).insert(
            MovementPath::new(&points, 0, gt.translation)
        );
        ccc.entity(man_entity).insert(Pair(ge));
    }
    if let Ok(cam_e) = cam_q.get_single() {
        ccc.entity(cam_e).insert(CameraTarget::from_position(Vec3::ZERO.with_y(20.)).with_direction(Dir3::Y));
    }
    next.set(MenState::Moving);
}

// ---

fn move_done(
    mut ev_r: EventReader<MovementPathDone>,
    man_q: Query<(Entity, &Pair, Option<&Paired>, &Transform), (With<Man>, Without<Lift>)>,
    girl_q: Query<Entity, With<Girl>>,
    lifts_q: Query<&Transform, (With<Lift>, Without<Man>)>,
    mut next: ResMut<NextState<GameState>>,
    mut cmd: Commands,
    mut first_man_set : Local<bool>,
    mut count: Local<usize>,
    time: Res<Time>
) {
    for e in ev_r.read() {
        if let Ok((entity_m, pair, op, man_t)) = man_q.get(e.0) {
            if op.is_some() {
                continue;
            }
            if ! *first_man_set {
                *first_man_set = true;
                cmd.trigger(SetCameraTarget(
                    CameraTarget::from_entity(entity_m).with_translation_bias(Vec3::new(2., 4., 10.)),
                    0
                ));
                next.set(GameState::TakeOff);
            }

            let closest = lifts_q.iter()
            .map(|t| t.translation)
            .min_by(|a, b| {
                a.distance_squared(man_t.translation).total_cmp(&b.distance_squared(man_t.translation))
            })
            .unwrap()
            ;
            
            let dir = closest.normalize();
            let p0 = 25.5 * dir;

            let p1 = (p0 + 16. * dir).with_y(7.);
            let bias = fastrand::f32() * 2.;
            let p2 = (p1 + 2.0 * dir * (1. + bias)).with_y(6.5);
            let p3 = p2.with_y(200.0);

            let last_dir = Dir3::new((Vec3::ZERO - p0).normalize()).unwrap();

            let points_m = vec![
                (p0, MAN_VELOCITY, Some(4), None),
                (p1, MAN_VELOCITY, None, None),
                (p2, MAN_VELOCITY, None, None),
                (p3, MAN_VELOCITY, Some(1), Some(-last_dir)),
            ];
            cmd.entity(entity_m).insert(MovementPath::new(&points_m, 2 , Vec3::X));
            cmd.entity(entity_m).insert(Paired);
            let start_time = time.elapsed_seconds() + *count  as f32 * 1.0;
            cmd.entity(entity_m).insert(MovementStart(start_time));

            if let Ok(entity_g) =  girl_q.get(pair.0) {
                let shift = dir.cross(Vec3::Y) * 2.0 *  (if *count % 2 == 0 {-1.} else {1.});
                let points_g = vec![
                    (p0 + shift, MAN_VELOCITY, Some(5), None),
                    (p1 + shift, MAN_VELOCITY, None, None),
                    (p2 + shift, MAN_VELOCITY, None, None),
                    (p3 + shift, MAN_VELOCITY, Some(12), Some(last_dir))
                ];
                cmd.entity(entity_g).insert(MovementPath::new(&points_g, 2 , Vec3::ZERO));
                cmd.entity(entity_g).insert(MovementStart(start_time));

            }
            *count += 1;
        }
    }
}

// ---

fn cam_rotate(
    men_q: Query<&Transform, With<Man>>,
    cam_q: Query<&Transform , With<Cam>>,
    mut ccc: Commands,
    mut rotated: Local<bool>
) {
    if *rotated {
        return;
    }
    if let Ok(cam_t) = cam_q.get_single() {
        for t in men_q.iter() {
            if t.translation.y < cam_t.translation.y {
                ccc.trigger(SetCameraTarget(
                    CameraTarget::from_position(Vec3::ZERO + Vec3::Y * 2.).with_translation_bias(Vec3::new(10., 10., 0.)), 0
                ));
                *rotated = true;
            }
        }
    }
}
