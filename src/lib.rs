use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin};

use bevy::{
    core::FixedTimestep,
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};
use rand::Rng;
use wasm_bindgen::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;
// left, right, bottom, top
const GAME_BOARD: (f32, f32, f32, f32) = (-200.0, 200.0, -340.0, 330.0);
const GRAVITY_FAC: f32 = 0.09;
// const GRAVITY_FAC: f32 = 0.02;
const UPWARD_FAC: f32 = 0.25;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
enum AppState {
    WarmUp,
    Menu,
    Game,
}
struct Mine {
    selected: bool,
    hooked: bool,
    velocity: Vec3,
}

// Just a marker for the bg
struct Background;
struct MainCamera;
//Marker for the highscoretext
struct ScoreText;

struct Line;

impl Default for Mine {
    fn default() -> Self {
        Self {
            selected: false,
            hooked: false,
            velocity: Vec3::default(),
        }
    }
}

#[derive(Debug, Default)]
struct Player {
    velocity: Vec3,
    maxheight: f32,
    dead: bool,
}

// impl Default for Player {
//     fn default() -> Self {
//         Self {
//             velocity: Vec3::default(),
//             maxheight: 0.0,
//             dead: false,
//         }
//     }
// }

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        width: 500.,
        height: 800.,
        ..Default::default()
    });
    // app.add_plugin(AudioPlugin);
    app.add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .init_resource::<ButtonMaterials>()
        .add_state(AppState::WarmUp)
        .add_system_set(
            SystemSet::on_enter(AppState::WarmUp)
                .with_system(setup.system())
                .with_system(start_music.system()),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::Game)
                .with_system(init_game.system())
        )
        .add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(velocity_towards_mine_system.system())
                .with_system(velocity_towards_player_system.system())
                .with_system(gravity_system.system())
                .with_system(ball_collision_system.system())
                .with_system(bg_system.system())
                .with_system(spawn_new_mine_system.system())
                // .with_system(play_hooked_system.system())
                .with_system(clean_old_mines_system.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(mine_movement_system.system())
                .with_system(player_movement_system.system())
                .with_system(mine_selector_system.system())
                .with_system(mine_highlighter_system.system())
                .with_system(draw_line_system.system())
                .with_system(move_camera_system.system())
                .with_system(is_player_dead_system.system())
                .with_system(player_too_low_system.system())
                .with_system(mine_hook_system.system()),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::Game)
                .with_system(end_game_system.system())
                .with_system(setup_menu.system()),
        )
        .add_system_set(SystemSet::on_update(AppState::Menu).with_system(menu.system()))
        .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu.system()))
        .add_system(scoreboard_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system());
    // app.add_state(AppState::End);
    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    app.run();
}

fn start_music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_looped(asset_server.load("music.ogg"));
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<AppState>>,
) {
    // Add the game's entities to our world

    // cameras
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands.spawn_bundle(UiCameraBundle::default());

    // bg
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(GAME_BOARD.1 * 2. + 32., 1000.0)),
            ..Default::default()
        })
        .insert(Background);

    // line
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.9),
            sprite: Sprite::new(Vec2::new(1.0, 2.0)),
            ..Default::default()
        })
        .insert(Line);

    // player
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(asset_server.load("player.png").into()),
            sprite: Sprite::new(Vec2::new(32.0, 32.0)),
            ..Default::default()
        })
        .insert(Player::default());

    // scoreboard
    commands
        .spawn_bundle(TextBundle {
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
        })
        .insert(ScoreText);

    // proceed to game
    state.set(AppState::Game).unwrap();
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
        let camera_transform = q_camera.single().expect("Need exactly one camera");

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
                mat.color.set_g(4.);
            } else {
                mat.color.set_g(1.);
            }
            if m.hooked {
                mat.color.set_r(4.);
            } else {
                mat.color.set_r(1.);
            }
        }
    }
}

/// Mark a mine hooked

fn mine_hook_system(
    btns: Res<Input<MouseButton>>,
    mut q_mine: Query<&mut Mine>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>
) {
    if btns.just_pressed(MouseButton::Left) {
        // a left click just happened
        for mut m in &mut q_mine.iter_mut() {
            if m.selected {
                m.hooked = true;
                audio.play(asset_server.load("sfx100v2_air_02.ogg"));
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

        // if
        if highest_mine - p.maxheight < 150. {
            let mut rng = rand::thread_rng();

            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(asset_server.load("mine.png").into()),
                    sprite: Sprite::new(Vec2::new(32.0, 32.0)),
                    transform: Transform::from_xyz(
                        rng.gen_range(GAME_BOARD.0..GAME_BOARD.1),
                        rng.gen_range(p.maxheight + GAME_BOARD.3..p.maxheight + 450.),
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
                let grad = colorgrad::rainbow();
                let c = grad.at(p.maxheight as f64 / 10000.);

                mat.color.set_r(c.r as f32);
                mat.color.set_g(c.g as f32);
                mat.color.set_b(c.b as f32);
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
                player.velocity += dir.normalize() * UPWARD_FAC;
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
                mine.velocity += dir.normalize() * 0.04;
            } else {
                // slow down mine again if not hooked
                mine.velocity *= 0.9;
            }
        }
    }
}

fn draw_line_system(
    q_player: Query<&Transform, (With<Player>, Without<Mine>, Without<Line>)>,
    q_mines: Query<(&Transform, &Mine), (Without<Player>, Without<Line>)>,
    mut q_line: Query<&mut Transform, (With<Line>, Without<Player>, Without<Mine>)>,
) {
    let mut any_hooked = false;
    if let Ok(p_t) = q_player.single() {
        for (m_t, mine) in q_mines.iter() {
            if let Ok(mut l_t) = q_line.single_mut() {
                if mine.hooked {
                    let m = m_t.translation.truncate();
                    let p = p_t.translation.truncate();
                    l_t.translation = midpoint(m_t.translation, p_t.translation);
                    l_t.scale.x = dist(m, p);
                    let diff = m_t.translation - p_t.translation;
                    let angle = diff.y.atan2(diff.x);
                    l_t.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle);
                    any_hooked = true;
                }
            }
        }
        if !any_hooked {
            if let Ok(mut l_t) = q_line.single_mut() {
                l_t.scale.x = 0.;
            }
        }
    }
}

fn midpoint(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new((a.x + b.x) / 2., (a.y + b.y) / 2., (a.z + b.z) / 2.)
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

/// Trigger state change
fn is_player_dead_system(
    mut app_state: ResMut<State<AppState>>,
    mut player_query: Query<&mut Player>,
) {
    if let Ok(mut player) = player_query.single_mut() {
        if player.dead {
            player.maxheight = 0.;
            app_state.set(AppState::Menu).unwrap();
        }
    }
}

/// update the score
fn scoreboard_system(mut query: Query<&mut Text, With<ScoreText>>, player_query: Query<&Player>) {
    if let Ok(player) = player_query.single() {
        let mut text = query.single_mut().unwrap();
        text.sections[0].value = format!("Score: {:}", player.maxheight as i32);
    }
}

/// Very simple wall collision (left/right)
fn ball_collision_system(
    mut ball_query: Query<(&mut Player, &mut Transform)>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    if let Ok((mut player, mut p_t)) = ball_query.single_mut() {
        // check collision with walls and "reflect"
        if p_t.translation.x <= GAME_BOARD.0 || p_t.translation.x >= GAME_BOARD.1 {
            p_t.translation.x = p_t.translation.x.max(GAME_BOARD.0);
            p_t.translation.x = p_t.translation.x.min(GAME_BOARD.1);
            // reverse and dampen
            player.velocity.x *= -0.5;
            audio.play(asset_server.load("sfx100v2_metal_01.ogg"));
        }
    }
}

/// TODO: play / end looping sound when hooked, maybe pitch
fn play_hooked_system(p_query: Query<&Mine>, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    for m in p_query.iter() {
        if m.hooked {
            audio.play(asset_server.load("sfx100v2_air_02.ogg"));
        }
    }
}

/// clean up everything
fn end_game_system(
    mut mine_query: Query<Entity, With<Mine>>,
    player_query: Query<Entity, With<Player>>,
    cam_query: Query<Entity, With<MainCamera>>,
    mut commands: Commands,
) {
    // if let Ok(player) = player_query.single() {
    //     commands.entity(player).despawn();
    // }
    for mine in mine_query.iter_mut() {
        commands.entity(mine).despawn();
    }
    // if let Ok(cam) = cam_query.single() {
    //     commands.entity(cam).despawn();
    // }
}

/// clean up everything
fn init_game(
    mut player_query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = player_query.single_mut() {
        *transform = Transform::from_xyz(0.0, -160.0, 1.0);
        player.velocity = Vec3::new(0.5, 15.5, 0.0);
        player.maxheight = 0.0;
        player.dead = false;
    }
    // if let Ok(cam) = cam_query.single() {
    //     commands.entity(cam).despawn();
    // }
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    // ui camera
    // commands.spawn_bundle(UiCameraBundle::default());
    let button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Play",
                    TextStyle {
                        font: asset_server.load("vcr.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();
    commands.insert_resource(MenuData { button_entity });
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

/// Despawn all menu items
fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

fn menu(
    mut state: ResMut<State<AppState>>,
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
                state.set(AppState::Game).unwrap();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}
struct MenuData {
    button_entity: Entity,
}
