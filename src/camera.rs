use bevy::prelude::*;
use bevy::core_pipeline::{Skybox, motion_blur::* };
use bevy::{render::camera::Viewport, window::{PrimaryWindow, WindowResized}};

use crate::camera_target::CameraTarget;

// ---

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn) 
        .add_systems(Update, split.run_if(resource_added::<CameraSplitConf>))
        .add_systems(Update, (unsplit, spawn).chain().run_if(resource_removed::<CameraSplitConf>()))
        .add_systems(Update, set_camera_viewports
            .run_if(on_event::<WindowResized>())
            .run_if(in_state(CameraState::Splitted))
        )
        .init_state::<CameraState>()
        ;
    }
} 

// ---

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraState {
    #[default]
    Single,
    Splitted
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ViewportGeom {
    pub start: (f32, f32),
    pub size: (f32, f32)
}

#[derive(Resource)]
pub struct CameraSplitConf(pub Vec<(CameraTarget, ViewportGeom)>);


#[derive(Component)]
pub struct Cam;

#[derive(Component)]
pub struct Splitted;

// ---

fn spawn (
    mut commands : Commands,
    assets: ResMut<AssetServer>
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 50., 50.).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        MotionBlurBundle {
            motion_blur: MotionBlur {
                shutter_angle: 1.0,
                samples: 2,
                ..default()
            },
            ..default()
        },
        Skybox{
            image: assets.load("skyboxes/interstellar.ktx2"),
            brightness: 100.
        }, 
        Cam,
        EnvironmentMapLight {
            diffuse_map: assets.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: assets.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 800.0,
        },
    ));
}

// ---

pub fn split(
    mut com : Commands,
    mut e_w: EventWriter<WindowResized>,
    mut main_cam_q: Query<(Entity, &Skybox, &EnvironmentMapLight), With<Cam>>,
    q_window: Query<(Entity, &Window), With<PrimaryWindow>>,
    split_settings: Res<CameraSplitConf>,
    mut next: ResMut<NextState<CameraState>>
) {
    let Ok((mce, sb, eml)) = main_cam_q.get_single_mut() else {
        return;
    };
    com.entity(mce).despawn_recursive();

    for i in 0 .. split_settings.0.len() {
        let c = com.spawn((
            Camera3dBundle {
                camera: Camera {
                    order: i as isize,
                    clear_color: if i == 0 { ClearColorConfig::Default} else { ClearColorConfig::None},
                    ..default()
                },
                ..default()
            },
            split_settings.0[i].1.clone(),
            sb.clone(),
            eml.clone(),
            split_settings.0[i].0 ,
            Cam,
            Splitted
            
        )).id();

        com.spawn((
            TargetCamera(c),
            Splitted,
            viewport_node()
        ));

    }
    next.set(CameraState::Splitted);
    if let Ok((e_win, win)) = q_window.get_single()  {
        e_w.send(WindowResized {
            width: win.width(),
            height:win.height(),
            window: e_win
        });
    }

}

// ---

fn unsplit (
    d_q: Query<Entity, With<Splitted>>,
    mut ccc: Commands,
    mut next: ResMut<NextState<CameraState>>
) {
    for e in d_q.iter() {
        ccc.entity(e).despawn_recursive();
    }
    next.set(CameraState::Single);
}

// ---

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<(&ViewportGeom, &mut Camera)>,
) {
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let wsize = window.physical_size();
        for (vg, mut camera) in &mut query {
            camera.viewport = Some(Viewport {
                physical_position: scale(wsize, vg.start),
                physical_size: scale(wsize, vg.size),
                ..default()
            });
        }
    }
}

// ---

fn scale (a: UVec2, b: (f32, f32)) -> UVec2 {
    UVec2 {
        x: (a.x as f32 * b.0).trunc() as _, 
        y: (a.y as f32  * b.1).trunc() as _
    }
}

// ---

fn viewport_node() -> NodeBundle{
    NodeBundle {
        style: Style {
            padding: UiRect::all(Val::Px(5.)),
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        border_color: Color::WHITE.with_alpha(0.2).into(),
        ..default()
    }
}