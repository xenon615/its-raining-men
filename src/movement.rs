use bevy::prelude::*;

use crate::animator::CurrentAnimation;
pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, setup)
        .add_systems(Update, do_moving.run_if(any_with_component::<MovementPath>))
        ;
    }
}

// ---
#[derive(Component)]
pub struct MovementStart(pub f32);

#[derive(Clone, Copy)]
pub struct PathElement {
    pub pos: Vec3,
    pub velocity: f32,
    pub animation_index: Option<usize>,
    pub look_to: Option<Dir3>
}

#[derive(Component)]
pub struct MovementPath {
    pub points: Vec<PathElement>,
    pub finish_animation: usize,
    pub finish_look_at: Vec3
}

impl MovementPath {
    pub fn new(points: &Vec<(Vec3, f32, Option<usize>, Option<Dir3>)>, finish_animation: usize, finish_look_at: Vec3 ) -> Self {
        MovementPath {
            points: points.iter().map(|(p, v, a, l)| {
                    PathElement {
                        pos: *p,
                        velocity: *v, 
                        animation_index: *a,
                        look_to: *l
                    }
                    }).collect(),
            finish_animation,
            finish_look_at        
        }

    }
}

// ---

#[derive(Event)]
pub struct MovementPathDone(pub Entity);

// ---

fn setup(
    world: &mut World
) {
    world.register_component_hooks::<MovementPath>()
    .on_insert(|mut world, entity, _| {
        let mp0 = world.get::<MovementPath>(entity).unwrap().points[0];
        let idx = mp0.animation_index.unwrap();
        let mut ca = world.get_mut::<CurrentAnimation>(entity).unwrap();
        ca.0 = idx;
        let mut t =  world.get_mut::<Transform>(entity).unwrap();
        t.look_at(mp0.pos, Vec3::Y);
    })
    .on_remove(|mut world, entity, _| {
        world.send_event(MovementPathDone(entity));
    })
    ;
}

// ---

pub fn do_moving (
    mut objects_q: Query<(&mut Transform, &mut MovementPath, &mut CurrentAnimation, Option<&MovementStart>, Entity), With<MovementPath>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (mut t, mut mp, mut ca, ms, entity) in objects_q.iter_mut() {
        if let Some(start) = ms {
            if start.0 > time.elapsed_seconds() {
                continue;
            }
            commands.entity(entity).remove::<MovementStart>();
        } 
        let delta = mp.points[0].pos - t.translation;
        let step = time.delta_seconds() * mp.points[0].velocity;

        if delta.length_squared() < step * step {
            t.translation = mp.points[0].pos;
            commands.trigger(MovementPathDone(entity));
            mp.points.remove(0);
            
            if mp.points.len() == 0 {
                ca.0 = mp.finish_animation;
                commands.entity(entity).remove::<MovementPath>();
                
                t.look_at(mp.finish_look_at, Vec3::Y);
            } else {
                if let Some(ai) = mp.points[0].animation_index {
                    ca.0 = ai;   
                }
                if let Some(l_to)  = mp.points[0].look_to  {
                    t.look_to(l_to, Vec3::Y);
                } else {
                    t.look_at(mp.points[0].pos, Vec3::Y);
                }
                
            }
        } else {
            t.translation += step * delta.normalize();
        }
    }
}

// ---

#[allow(dead_code)]
pub fn debug_moving<M: Component>(
    objects_q: Query<(&mut Transform, &mut MovementPath), With<M>>,
    mut gizmos: Gizmos
) {
    for (gt, mp) in &objects_q {
        if mp.points.len() > 0 {
            gizmos.line(gt.translation, mp.points[0].pos, Color::srgb(0., 1., 0.));
        }
    }
}