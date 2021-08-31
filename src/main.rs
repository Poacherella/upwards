use bevy::{
    core::FixedTimestep,
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_prototype_debug_lines::*;


const TIME_STEP: f32 = 1.0 / 60.0;
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_startup_system(setup.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(ball_collision_system.system())
                .with_system(mine_selector_system.system())
                .with_system(gravity_system.system())
                .with_system(mine_highlighter_system.system())
                .with_system(draw_line_system.system())
                .with_system(mine_hook_system.system())
                .with_system(player_movement_system.system()),
        )
        .add_system(scoreboard_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

struct Mine {
    selected: bool,
    hooked: bool,
}

impl Default for Mine {
    fn default() -> Self {
        Self {
            selected: false,
            hooked: false,
        }
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

    // player
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(asset_server.load("bomb.png").into()),
            transform: Transform::from_xyz(0.0, -160.0, 1.0),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .insert(Player {
            velocity: Vec3::new(0.0, -0.5, 0.0).normalize(),
        });

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(asset_server.load("bomb.png").into()),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            transform: Transform::from_xyz(0.0, -50.0, 1.0),
            ..Default::default()
        })
        .insert(Mine::default());

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

        for (t_mine, mut mine) in q_mine.single_mut() {
            let a = Vec2::new(t_mine.translation.x, t_mine.translation.y);
            let b = Vec2::new(pos_wld.x, pos_wld.y);
            let d = dist(a, b);

            if d < 30. {
                mine.selected = true
            } else {
                mine.selected = false
            }
        }
    }
}

fn mine_highlighter_system(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_mine: Query<(&Sprite, &Handle<ColorMaterial>, &Mine)>,
) {
    for (_sprite, handle, m) in &mut q_mine.iter_mut() {
        // let material = materials.get(&handle).unwrap();
        let material = materials.get_mut(handle);
        if let Some(mat) = material {
            if m.selected {
                mat.color.set_g(10.);
            } else {
                mat.color.set_g(1.);
            }

            if m.hooked {
                mat.color.set_r(10.);
            } else {
                mat.color.set_r(1.);
            }
        }
    }
}

fn mine_hook_system(btns: Res<Input<MouseButton>>, mut q_mine: Query<&mut Mine>) {
    // Mouse buttons
    // if btns.
    if btns.just_pressed(MouseButton::Left) 
    {
        // a left click just happened
        for mut m in &mut q_mine.iter_mut() {
            if m.selected {
                m.hooked = true;
            } else {
                m.hooked = false;
            }
        }
    }
    if btns.just_released(MouseButton::Left) 
    
    {
        // deselect
        for mut m in &mut q_mine.iter_mut() {
         m.hooked = false;
        }
    }
}

// 2d dist
fn dist(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x).powf(2.) + (a.y - b.y).powf(2.)).sqrt()
}

fn gravity_system(mut player_query: Query<&mut Player>) {
    if let Ok(mut player) = player_query.single_mut() {
        player.velocity.y *= 1.011;
    }
}

fn target_mine_system(mut player_query: Query<(&mut Player, &Mine)>) {
    if let Ok((mut player, mine)) = player_query.single_mut() {
        // player.velocity.y *= mine.;
    }
}

fn draw_line_system(
    player_query: Query<(&Player, &Transform)>,
    mine_query: Query<(&Mine, &Transform)>,
    mut lines: ResMut<DebugLines>
) {
    if let Ok((mut player, p_t)) = player_query.single() {
        // player.velocity.y *= mine.;
        for (mine, m_t) in mine_query.iter() {
            if mine.hooked {
                lines.line(p_t.translation, m_t.translation, 0.);
            }
        }
    }
}

fn player_movement_system(mut player_query: Query<(&Player, &mut Transform)>) {
    if let Ok((player, mut transform)) = player_query.single_mut() {
        transform.translation += player.velocity;
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
