use std::f32::consts::PI;

use bevy::prelude::*;
use crate::animator::{AllAnimations, AnimationKey, CurrentAnimation};
use crate::GameState;
use crate::camera::{CameraSplitConf, ViewportGeom, CameraState};
use crate::camera_target::{CameraTarget, SetCameraTarget};
use crate::music::MusicEvent;
use crate::movement::MovementPath;

// ---

pub struct IntroPlugin;
impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, load)
        .add_systems(OnEnter(GameState::Intro), enter_intro)
        .add_systems(OnEnter(CameraState::Splitted), set_ui.run_if(in_state(GameState::Intro)))
        .add_systems(Update, dialogue.run_if(in_state(GameState::Intro)))
        .add_systems(OnEnter(CameraState::Single), enter_portal.run_if(in_state(GameState::Portal)))
        .observe(music_event)
        ;
    }
}

// ---

#[derive(Component)]
pub struct Director;

#[derive(Component)]
pub struct RedGirl;

#[derive(Component)]
pub struct IntroMarker;

#[derive(Resource)]
pub struct Dialogue(Vec<(bool, f32, AnimationKey, &'static str)>);

// ---

fn load(
    mut cmd: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut all_animations: ResMut<AllAnimations>,
    assets: ResMut<AssetServer>
) {
    all_animations.add(AnimationKey::Director, "models/other/director.glb", 6, &mut graphs, &assets);
    all_animations.add(AnimationKey::RedGirl, "models/other/red-girl.glb", 6, &mut graphs, &assets);

    cmd.spawn((
        SceneBundle {
            scene: assets.load("models/other/director.glb#Scene0"),
            transform: Transform::from_translation(Vec3::new(-78.5, 79.5, 0.)).looking_to(-Vec3::X, Vec3::Y),
            visibility: Visibility::Hidden,
            ..default()
        },
        Director,
        IntroMarker,
        AnimationKey::Director,
    ));

    cmd.spawn((
        SceneBundle {
            scene: assets.load("models/other/red-girl.glb#Scene0"),
            transform: Transform::from_translation(Vec3::new(40.0, 6., 40.)).looking_to(-Vec3::X, Vec3::Y),
            visibility: Visibility::Hidden,
            ..default()
        },
        RedGirl,
        IntroMarker,
        AnimationKey::RedGirl,
    ));

} 

// ---

pub fn enter_intro(
    mut cmd: Commands,
    mut staff_q: Query<(Entity, &mut CurrentAnimation, &mut Visibility), With<IntroMarker>>,
    time: Res<Time>
) {

    let mut csc = CameraSplitConf(Vec::new());

    for ((e, mut ca, mut v), vps) in staff_q
        .iter_mut()
        .zip([
            (
                (Vec3::new(0.5, 3., 2.), Vec3::new(0., 1.,0.)),
                (0.0, 0.0)
            ), 
            (
                (Vec3::new(-1., 2., 8.), Vec3::new(0., 1.,0.)),
                (0.5, 0.0)
            ), 
        ]) 
    {
        ca.0 = 0;
        *v = Visibility::Visible;

        csc.0.push(
            (
                CameraTarget::from_entity(e)
                .with_translation_bias(vps.0.0)
                .with_rotation_bias(vps.0.1)
                , 
                ViewportGeom{ start: vps.1, size: (0.5, 0.5)}
            ),             
        )
    }

    csc.0.push(
        (
            CameraTarget::from_position(Vec3::new(0., 3., 22.))
            .with_direction(Dir3::Z)
            .with_velocity(0.5)
           , 
            ViewportGeom{ start: (0., 0.5), size: (1.0, 0.5)}
        ),             
    );
    cmd.insert_resource(csc);

    let start = time.elapsed_seconds();
    cmd.insert_resource(Dialogue(
        vec![
            (false, start + 4.0, AnimationKey::Director, "Ok.."),
            (false, start + 6.0, AnimationKey::Director, "There is no rain in the forecast?"),
            (false, start + 7.0, AnimationKey::RedGirl, "No"),
            (false, start + 8.0, AnimationKey::Director, "Girls on site?"),
            (false, start + 9.0, AnimationKey::RedGirl, "Yup"),
            (false, start + 10.0, AnimationKey::Director, "Are the propellers ready?"),
            (false, start + 11.0, AnimationKey::RedGirl, "Sure"),
            (false, start + 12.0, AnimationKey::Director, "By the way, be careful there, you might get carried away"),
            (false, start + 13.0, AnimationKey::RedGirl, "Don't worry"),
            (false, start + 14.0, AnimationKey::Director, "Let's  go!"),
            (false, start + 14.2, AnimationKey::RedGirl, "Go!"),
        ]
    ));


}

// ---

fn set_ui(
    mut nodes_q: Query<(Entity, &TargetCamera, &mut Style)>,
    cam_q: Query<&Camera>,
    mut cmd: Commands
) {
    for (etc, tc, mut style) in nodes_q.iter_mut() {
        if let Ok(cam) = cam_q.get(tc.0) {
            if cam.order < 2 {
                style.align_items = AlignItems::Center;
                style.justify_content = JustifyContent::Center;
                style.padding = UiRect::all(Val::Percent(10.));
                let text_id = cmd.spawn((
                    TextBundle::from_section("...", TextStyle{
                        font_size: 40.,
                        ..default()
                    }),
                )).id();
                if cam.order == 0 {
                    cmd.entity(text_id).insert(AnimationKey::Director);
                } else {
                    cmd.entity(text_id).insert(AnimationKey::RedGirl);
                }
                cmd.entity(etc).add_child(text_id);
            }
        } 
    }
}

// ---

fn dialogue(
    mut dialogue: ResMut<Dialogue>,
    time: Res<Time>,
    mut text_fields: Query<(&mut Text, &AnimationKey)>

) {
    let elapsed = time.elapsed_seconds();
    for (mut t, ak ) in text_fields.iter_mut() {
        if let Some(dk) = dialogue.0
            .iter_mut()
            .filter(|t| !t.0 )
            .find(|(_, t, k, _)| {
                k == ak &&  elapsed >= *t
            }) 
        {
            dk.0 = true;
            t.sections[0].value = dk.3.to_string();
        }
    }  
}

// ---

fn enter_portal (
    r_q: Query<(Entity, &Transform), With<RedGirl>>,
    mut cmd: Commands
) {
    if let Ok((r_e, t)) =  r_q.get_single() {
        cmd.trigger(SetCameraTarget(
            CameraTarget::from_entity(r_e)
            .with_translation_bias(Vec3::new(-10.0, 10.0, 5.0)).with_velocity(5.0)
            ,
            0
        ));
        let dir = Dir3::new((Vec3::ZERO - t.translation).normalize()).unwrap();
        cmd.entity(r_e).insert(
            MovementPath::new(&vec! [
                (t.translation.with_y(100.), 5., Some(3), Some(dir))
            ], 0, Vec3::ZERO)
        );        
    }
}

// ---

fn music_event(
    trigger: Trigger<MusicEvent>,
    mut dir_q: Query<(Entity, &mut Transform, &mut CurrentAnimation), With<Director>>,
    mut cmd: Commands,
) {
    let MusicEvent(GameState::Portal, count)  =  *trigger.event() else {
        return;
    };
    if count < 4 {
        return;
    }

    if let Ok((dir_e, mut dir_t, mut ca)) =  dir_q.get_single_mut() {
        if count == 4 {
            dir_t.rotate_y(PI);
            ca.0 = 1;
            cmd.trigger(SetCameraTarget(
                CameraTarget::from_entity(dir_e).with_translation_bias(Vec3::new(1., 10., -8.)), 0)
            );     
        } else if count == 5 {
            cmd.trigger(SetCameraTarget(
                CameraTarget::from_position(Vec3::new(0., 5.0, 40.)).with_direction(-Dir3::Z).with_velocity(0.4), 0)
            );       
        } else if count == 6 {
            cmd.trigger(SetCameraTarget(
                CameraTarget::from_position(Vec3::new(100., 5., 0.)).with_direction(-Dir3::X).with_velocity(2.), 0)
            );       
        } else if count == 7 {
            cmd.trigger(SetCameraTarget(
                CameraTarget::from_position(Vec3::new(100., 80., 0.)).with_direction(-Dir3::X), 0)
            );       
        } else if count == 8 {
            cmd.trigger(SetCameraTarget(
                CameraTarget::from_position(Vec3::new(0., 50., 0.)).with_direction(-Dir3::Y), 0)
            );       
        } 
    }
    
}
