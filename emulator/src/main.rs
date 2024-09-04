use core::f32;
use std::borrow::BorrowMut;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle, Wireframe2dConfig, Wireframe2dPlugin},
};
use rand::{distributions::Bernoulli, distributions::Distribution, prelude::SliceRandom, Fill};
use rand::{thread_rng, Rng};

const RES_WIDTH: u16 = 64;
const RES_HEIGHT: u16 = 32;
const TOTAL_LEDS: usize = const { RES_WIDTH as usize * RES_HEIGHT as usize };
const CPU_WIDTH: u8 = 10;
const CPU_HEIGHT: u8 = 16;

#[derive(Component)]
struct LedMatrix<const S: usize> {
    /// A LED has a value between 0 and 255 which ought to be enough for future uses also.
    vals: [u8; S],
}

#[derive(Resource)]
struct Board {
    width: u16,
    height: u16,
    matrix: LedMatrix<TOTAL_LEDS>,
}

#[derive(Component)]
struct Location(usize);
#[derive(Resource)]
struct Syswrapper(sysinfo::System);

fn main() {
    let mut rng = thread_rng();
    let mut board = Board {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        matrix: LedMatrix {
            vals: [0; TOTAL_LEDS],
        },
    };
    Fill::try_fill(&mut board.matrix.vals, &mut rng);
    let system = Syswrapper(sysinfo::System::new());
    App::new()
        .insert_resource(board)
        .insert_resource(system)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RP2040 CPU display emulator".into(),
                    resolution: (RES_WIDTH as f32 * 20., RES_HEIGHT as f32 * 20.).into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Wireframe2dPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_wireframe, update_leds, draw_board))
        .run();
}

const X_EXTEND: f32 = 900.;

fn setup(
    mut commands: Commands,
    board: Res<Board>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Grid,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                grid_template_columns: vec![GridTrack::auto(); usize::from(board.width)],
                grid_template_rows: vec![GridTrack::auto(); usize::from(board.height)],
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..Default::default()
        })
        .with_children(|builder| {
            for c in 0..board.width {
                for r in 0..board.height {
                    let idx = usize::from(c + r * 64);
                    info!("c: {}, r: {}, idx: {}", c, r, idx);
                    let val = board.matrix.vals[idx];
                    let color = Color::rgb(val as f32 / 255., 0., 0.);
                    info!("color: {:?}", color);
                    builder.spawn((
                        NodeBundle {
                            style: Style {
                                border: UiRect::all(Val::Px(1.)),
                                display: Display::Grid,
                                width: Val::Px(20.),
                                height: Val::Px(20.),
                                ..Default::default()
                            },
                            background_color: BackgroundColor(color),
                            ..Default::default()
                        },
                        Location(idx),
                    ));
                }
            }
        });
}

fn update_leds(mut board: ResMut<Board>, mut system: ResMut<Syswrapper>) {
    let mut rng = thread_rng();
    let mut cpu_indexes = (0..12)
        .into_iter()
        .map(|i| generate_indexes(i))
        .collect::<Vec<_>>();
    system.0.refresh_cpu_usage();
    let mut cpu_usages = system.0.cpus().iter();
    let mut percentages = [0.; 12];
    for i in 0..percentages.len() {
        // We assume for now that threads on the same core are adjacent
        percentages[i] =
            (cpu_usages.next().unwrap().cpu_usage() + cpu_usages.next().unwrap().cpu_usage()) / 2.;
    }
    for i in 0..12 {
        let cpu_usage = percentages[i];
        let d = Bernoulli::new(cpu_usage as f64 / 100.).unwrap();
        for led in 0..cpu_indexes[i].clone().len() {
            let value = if d.sample(&mut rng) { 255 } else { 0 };
            trace!("Setting value: {} for led: {} on CPU: {}", value, led, i);
            board.matrix.vals[led] = value;
        }
    }
}

fn draw_board(mut query: Query<(&mut BackgroundColor, &Location)>, board: Res<Board>) {
    for (mut color, location) in &mut query {
        let val = board.matrix.vals.get(location.0).unwrap_or_else(|| {
            error!("Location: {} not found in board", location.0);
            &0
        });
        color.0 = Color::rgb(*val as f32 / 255., 0., 0.);
    }
}

#[derive(Component)]
struct Canvas;

fn toggle_wireframe(
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}



#[cfg(debug_assertions)]
mod tests {
    use crate::{generate_indexes, CPU_HEIGHT, CPU_WIDTH, RES_WIDTH};

    fn print_chunks(res: &[usize]) {
        for n in res.chunks(CPU_WIDTH as usize).into_iter() {
            println!("{n:?}");
        }
    }

    #[test]
    fn test_generate_indexes() {
        let res = generate_indexes(0);
        assert_eq!(res.clone()[0], 0);
        print_chunks(&res);

        let cpu_6 = generate_indexes(6);
        print_chunks(&cpu_6);
        for i in 1..6 {
            assert_eq!(
                res.clone()
                    .into_iter()
                    .map(|v| v + (CPU_WIDTH as usize) * i)
                    .collect::<Vec<usize>>(),
                generate_indexes(i)
            );
            assert_eq!(
                cpu_6
                    .clone()
                    .into_iter()
                    .map(|v| v + (CPU_WIDTH as usize) * i)
                    .collect::<Vec<usize>>(),
                generate_indexes(i + 6)
            );
        }
    }
}
