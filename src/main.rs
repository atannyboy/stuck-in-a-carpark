mod vehicle_struct;
mod game;
mod puzzle_generator;

use game::Game;

use piston::window::WindowSettings;
use piston_window::{PistonWindow, EventLoop, Window, AdvancedWindow};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{Events, EventSettings};
use piston::input::{RenderEvent, UpdateEvent, PressEvent, ReleaseEvent, MouseCursorEvent};
use piston::Button;
use piston::input::{MouseButton};

use crate::vehicle_struct::Vehicle;
use crate::puzzle_generator::PuzzleGenerator;
use crate::puzzle_generator::GRID_WIDTH;
use crate::puzzle_generator::GameManager;

const CELL_SIZE: f64 = 50.0;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Stuck in a Carpark", [(7 * CELL_SIZE as i32) as u32, (7 * CELL_SIZE as i32) as u32])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game::new();
    let mut puzzle_generator = PuzzleGenerator::new();
    let mut game_manager = GameManager::new(game);

    // Generate initial puzzle with vehicles
    let initial_vehicles = puzzle_generator.generate_puzzle(&mut game_manager.game);
    game_manager.update_vehicles(initial_vehicles);

    let mut gl = GlGraphics::new(opengl);
    let mut cursor_pos: [f64; 2] = [0.0, 0.0];

    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            game_manager.game.render(&args, &mut gl);
        }

        e.mouse_cursor(|pos| {
            cursor_pos = pos;
        });

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            game_manager.game.handle_mouse_click(cursor_pos[0], cursor_pos[1]);
            
            // Optionally, update the game state based on the result of the mouse click
            // For example, if clicking moves a vehicle or changes the game state
            // let new_vehicles = puzzle_generator.add_vehicle_strategically(&game_manager.game);
            // game_manager.update_vehicles(new_vehicles);
        }
    }
}

fn display_carpark(vehicles: &[Vehicle], grid: &[[Option<usize>; GRID_WIDTH as usize]; GRID_WIDTH as usize]) {
    let mut output = String::new();

    for y in 0..grid.len() {
        for x in 0..grid[0].len() {
            match grid[y][x] {
                Some(index) => {
                    let vehicle = &vehicles[index];
                    output.push_str(&format!("{}▓▓", vehicle.ansi_color.to_ansi())); // Color the vehicle
                },
                None => output.push_str("\x1b[90m░░"), // Grey for empty spaces
            }
        }
        output.push_str("\x1b[0m\n"); // Reset color and new line
    }

    println!("{}", output);
}