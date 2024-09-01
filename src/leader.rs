use bevy::prelude::*;
use crate::
{
    animator::{AnimationKey, CurrentAnimation}, 
    camera_target::{CameraTarget, SetCameraTarget}, 
    intro::{Director, RedGirl}, 
    movement::{MovementPath, MovementPathDone}, music::MusicEvent, GameState
};

use crate::girls::GIRL_VELOCITY;
use crate::camera::{CameraSplitConf, ViewportGeom, CameraState};
pub struct  LeaderPlugin;

impl Plugin for LeaderPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn)
        .add_systems(OnEnter(GameState::Portal), enter_portal)
        .add_systems(OnEnter(GameState::Speak), enter_speak)
        .add_systems(OnEnter(GameState::Sing), enter_sing)
        .add_systems(OnExit(GameState::Sing), exit_sing)
        .add_systems(OnEnter(GameState::Finish), enter_finish)
        .add_systems(Update, finish
            .run_if(on_event::<MovementPathDone>())
            .run_if(in_state(GameState::Finish))
        )
        .add_systems(OnEnter(CameraState::Single), super_finish.run_if(in_state(GameState::Finish)))
        .observe(music_event)
        ;
    }
} 

// ---

#[derive(Component)]
pub struct Leader;

#[derive(Component)]
pub struct LeaderLight;

// ---

fn spawn (
    mut commands: Commands,
    assets: ResMut<AssetServer>,
) {
    commands.spawn((
        SceneBundle {
            scene: assets.load("models/girls/girl.glb#Scene0"),
            transform: Transform::from_translation(Vec3::ZERO),
            visibility: Visibility::Hidden,
            ..default()
        },
        Leader,
        AnimationKey::Girl
    ));

    commands.spawn((
        SpotLightBundle {
            transform: Transform::from_xyz(0., 8., 0.).with_rotation(Quat::from_rotation_x(-1.5)),
            spot_light: SpotLight {
                intensity: 5_000.,
                color: Color::srgb(0.9, 0.8, 0.),
                range: 20.,
                radius:2.,
                ..default()
            },
            ..default()
        },
        LeaderLight
    ));

}

// ---

fn enter_portal(
    mut l_q: Query<(&mut CurrentAnimation, &mut Visibility), With<Leader>>,
) {
    if let Ok((mut ca, mut v)) = l_q.get_single_mut() {
        ca.0 = 3;
        *v = Visibility::Visible;
    }
}

// ---

fn enter_speak(
    mut l_q: Query<(&mut CurrentAnimation, Entity), With<Leader>>,
    mut sl_q: Query<&mut SpotLight, With<LeaderLight>>,
    mut cmd: Commands
) {
    if let Ok((mut ca, te)) = l_q.get_single_mut() {
        ca.0 = 0;
        cmd.trigger(SetCameraTarget(CameraTarget::from_entity(te).with_translation_bias(Vec3::new(2., 5., 10.)), 0 ));
    }
    
    if let Ok(mut sl) = sl_q.get_single_mut() {
        sl.intensity *= 2.5;
        sl.color =  Color::srgb(0.9, 0.8, 0.3);
    }
}

// ---

fn music_event(
    trigger: Trigger<MusicEvent>,
    mut sl_q: Query<&mut SpotLight, With<LeaderLight>>,
    leader_q: Query<Entity, With<Leader>>,
    mut cmd: Commands,
) {
   
    let MusicEvent(GameState::Portal, count  ) = *trigger.event() else {
        return;
    };
    if count > 3 {
        return;
    }

    let mut sl = sl_q.get_single_mut().unwrap();

    if count == 1 {
        if let Ok(le) = leader_q.get_single() {
            cmd.trigger(SetCameraTarget(CameraTarget::from_entity(le).with_translation_bias(Vec3::new(0., 5., 10.)).with_velocity(0.3), 0 ));
        }
        sl.intensity = 5_000_000.;
    } else if count == 2{
        sl.color = Color::srgb(6., 0., 0.);
    } else {
        sl.color = Color::srgb(3., 3., 0.);
    }
}

fn enter_sing(
    mut cmd: Commands,
    leader_q: Query<Entity, With<Leader>>
) {
    let Ok(le) = leader_q.get_single() else {
        return;
    };
    let ct = CameraTarget::from_entity(le);
    let csc = CameraSplitConf(
        vec![
            (
                ct
                .with_translation_bias(Vec3::new(0., 2., 5.)) 
                .with_rotation_bias(Vec3::new(0., 1., 0.)), 
                ViewportGeom{start: (0., 0.),size: (0.5, 1.0)}
            ),
            (
                ct.with_translation_bias(Vec3::new(5., 3., 0.))
                .with_rotation_bias(Vec3::new(0., 1., 0.)),  
                ViewportGeom{start: (0.5, 0.),size: (0.5, 0.5)}
            ),
            (
                ct.with_translation_bias(Vec3::new(-5., 3., 0.))
                .with_rotation_bias(Vec3::new(0., 1., 0.)),  
                ViewportGeom{start: (0.5, 0.5),size: (0.5, 0.5)}
            )
        ]
    );
    cmd.insert_resource(csc);

}

fn exit_sing(
    mut cmd: Commands   
) {
    cmd.remove_resource::<CameraSplitConf>()
}

// ---

fn enter_finish(
    mut cmd : Commands,
    dir_q: Query<Entity, With<Director>>,
    rg_q: Query<Entity, With<RedGirl>>,
    l_q: Query<Entity, With<Leader>>,

) {
    let meet_point = Vec3::new(89.0, 50.5, -10.7);
    let de = dir_q.get_single().unwrap();
    let re = rg_q.get_single().unwrap();
    let le = l_q.get_single().unwrap();
    
    let mut csc = CameraSplitConf(
        vec![
            (
                CameraTarget::from_entity(de).with_translation_bias(Vec3::new(10., 6., 1.)),
                ViewportGeom{ start: (0., 0.), size: (0.35, 0.5)}
            ),
            (
                CameraTarget::from_entity(re).with_translation_bias(Vec3::new(10., 2., 1.)),
                ViewportGeom{ start: (0.35, 0.), size: (0.3, 0.5)}
            ),
            (
                CameraTarget::from_entity(le).with_translation_bias(Vec3::new(10., 10., 1.)),
                ViewportGeom{ start: (0.65, 0.), size: (0.35, 0.5)}
            )
        ] 
    );

    let velo = GIRL_VELOCITY * 3.;

    cmd.entity(de).insert(MovementPath::new(&vec![
        (meet_point + Vec3::Z, velo, Some(4), None)
    ], 5, -Vec3::X));

    cmd.entity(re).insert(MovementPath::new(&vec![
        (meet_point + Vec3::X * 0.25, velo, Some(3), None)
    ], 5, -Vec3::X));

    cmd.entity(le).insert(MovementPath::new(&vec![
        (meet_point - Vec3::Z + Vec3::X * 0.25, velo, Some(12), None)
    ], 9, -Vec3::X));

    csc.0.push(
        (
            CameraTarget::from_position(Vec3::ZERO).with_translation_bias(Vec3::new(50., 50., 50.)), 
            ViewportGeom{ start: (0., 0.5), size: (1., 0.5)}
        ),             
    );
    cmd.insert_resource(csc);
}

// ---

fn finish (
    check_q: Query<Entity, With<Leader>>,
    mut er: EventReader<MovementPathDone>,
    mut cmd: Commands
) {
    for ev in er.read() {
        if check_q.contains(ev.0) {
            cmd.remove_resource::<CameraSplitConf>();
        }
    }
}

// ---

fn super_finish(
    mut cmd: Commands,
    l_q: Query<Entity, With<Leader>>
) {
    let le = l_q.get_single().unwrap();
    cmd.trigger(SetCameraTarget(
        CameraTarget::from_entity(le)
        .with_translation_bias(Vec3::new(0., 1.0, -6.))
        .with_rotation_bias(Vec3::new(0., 2.0, 0.0))
        ,
        0
    ));
}