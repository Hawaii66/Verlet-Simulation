use std::time::Duration;

use bevy::prelude::{Commands, *};
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*};

#[derive(Component, Debug)]
struct Point {
    x: f32,
    y: f32,
    old_x: f32,
    old_y: f32,
    radius: f32,
    color: u8,
    id: i32,
    acc_x: f32,
    acc_y: f32,
}

#[derive(Resource)]
struct Bounds {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

enum Axis {
    Horizontal,
    Vertical,
}

impl Bounds {
    fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Bounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    fn constrain_point(&self, point: &mut Point, axis: Axis) {
        match axis {
            Axis::Horizontal => {
                if point.x > self.max_x as f32 {
                    let vel_x = point.vel_x() * FRICTION;
                    point.x = self.max_x as f32;
                    point.old_x = self.max_x as f32 + vel_x * BOUNCE;
                } else if point.x < self.min_x as f32 {
                    let vel_x = point.vel_x() * FRICTION;
                    point.x = self.min_x as f32;
                    point.old_x = self.min_x as f32 + vel_x * BOUNCE;
                }
            }
            Axis::Vertical => {
                if point.y > self.max_y as f32 {
                    let vel_y = point.vel_y() * FRICTION;
                    point.y = self.max_y as f32;
                    point.old_y = self.max_y as f32 + vel_y * BOUNCE;
                } else if point.y < self.min_y as f32 {
                    let vel_y = point.vel_y() * FRICTION;
                    point.y = self.min_y as f32;
                    point.old_y = self.min_y as f32 + vel_y * BOUNCE;
                }
            }
        }
    }
}

impl Clone for Point {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            old_x: self.old_x.clone(),
            old_y: self.old_y.clone(),
            radius: self.radius.clone(),
            color: self.color.clone(),
            id: self.id.clone(),
            acc_x: 0.0,
            acc_y: 0.0,
        }
    }
}

impl Point {
    fn new(id: i32, x: f32, y: f32, vel_x: f32, vel_y: f32) -> Self {
        let old_x = x - vel_x;
        let old_y = y - vel_y;

        //println!("{:?} {:?} {:?}", x, old_x, vel_x);

        let p = Point {
            id,
            x,
            y,
            old_x,
            old_y,
            radius: 1.0,
            color: 0,
            acc_x: 0.0,
            acc_y: 0.0,
        };
        //println!("{:?}", p);
        p
    }

    fn vel_x(&self) -> f32 {
        self.x - self.old_x
    }
    fn vel_y(&self) -> f32 {
        self.y - self.old_y
    }

    fn move_point(&mut self, bounds: &Bounds, dt: f32) {
        let vel_x = self.vel_x() * FRICTION;
        let vel_y = self.vel_y() * FRICTION;

        self.old_x = self.x;
        self.old_y = self.y;

        self.x += vel_x + self.acc_x * dt * dt;
        self.y += vel_y + self.acc_y * dt * dt;

        self.acc_x = 0.0;
        self.acc_y = 0.0;

        bounds.constrain_point(self, Axis::Horizontal);
        bounds.constrain_point(self, Axis::Vertical);

        //println!("{:?}", self);
    }

    fn apply_acceleration(&mut self, x: f32, y: f32) {
        self.acc_x += x;
        self.acc_y += y;
    }

    fn dist(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dist = (dx * dx + dy * dy).sqrt();
        dist
    }

    fn colliding(&self, other: &Point) -> bool {
        let dist = self.dist(other);
        let colliding = self.radius + other.radius > dist;
        colliding
    }
}

fn solve_collision(p1: &mut Point, p2: &mut Point) {
    let delta_x = p1.x - p2.x;
    let delta_y = p1.y - p2.y;

    let dist = p1.dist(p2);
    let n_x = delta_x / dist;
    let n_y = delta_y / dist;

    let delta = p1.radius + p2.radius - dist;

    p1.x += 0.5 * delta * n_x * FRICTION;
    p1.y += 0.5 * delta * n_y * FRICTION;
    p2.x -= 0.5 * delta * n_x * FRICTION;
    p2.y -= 0.5 * delta * n_y * FRICTION;
}

const GRAVITY: f32 = -9.80;
const FRICTION: f32 = 0.99;
const BOUNCE: f32 = 0.95;
const SUBSTEPS: u8 = 8;

const GAME_SCALE: f32 = 20.0;

fn create_sprite(radius: f32, color: Color) -> ShapeBundle {
    let shape = shapes::RegularPolygon {
        sides: 24,
        feature: shapes::RegularPolygonFeature::Radius(radius),
        ..shapes::RegularPolygon::default()
    };

    GeometryBuilder::build_as(
        &shape,
        DrawMode::Outlined {
            fill_mode: FillMode::color(color),
            outline_mode: StrokeMode::new(Color::BLACK, 0.2),
        },
        Transform {
            translation: Vec3::ZERO,
            scale: Vec3::new(GAME_SCALE, GAME_SCALE, GAME_SCALE),
            ..default()
        },
    )
}

fn add_points(mut commands: Commands) {
    commands.spawn((
        create_sprite(1.0, Color::BLUE),
        Point::new(0, 5.0, 20.0, 0.1, 0.0),
    ));
}

fn update_points_system(mut query: Query<&mut Point>, time: Res<Time>, bounds: Res<Bounds>) {
    for mut point in query.iter_mut() {
        let sub_dt = time.delta_seconds() / (SUBSTEPS as f32);
        println!("Sub {:?}", sub_dt);
        for _ in 0..SUBSTEPS {
            point.apply_acceleration(0.0, GRAVITY);
            point.move_point(&bounds, sub_dt);
        }
    }

    let mut i = query.iter_combinations_mut();
    while let Some([mut p1, mut p2]) = i.fetch_next() {
        if p1.colliding(p2.as_ref()) {
            solve_collision(p1.as_mut(), p2.as_mut());
        }
    }
}

fn update_visual_point(mut query: Query<(&Point, &mut Transform)>) {
    for (point, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(point.x * GAME_SCALE, point.y * GAME_SCALE, 0.0);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource)]
struct SpawnTimer {
    timer: Timer,
    id: i32,
}

fn spawn_item(mut commands: Commands, time: Res<Time>, mut config: ResMut<SpawnTimer>) {
    config.timer.tick(time.delta());

    if config.timer.finished() {
        commands.spawn((
            create_sprite(1.0, Color::RED),
            Point::new(config.id, 0.0, 20.0, 0.05, 0.0),
        ));
        config.id += 1;
    }
}

fn main() {
    let bounds = Bounds::new(0, 0, 21, 50);

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(bounds)
        .insert_resource(SpawnTimer {
            timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
            id: 10,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup_scene)
        .add_startup_system(add_points)
        .add_system(update_points_system)
        .add_system(update_visual_point)
        .add_system(spawn_item)
        .run();
}
