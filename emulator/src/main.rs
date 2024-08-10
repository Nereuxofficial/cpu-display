use core::f32;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle, Wireframe2dConfig, Wireframe2dPlugin},
};
use rand::{prelude::SliceRandom, Fill};
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
struct Location {
    x: u16,
    y: u16,
}

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
    App::new()
        .insert_resource(board)
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
                for r in 0..board.width {
                    let val = board.matrix.vals[usize::from(c + r)];
                    info!("val: {}", val);
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
                        Location { x: c, y: r },
                    ));
                }
            }
        });
}

fn update_leds(mut board: ResMut<Board>) {
    let mut rng = thread_rng();
    Fill::try_fill(&mut board.matrix.vals, &mut rng);
}

fn draw_board(mut query: Query<(&mut BackgroundColor, &Location)>, board: Res<Board>) {
    for (mut color, bundle) in &mut query {
        let val = board.matrix.vals[usize::from(bundle.x + bundle.y)];
        color.0 = Color::rgb(val as f32 / 255., 0., 0.);
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

// If we assume 10x16 LEDs represent a single CPU core. We have 12 cores so we need to get the LED slice for each one.
// Given our 64*32 LED matrix we leave the last four rows for 60*32 LEDs used.
/// Generate the LED indexes of the specific CPU
fn generate_indexes(cpu_index: usize) -> Vec<usize> {
    let mut indexes = Vec::new();
    let mut idx = if cpu_index < 6 { 0 } else { TOTAL_LEDS / 2 };
    let rel_cpu = cpu_index % 6;
    loop {
        let min_x = (CPU_WIDTH as usize) * rel_cpu;
        let max_x = min_x + CPU_WIDTH as usize;
        if (min_x..max_x).contains(&(idx % (RES_WIDTH as usize))) {
            indexes.push(idx);
            if indexes.len() as u8 == CPU_WIDTH * CPU_HEIGHT {
                break;
            }
        }
        idx += 1;
    }
    indexes
}

#[cfg(debug_assertions)]
mod tests {
    use crate::{generate_indexes, CPU_HEIGHT, CPU_WIDTH, RES_WIDTH};

    fn print_chunks(res: &Vec<usize>) {
        for n in res.chunks(CPU_WIDTH as usize).into_iter() {
            println!("{n:?}");
        }
    }

    #[test]
    fn test_generate_indexes() {
        let res = generate_indexes(0);
        assert_eq!((CPU_WIDTH * CPU_HEIGHT) as usize, res.len());
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
