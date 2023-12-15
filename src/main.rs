mod vehicle_struct;
mod game;
mod puzzle_generator;

use game::Game;

use piston::window::WindowSettings;
use piston_window::PistonWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::{RenderEvent, PressEvent, MouseCursorEvent};
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
    /*game_manager.update_vehicles(initial_vehicles.clone());*/
    let vehicles_count = initial_vehicles.len();
    game_manager.game.vehicles = initial_vehicles.clone();
    
    println!("Setting vehicles. Vehicle count: {}", vehicles_count);
    game_manager.game.update_grid(); // Update grid every time vehicles are set

    let grid_clone = game_manager.game.grid.clone();
    game_manager.game.display_carpark(&initial_vehicles, &grid_clone);

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