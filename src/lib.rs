use bevy::{
    core::FixedTimestep,
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};
#[cfg(not(target_arch = "wasm32"))]
use bevy_prototype_debug_lines::*;
use rand::Rng;
use wasm_bindgen::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;
// left, right, bottom, top
const GAME_BOARD: (f32, f32, f32, f32) = (-500.0, 500.0, -350.0, 350.0);
const GRAVITY_FAC: f32 = 0.03;


#[derive(PartialEq, Eq, Hash, Debug, Clone)]
enum AppState {
    Menu,
    Game,
    End,
}
struct Mine {
    selected: bool,
    hooked: bool,
    velocity: Vec3,
}

// Just a marker for the bg
struct Background;
struct MainCamera;

impl Default for Mine {
    fn default() -> Self {
        Self {
            selected: false,
            hooked: false,
            velocity: Vec3::default(),
        }
    }
}
struct Player {
    velocity: Vec3,
    maxheight: f32,
    dead: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            velocity: Vec3::new(0.5, 2.5, 0.0),
            maxheight: 0.0,
            dead: false,
        }
    }
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        // .add_startup_system(setup.system())
        .add_state(AppState::Game)
        .add_system_set(
            SystemSet::on_enter(AppState::Game)
            .with_system(setup.system())
        )
        .add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(mine_selector_system.system())
                .with_system(mine_highlighter_system.system())
                .with_system(draw_line_system.system())
                .with_system(velocity_towards_mine_system.system())
                .with_system(velocity_towards_player_system.system())
                .with_system(gravity_system.system())
                .with_system(ball_collision_system.system())
                .with_system(move_camera_system.system())
                .with_system(is_player_dead_system.system())
                .with_system(bg_system.system())
                .with_system(player_too_low_system.system())
                .with_system(spawn_new_mine_system.system())
                .with_system(clean_old_mines_system.system())
                .with_system(mine_hook_system.system())
                .with_system(mine_movement_system.system())
                .with_system(player_movement_system.system()),
        )
        .add_system_set(SystemSet::on_exit(AppState::Game)
        .with_system(clean_assets_system.system())
    )
        .add_system(scoreboard_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system());
    // app.add_state(AppState::End);
    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugin(DebugLinesPlugin);
    app.run();
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
        .insert(Player::default());

    // bg
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(GAME_BOARD.1 * 2.05, 1000.0)),
            ..Default::default()
        })
        .insert(Background);

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

/// Use the mouse to select a mine
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

        for (t_mine, mut mine) in q_mine.iter_mut() {
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

/// Highlight mine under cursor and if hooked
fn mine_highlighter_system(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_mine: Query<(&Sprite, &Handle<ColorMaterial>, &Mine)>,
) {
    for (_sprite, handle, m) in &mut q_mine.iter_mut() {
        if let Some(mat) = materials.get_mut(handle) {
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

/// Mark a mine hooked
fn mine_hook_system(btns: Res<Input<MouseButton>>, mut q_mine: Query<&mut Mine>) {
    if btns.just_pressed(MouseButton::Left) {
        // a left click just happened
        for mut m in &mut q_mine.iter_mut() {
            if m.selected {
                m.hooked = true;
            } else {
                m.hooked = false;
            }
        }
    }
    if btns.just_released(MouseButton::Left) {
        // deselect
        for mut m in &mut q_mine.iter_mut() {
            m.hooked = false;
        }
    }
}

// Make sure there are enough things to grab
fn spawn_new_mine_system(
    q_mine: Query<&Transform, With<Mine>>,
    q_player: Query<&Player>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if let Ok(p) = q_player.single() {
        let mut highest_mine: f32 = 0.0;
        for mine_t in q_mine.iter() {
            highest_mine = highest_mine.max(mine_t.translation.y);
        }
        if highest_mine - p.maxheight < 100. {
            let mut rng = rand::thread_rng();

            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(asset_server.load("bomb.png").into()),
                    sprite: Sprite::new(Vec2::new(30.0, 30.0)),
                    transform: Transform::from_xyz(
                        rng.gen_range(GAME_BOARD.0..GAME_BOARD.1),
                        rng.gen_range(p.maxheight + 300.0..p.maxheight + 350.),
                        1.0,
                    ),
                    ..Default::default()
                })
                .insert(Mine::default());
        }
    }
}

/// Follow the player upward
fn move_camera_system(
    q_player: Query<&Player>,
    mut q_cam: Query<&mut Transform, With<MainCamera>>,
) {
    if let Ok(p) = q_player.single() {
        if let Ok(mut cam_t) = q_cam.single_mut() {
            cam_t.translation.y = p.maxheight
        }
    }
}

/// Set player dead if too low
fn player_too_low_system(mut q_player: Query<(&mut Player, &Transform)>) {
    if let Ok((mut p, t)) = q_player.single_mut() {
        if p.maxheight - t.translation.y > GAME_BOARD.2.abs() {
            dbg!("too low");
            p.dead = true;
        }
    }
}


/// Tint the background as you get higher
fn bg_system(
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_player: Query<&Player>,
    mut q_bg: Query<(&mut Transform, &Sprite, &Handle<ColorMaterial>), With<Background>>,
) {
    if let Ok(p) = q_player.single() {
        if let Ok((mut bg_t, _s, handle)) = q_bg.single_mut() {
            bg_t.translation.y = p.maxheight;
            if let Some(mat) = materials.get_mut(handle) {
                mat.color.set_g(p.maxheight / 10000.);
            }
        }
    }
}

// 2d dist
fn dist(a: Vec2, b: Vec2) -> f32 {
    ((a.x - b.x).powf(2.) + (a.y - b.y).powf(2.)).sqrt()
}

/// Very simple gravity system
fn gravity_system(mut player_query: Query<&mut Player>) {
    if let Ok(mut player) = player_query.single_mut() {
        player.velocity -= Vec3::Y * GRAVITY_FAC;
    }
}

fn velocity_towards_mine_system(
    mut player_query: Query<(&mut Player, &Transform)>,
    mine_query: Query<(&Mine, &Transform)>,
) {
    if let Ok((mut player, p_t)) = player_query.single_mut() {
        // player.velocity.y *= mine.;
        for (mine, m_t) in mine_query.iter() {
            if mine.hooked {
                let dir = m_t.translation - p_t.translation;
                player.velocity += dir.normalize() * 0.15;
            }
        }
    }
}

/// Drag mines towards players
fn velocity_towards_player_system(
    player_query: Query<&Transform, With<Player>>,
    mut mine_query: Query<(&mut Mine, &Transform)>,
) {
    if let Ok(p_t) = player_query.single() {
        for (mut mine, m_t) in mine_query.iter_mut() {
            if mine.hooked {
                let dir = p_t.translation - m_t.translation;
                mine.velocity += dir.normalize() * 0.05;
            } else {
                // slow down mine again if not hooked
                mine.velocity *= 0.9;
            }
        }
    }
}

fn draw_line_system(
    player_query: Query<&Transform, With<Player>>,
    mine_query: Query<(&Mine, &Transform)>,
    mut lines: ResMut<DebugLines>,
) {
    if let Ok(p_t) = player_query.single() {
        // player.velocity.y *= mine.;
        for (mine, m_t) in mine_query.iter() {
            if mine.hooked {
                lines.line(p_t.translation, m_t.translation, 0.);
            }
        }
    }
}

/// Move player and update the max y position
fn player_movement_system(mut player_query: Query<(&mut Player, &mut Transform)>) {
    if let Ok((mut player, mut transform)) = player_query.single_mut() {
        transform.translation += player.velocity;
        player.maxheight = transform.translation.y.max(player.maxheight);
    }
}

/// Move mines
fn mine_movement_system(mut mine_query: Query<(&mut Transform, &Mine)>) {
    for (mut transform, mine) in mine_query.iter_mut() {
        transform.translation += mine.velocity;
    }
}

/// Despawn mines that are too low (we'll never need them again)
fn clean_old_mines_system(
    mut mine_query: Query<(&mut Transform, Entity), With<Mine>>,
    player_query: Query<&Player>,
    mut commands: Commands,
) {
    if let Ok(player) = player_query.single() {
        for (transform, mine) in mine_query.iter_mut() {
            if transform.translation.y < player.maxheight + GAME_BOARD.2 {
                commands.entity(mine).despawn();
            }
        }
    }
}

/// clean up everything
fn clean_assets_system(
    mut mine_query: Query<Entity, With<Mine>>,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    if let Ok(player) = player_query.single() {
        commands.entity(player).despawn();
    }
    for mine in mine_query.iter_mut() {
        commands.entity(mine).despawn();
    }
}

/// Trigger state change
fn is_player_dead_system(mut app_state: ResMut<State<AppState>>, player_query: Query<&Player>) {
    if let Ok(player) = player_query.single() {
        if player.dead {
            app_state.set(AppState::End).unwrap();
        }
    }
}

/// update the score
fn scoreboard_system(mut query: Query<&mut Text>, player_query: Query<&Player>) {
    if let Ok(player) = player_query.single() {
        let mut text = query.single_mut().unwrap();
        text.sections[0].value = format!("Score: {:}", player.maxheight as i32);
    }
}

/// Very simple wall collision (left/right)
fn ball_collision_system(mut ball_query: Query<(&mut Player, &Transform)>) {
    if let Ok((mut player, p_t)) = ball_query.single_mut() {
        // check collision with walls and "reflect"
        if p_t.translation.x < GAME_BOARD.0 || p_t.translation.x > GAME_BOARD.1 {
            player.velocity.x *= -1.;
            // dampen a bit on impact (seems to get player stuck, perhaps better w/ fixed timestep)
            // player.velocity *= 0.9;
        }
    }
}
