use bevy::{
    core::FixedTimestep,
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};

const TIME_STEP: f32 = 1.0 / 60.0;
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_startup_system(setup.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(ball_collision_system.system())
                .with_system(mine_selector_system.system())
                .with_system(gravity_system.system()),
        )
        .add_system(scoreboard_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

struct Mine {
    active: bool,
}

impl Default for Mine {
    fn default() -> Self {
        Self { active: false }
    }
}
struct MainCamera;
struct Player {
    velocity: Vec3,
}

struct Scoreboard {
    score: usize,
}

enum Collider {
    Solid,
    Mine,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Add the game's entities to our world

    // cameras
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn_bundle(UiCameraBundle::default());

    // ball
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
            transform: Transform::from_xyz(0.0, -160.0, 1.0),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .insert(Player {
            velocity: 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
        });

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 1.0, 0.5).into()),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            transform: Transform::from_xyz(0.0, -50.0, 1.0),
            ..Default::default()
        })
        .insert(Mine::default())
        ;

    // scoreboard
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("vcr.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("vcr.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(1.0, 0.5, 0.5),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

fn mine_selector_system(
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    q_camera: Query<&Transform, With<MainCamera>>,
    mut q_mine: Query<(&Transform, &mut Mine)>,
) {
    // get the primary window
    let wnd = wnds.get_primary().unwrap();

    // check if the cursor is in the primary window
    if let Some(pos) = wnd.cursor_position() {
        // get the size of the window
        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let p = pos - size / 2.0;

        // assuming there is exactly one main camera entity, so this is OK
        let camera_transform = q_camera.single().unwrap();

        // apply the camera transform
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
        // eprintln!("World coords: {}/{}", pos_wld.x, pos_wld.y);

        for (t_mine, mut mine) in q_mine.single_mut() {
            let a = Vec2::new(t_mine.translation.x, t_mine.translation.y);
            let b = Vec2::new(pos_wld.x, pos_wld.y);
            let d = dist(a, b);
            // eprintln!("d {}", d);

            if d < 30. {
                mine.active = true
            } else {
                mine.active = false
            }
        }
    }
}

// 2d dist
fn dist(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x).powf(2.) + (a.y - b.y).powf(2.)).sqrt()
}

fn ball_movement_system(mut ball_query: Query<(&Player, &mut Transform)>) {
    if let Ok((ball, mut transform)) = ball_query.single_mut() {
        // transform.translation += ball.velocity * TIME_STEP;
    }
}

fn gravity_system(mut player_query: Query<(&Player, &mut Transform)>) {
    if let Ok((_ball, mut transform)) = player_query.single_mut() {
        transform.translation -= Vec3::new(0., 30.23, 0.) * TIME_STEP;
    }
}

fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut().unwrap();
    text.sections[0].value = format!("Score: {}", scoreboard.score);
}

fn ball_collision_system(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Player, &Transform, &Sprite)>,
    collider_query: Query<(Entity, &Collider, &Transform, &Sprite)>,
) {
    if let Ok((mut ball, ball_transform, sprite)) = ball_query.single_mut() {
        let ball_size = sprite.size;
        let velocity = &mut ball.velocity;

        // check collision with walls
        for (collider_entity, collider, transform, sprite) in collider_query.iter() {
            let collision = collide(
                ball_transform.translation,
                ball_size,
                transform.translation,
                sprite.size,
            );
        }
    }
}
