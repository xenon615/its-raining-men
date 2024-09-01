use std::time::Duration;

use bevy::{audio::PlaybackMode, prelude::*, time::common_conditions::once_after_delay};
use crate::GameState;
pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(PreStartup, load)
        .add_systems(Update, play
            .run_if(once_after_delay(Duration::from_secs(15)))
            .run_if(in_state(GameState::Intro))
        )
        .add_systems(FixedUpdate, timeline.run_if(in_state(MusicState::Playing)))
        .init_resource::<MusicTimeline>()
        .init_state::<MusicState>()
        ;
    }
}

// ---

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MusicState {
    #[default]
    Stopeed,
    Playing
}

#[derive(Event, PartialEq, Clone, Copy, Debug)]
pub struct MusicEvent(pub GameState, pub usize);


#[derive(Resource)]
pub struct MusicTimeline {
    start: f32,
    events: Vec<(f32, bool, MusicEvent)>
}

impl FromWorld for MusicTimeline {
    fn from_world(_world: &mut World) -> Self {
        MusicTimeline {
            start: 0.,
            events: vec![
                (3.7, false, MusicEvent(GameState::Portal, 0)),    // screw start
                (10.8, false, MusicEvent(GameState::Portal, 1)),         // camera to leader  leader light more 
                (14.3, false, MusicEvent(GameState::Portal, 2)),  // red light
                (16.0, false, MusicEvent(GameState::Portal, 3)),  // yellow light

                (17.23, false, MusicEvent(GameState::Portal, 4)),
                (21.0, false, MusicEvent(GameState::Portal, 5)),
                (25.1, false, MusicEvent(GameState::Portal, 6)),
                (28.1, false, MusicEvent(GameState::Portal, 7)),
                (30.3, false, MusicEvent(GameState::Portal, 8)),

                (32.25, false, MusicEvent(GameState::Speak, 0)),

                (45.138, false, MusicEvent(GameState::Sing, 0)),
                (46.1, false, MusicEvent(GameState::Sing, 1)),  // humidity
                (56.7, false, MusicEvent(GameState::Sing, 2)) , // the streets
                (60.29, false, MusicEvent(GameState::Sing, 3)) , // 'Cause tonight for the first time
                (65., false, MusicEvent(GameState::Sing, 4)),  // the 10:30
                (68., false, MusicEvent(GameState::Sing, 5)),  // history

                (76., false, MusicEvent(GameState::Raining, 0)), // its raining

                (120.0, false, MusicEvent(GameState::Finish, 0))

            ]
        }
    }
}


// ---

fn load(
    mut commands: Commands,
    ass: ResMut<AssetServer>
) {
    commands.spawn((
        AudioBundle {
            source: ass.load("music/irm.ogg"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Once,
                paused: true,
                ..default()
            },
            ..default()
        },
    ));
}

// ---

fn play(
    sink_q : Query<&AudioSink>,
    mut timeline: ResMut<MusicTimeline>,
    time: Res<Time>,
    mut next: ResMut<NextState<MusicState>>,
    mut next_g: ResMut<NextState<GameState>>,
) {
    if let Ok(sink) = sink_q.get_single() {
        sink.play();
        timeline.start = time.elapsed_seconds();
        next.set(MusicState::Playing);
        next_g.set(GameState::Thunder);
    }
}

// ---

fn timeline (
    time: Res<Time>,
    mut tl: ResMut<MusicTimeline>,
    mut cmd: Commands,
    mut next: ResMut<NextState<GameState>>
) {
    let playtime = time.elapsed_seconds() - tl.start;

    for e in &mut tl.events {
        if !e.1 && playtime > e.0 {
            e.1 = true;
            if e.2.1 == 0 {
                next.set(e.2.0);
            } else {
                cmd.trigger(e.2);
            }
            // println!("{:?}", e.2);
        } 
    }    
}