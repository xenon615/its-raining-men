use bevy::prelude::*;
use bevy::input::keyboard::KeyboardInput;


use crate::{GameState, NotReady};
use crate::camera::CameraSplitConf;
pub struct EnvPlugin;
impl Plugin for EnvPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<LanternLight>()
        .register_type::<TowerClock>()
        .add_systems(Startup, startup)
        .add_systems(Update, set_lanterns.run_if(in_state(GameState::Loading)))
        .add_systems(Update, key_input.run_if(on_event::<KeyboardInput>()))
        .add_systems(Update, run_thunder.run_if(in_state(GameState::Thunder)))
        .add_systems(OnExit(GameState::Thunder), exit_thunder)
        ;
    }
}

// ---

#[derive(Component)]
pub struct Bench;

#[derive(Component)]
pub struct Seat;

#[derive(Component)]
pub struct Lantern;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct LanternLight;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct TowerClock;


#[derive(Component)]
pub struct Env;

#[derive(Component)]
pub struct Humidity;

#[derive(Component)]
pub struct History;

const SEATS_COUNT: u8 = 4;
const AMBIENT_LIGHT_DEFAULT: f32 = 0.0;

// ---

fn startup(
    mut commands: Commands,
    aserver: ResMut<AssetServer>,
    mut al: ResMut<AmbientLight>
) {
    al.brightness = AMBIENT_LIGHT_DEFAULT;
    commands.spawn(
        SceneBundle {
            scene: aserver.load("models/scenes/scene0.glb#Scene0"),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        }
    );

    commands.spawn((
        SceneBundle {
            scene: aserver.load("models/scenes/history.glb#Scene0"),
            transform: Transform::from_xyz(-100., 80., 0.).looking_to(Vec3::X, Vec3::Y),
            ..default()
        },
        History
    ));

    commands.spawn((
        SceneBundle {
            scene: aserver.load("models/scenes/humidity.glb#Scene0"),
            transform: Transform::from_xyz(88.9389, 20.6635, 0.).looking_to(-Vec3::X, Vec3::Y),
            ..default()
        },
        Humidity
    ));

    let bench = aserver.load("models/scenes/bench.glb#Scene0");
    let lantern = aserver.load("models/scenes/lantern.glb#Scene0");

    for i in 0 .. 2 {
        let is_horizontal = i == 0;
        let face_to = if is_horizontal {Vec3::Z} else {Vec3::X};
        for j in 0 .. 2 {
            let sign0 = if j == 0 {1.} else {-1.};
            let mut from = sign0 * if is_horizontal {Vec3::new(25., 0. ,0.)} else {Vec3::new(0., 0.,25.)};
            let step = from / 3.;
            
            for _k in 0 .. 2 {
                from += step;
                for l in  0 .. 2 {
                    let sign1 = if l == 0 {1.} else {-1.};
                    let bench_pos = from  + face_to * sign1 * 8.;
                    commands.spawn((
                        SceneBundle {
                            scene: bench.clone(),
                            transform: Transform::from_translation(bench_pos).looking_to(sign1 * face_to , Vec3::Y),
                            ..default()
                        },
                        Bench
                    ))
                    .with_children(|bench| {

                        let start = Vec3::new(-2., 0., 0.);
                        let lstep = Vec3::new(4. / SEATS_COUNT as f32, 0., 0.);
                        
                        for _c in 0 .. SEATS_COUNT{
                            bench.spawn((
                                TransformBundle {
                                    local: Transform::from_translation(start + lstep * (0.5 + _c as f32)),
                                ..default()
                                },
                                Seat
                            ));
                        } 

                    });
                    
                    let lantern_pos = bench_pos - step.normalize() * 4.;
                    commands.spawn((
                        SceneBundle {
                            scene: lantern.clone(),
                            transform: Transform::from_translation(lantern_pos).looking_to(sign1 * face_to , Vec3::Y),
                            ..default()
                        },
                        Lantern
                    ))
                    ;

                }
            }
        }
    }    
    commands.spawn((Env, NotReady));    
} 

// ---

fn set_lanterns(
    l_q : Query<(Entity, &Parent, &Transform), (With<LanternLight>, Without<SpotLight>)>,
    mut commands: Commands,
    ready_q : Query<Entity, (With<Env>, With<NotReady>)>,
    mut spawned: Local<bool>
) {

    if l_q.is_empty() {
        if *spawned {
            if let Ok(re) = ready_q.get_single() {
                commands.entity(re).despawn();
            }
        }
    } else {
        *spawned = true;
    }

    for (e, p, t) in l_q.iter() {
        let mut trans = t.with_rotation(Quat::from_rotation_x(-90_f32.to_radians()));
        trans.translation.y = -0.6;

        let plb = commands.spawn((
            SpotLightBundle {
                spot_light: SpotLight {
                    color: Color::srgb(1., 0.64, 0.),
                    intensity: 5_000_000.,
                    outer_angle: 2.8,
                    inner_angle: 4.5,
                    shadows_enabled: false,
                    ..default()
                },
                visibility: Visibility::Hidden,
                transform: trans,  
                ..default()
            },
            LanternLight
        )).id();

        commands.entity(**p).add_child(plb);
        commands.entity(e).despawn_recursive();
    }
}

// ---

fn key_input(
    keys: ResMut<ButtonInput<KeyCode>>,
    mut lantern_q: Query<&mut Visibility,  With<LanternLight>>
) {
    if keys.just_pressed(KeyCode::Space) {
        for mut v in &mut lantern_q {
            *v = if *v == Visibility::Visible {Visibility::Hidden} else {Visibility::Visible};
        }
    }
 }

//  ---


fn run_thunder(
    mut al: ResMut<AmbientLight>,
    mut cc: ResMut<ClearColor>,
    mut lb: Local<u8>
) {
    *lb += 1;
    if (*lb % 10 == 0) || (*lb % 13 == 0)  {
        al.brightness = 1000.;
        cc.0 = Color::WHITE;
    } else {
        al.brightness = AMBIENT_LIGHT_DEFAULT;
        cc.0 = Color::BLACK;
    }
}

// ---

fn exit_thunder(
    mut al: ResMut<AmbientLight> ,
    mut cc: ResMut<ClearColor>,
    mut cmd: Commands,

) {
    al.brightness = AMBIENT_LIGHT_DEFAULT;
    cc.0 = Color::BLACK;
    cmd.remove_resource::<CameraSplitConf>();
}
