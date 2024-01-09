use crate::vehicle_struct::{Vehicle, Orientation};

use crate::GlGraphics;

use crate::CELL_SIZE;

use crate::PuzzleGenerator;
use crate::puzzle_generator::GRID_WIDTH;
use crate::puzzle_generator::GRID_HEIGHT;
use crate::vehicle_struct::Move;

pub struct Game {
    pub vehicles: Vec<Vehicle>,
    pub grid: [[Option<usize>; 7]; 7],
    pub selected_vehicle_index: Option<usize>,

    // === start of movement code ===
    pub puzzle_generation_moves: Vec<Move>, // To store both placements and movements during puzzle generation
    pub move_history: Vec<Move>, // Add this line to store the move history
    // === end of movement code ===
}

// Use 'pub' to make the methods accessible from outside this module.
impl Game {
    pub fn new() -> Self {
        if GRID_WIDTH != 7 || GRID_HEIGHT != 7 {
            panic!("Grid dimensions are incorrect. Expected 7x7, found {}x{}", GRID_WIDTH, GRID_HEIGHT);
        }

        Game { 
            vehicles: Vec::new(), 
            grid: [[None; GRID_WIDTH as usize]; GRID_WIDTH as usize], 
            selected_vehicle_index: None,
            // === start of movement code ===
            puzzle_generation_moves: Vec::new(),
            move_history: Vec::new(),
            // === end of movement code ===
        }
    }
    
    // Method to update the grid with a new vehicle
    pub fn update_grid_with_new_vehicle(&mut self, vehicle: &Vehicle, vehicle_id: usize) {
        let (x, y) = (vehicle.position.0 as usize, vehicle.position.1 as usize);
        println!("Updating grid with new vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", vehicle_id, vehicle.size, vehicle.position, vehicle.orientation);
        match vehicle.orientation {
            Orientation::Horizontal => {
                for i in 0..vehicle.size.0 as usize {
                    if x + i < GRID_WIDTH {
                        self.grid[y][x + i] = Some(vehicle_id); // Assign vehicle's ID
                    }
                }
            },
            Orientation::Vertical => {
                for i in 0..vehicle.size.1 as usize {
                    if y + i < GRID_HEIGHT {
                        self.grid[y + i][x] = Some(vehicle_id); // Assign vehicle's ID
                    }
                }
            },
        }
    }

    pub fn is_position_empty(&self, x: usize, y: usize) -> bool {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            false // If the index is out of bounds, return false
        } else {
            self.grid[y][x].is_none()
        }
    }

    /*pub fn is_occupied_by_vehicle(&self, x: usize, y: usize, vehicle_index: usize) -> bool {
        match self.grid[y][x] {
            Some(id) => id == vehicle_index,
            None => false,
        }
    }*/

    pub fn vehicle_at_position(&self, grid_x: u8, grid_y: u8) -> Option<usize> {
        println!("Clicked position: grid_x: {}, grid_y: {}", grid_x, grid_y);
    
        for (index, vehicle) in self.vehicles.iter().enumerate() {
            let (vehicle_x, vehicle_y) = vehicle.position;
            if vehicle.orientation == Orientation::Horizontal && vehicle_y == grid_y && grid_x >= vehicle_x && grid_x < vehicle_x + vehicle.size.0 as u8 {
                println!("Vehicle found at clicked position: Vehicle Index: {}", index);
                return Some(index);
            } else if vehicle.orientation == Orientation::Vertical && vehicle_x == grid_x && grid_y >= vehicle_y && grid_y < vehicle_y + vehicle.size.1 as u8 {
                println!("Vehicle found at clicked position: Vehicle Index: {}", index);
                return Some(index);
            }
        }
    
        println!("No vehicle at clicked position");
        None
    }    

    pub fn is_path_clear(&self, vehicle_index: usize, new_x: u8, new_y: u8) -> bool {
        let vehicle = &self.vehicles[vehicle_index];
    
        // Debug: print initial check details
        println!("Checking path for vehicle index {}: Current position: ({}, {}), New position: ({}, {})", 
                 vehicle_index, vehicle.position.0, vehicle.position.1, new_x, new_y);
    
        let (start_x, end_x) = if new_x > vehicle.position.0 {
            (vehicle.position.0, new_x)
        } else {
            (new_x, vehicle.position.0)
        };
    
        let (start_y, end_y) = if new_y > vehicle.position.1 {
            (vehicle.position.1, new_y)
        } else {
            (new_y, vehicle.position.1)
        };
    
        match vehicle.orientation {
            Orientation::Horizontal => {
                for x in start_x..=end_x {
                    if self.grid[vehicle.position.1 as usize][x as usize].is_some() && self.grid[vehicle.position.1 as usize][x as usize] != Some(vehicle_index) {
                        // Debug: print information when path is blocked
                        println!("Path blocked for horizontal movement at grid position ({}, {})", x, vehicle.position.1);
                        return false;
                    }
                }
            }
            Orientation::Vertical => {
                for y in start_y..=end_y {
                    if self.grid[y as usize][vehicle.position.0 as usize].is_some() && self.grid[y as usize][vehicle.position.0 as usize] != Some(vehicle_index) {
                        // Debug: print information when path is blocked
                        println!("Path blocked for vertical movement at grid position ({}, {})", vehicle.position.0, y);
                        return false;
                    }
                }
            }
        }
    
        // Debug: Path is clear
        println!("Path is clear for vehicle index {} to move to ({}, {})", vehicle_index, new_x, new_y);
        true
    }    

    pub fn handle_mouse_click(&mut self, x: f64, y: f64) {
        println!("handle_mouse_click called with x: {}, y: {}", x, y);
    
        let grid_x = (x / CELL_SIZE).floor() as u8;
        let grid_y = (y / CELL_SIZE).floor() as u8;
        println!("Converted to grid coordinates: grid_x: {}, grid_y: {}", grid_x, grid_y);
    
        println!("Currently selected vehicle index before click: {:?}", self.selected_vehicle_index);
    
        match self.vehicle_at_position(grid_x, grid_y) {
            Some(new_vehicle_index) if Some(new_vehicle_index) != self.selected_vehicle_index => {
                println!("New vehicle selected at clicked position: Index {}", new_vehicle_index);
                self.selected_vehicle_index = Some(new_vehicle_index);
            }
            _ => {
                if let Some(selected_index) = self.selected_vehicle_index {
                    println!("Attempting to move currently selected vehicle at index: {}", selected_index);
                    self.attempt_to_move_vehicle(selected_index, grid_x, grid_y);
                }
            }
        }
    
        println!("Currently selected vehicle index after click: {:?}", self.selected_vehicle_index);
        println!("");
    }
    
    fn attempt_to_move_vehicle(&mut self, vehicle_index: usize, grid_x: u8, grid_y: u8) {
        let vehicle = &self.vehicles[vehicle_index];
    
        // Debug: Print the current vehicle's details
        println!("Attempting to move vehicle at index {}. Current position: ({}, {}), Orientation: {:?}", 
                 vehicle_index, vehicle.position.0, vehicle.position.1, vehicle.orientation);
    
        let new_position = match vehicle.orientation {
            Orientation::Horizontal if grid_y == vehicle.position.1 => {
                let new_x = if grid_x > vehicle.position.0 {
                    grid_x - (vehicle.size.0 as u8 - 1)
                } else {
                    grid_x
                };
                // Debug: Print new calculated position for horizontal orientation
                println!("Calculated new horizontal position: ({}, {})", new_x, vehicle.position.1);
                (new_x, vehicle.position.1)
            },
            Orientation::Vertical if grid_x == vehicle.position.0 => {
                let new_y = if grid_y > vehicle.position.1 {
                    grid_y - (vehicle.size.1 as u8 - 1)
                } else {
                    grid_y
                };
                // Debug: Print new calculated position for vertical orientation
                println!("Calculated new vertical position: ({}, {})", vehicle.position.0, new_y);
                (vehicle.position.0, new_y)
            },
            _ => {
                // Debug: Vehicle orientation not matching or invalid click
                println!("No movement required or invalid click. Vehicle remains at position: ({}, {})", vehicle.position.0, vehicle.position.1);
                vehicle.position
            }
        };
    
        if new_position != vehicle.position && self.is_path_clear(vehicle_index, new_position.0, new_position.1) {
            // Debug: Path is clear, proceeding with move
            println!("Path is clear. Moving vehicle from ({}, {}) to ({}, {})", 
                     vehicle.position.0, vehicle.position.1, new_position.0, new_position.1);
            self.move_vehicle(vehicle_index, new_position.0, new_position.1);
            //self.selected_vehicle_index = None; // Deselect vehicle after moving
        } else {
            // Debug: Path is not clear or no movement is required
            println!("Move not possible or no movement required. Vehicle remains at position: ({}, {})", vehicle.position.0, vehicle.position.1);
        }
    }    

    pub fn check_win_condition(&self) -> bool {
        let red_vehicle = self.vehicles.iter().find(|c| c.color == [1.0, 0.0, 0.0, 1.0]).unwrap();
        red_vehicle.position.0 == 3 && red_vehicle.position.1 == 0
    }

    pub fn move_vehicle(&mut self, vehicle_index: usize, new_x: u8, new_y: u8) {
        println!("Attempting to move vehicle. Index: {}, New Position: ({}, {})", vehicle_index, new_x, new_y); // Debug statement
    
        let vehicle = self.vehicles[vehicle_index];
        println!("Vehicle's current position: ({}, {})", vehicle.position.0, vehicle.position.1); // Debug statement
    
        if self.is_path_clear(vehicle_index, new_x, new_y) {
            // Clear the old position of the vehicle on the grid before updating its position
            self.clear_vehicle_position((vehicle.position.0 as usize, vehicle.position.1 as usize), vehicle.size, vehicle.orientation);
    
            // Update the vehicle's position in the vehicles vector
            self.vehicles[vehicle_index].position = (new_x, new_y);
    
            // Set the new position of the vehicle on the grid
            self.update_vehicle_position(vehicle_index, new_x, new_y, vehicle.orientation);
    
            println!("Vehicle moved to new position: {}, {}", new_x, new_y);
    
            if self.check_win_condition() {
                println!("You've won the game!");
            }
    
            println!("Vehicle moved to new position: {}, {}", new_x, new_y); // Debug statement
        } else {
            println!("Move is not valid. Another vehicle is in the way."); // Debug statement
        }
    }

    pub fn render(&mut self, args: &piston::input::RenderArgs, gl: &mut GlGraphics) {
        use graphics::*;

        gl.draw(args.viewport(), |c, gl| {
            // Clear the screen
            clear([1.0; 4], gl);

            for (i, row) in self.grid.iter().enumerate() {
                for (j, cell) in row.iter().enumerate() {
                    let x = (j as f64) * CELL_SIZE;
                    let y = (i as f64) * CELL_SIZE;

                    // Draw the grid cell
                    rectangle([0.8, 0.8, 0.8, 1.0], // light gray color
                              [x, y, CELL_SIZE, CELL_SIZE],
                              c.transform,
                              gl);

                    // If there's a vehicle in this cell, draw it
                    if let Some(vehicle_index) = cell {
                        let vehicle = &self.vehicles[*vehicle_index];   // Dereference the index to get the vehicle

                        let vehicle_x = (vehicle.position.0 as f64) * CELL_SIZE;
                        let vehicle_y = (vehicle.position.1 as f64) * CELL_SIZE;
                        
                        let vehicle_width = if vehicle.orientation == Orientation::Horizontal {
                            CELL_SIZE * vehicle.size.0 as f64
                        } else {
                            CELL_SIZE
                        };
                        let vehicle_height = if vehicle.orientation == Orientation::Vertical {
                            CELL_SIZE * vehicle.size.1 as f64
                        } else {
                            CELL_SIZE
                        };

                        rectangle(vehicle.color,
                                  [vehicle_x, vehicle_y, vehicle_width, vehicle_height],
                                  c.transform,
                                  gl);
                    }
                }
            }
        });
    }

    pub fn display_carpark(&mut self, vehicles: &[Vehicle], grid: &[[Option<usize>; GRID_WIDTH as usize]; GRID_WIDTH as usize]) {
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

    // === start of movement code ===

    // Method to record a vehicle placement during puzzle generation
    pub fn record_placement(&mut self, vehicle_index: usize, position: (usize, usize)) {
        let placement_move = Move {
            vehicle_index,
            move_type: MoveType::Placement,
            distance: 0, // Set to 0 or a default value for placements
            position_x: Some(position.0 as isize),
            position_y: Some(position.1 as isize),
            new_position_x: None,
            new_position_y: None,
        };
        self.record_puzzle_generation_move(placement_move);
    }

    // Method to record a vehicle movement during puzzle generation
    pub fn record_movement(&mut self, vehicle_index: usize, distance: isize) {
        let movement_move = Move {
            vehicle_index,
            move_type: MoveType::Movement,
            distance,
            position_x: None,
            position_y: None,
            new_position_x: None,
            new_position_y: None,
        };
        self.record_puzzle_generation_move(movement_move);
    }

    // Method to record a move during puzzle generation
    pub fn record_puzzle_generation_move(&mut self, game_move: Move) {
        self.puzzle_generation_moves.push(game_move);
    }

    // Method to apply a move to the game state
    pub fn apply_move(&mut self, game_move: Move) {
        if let Some(vehicle) = self.vehicles.get(game_move.vehicle_index) {
            let old_position = (vehicle.position.0 as usize, vehicle.position.1 as usize);
            let vehicle_size = vehicle.size;
            let vehicle_orientation = vehicle.orientation;
            let vehicle_id = vehicle.id;
    
            let new_position = self.calculate_new_position(&vehicle_orientation, vehicle_size, old_position, game_move.distance);
    
            // Separate the mutable borrow of self.vehicles
            self.clear_vehicle_position(old_position, vehicle_size, vehicle_orientation);
            if let Some(vehicle) = self.vehicles.get_mut(game_move.vehicle_index) {
                vehicle.position = (new_position.0 as u8, new_position.1 as u8);
            }
            self.update_vehicle_position(vehicle_id, new_position.0 as u8, new_position.1 as u8, /*vehicle_size,*/ vehicle_orientation);
        }
    }    

    fn calculate_new_position(&self, orientation: &Orientation, size: (u8, u8), old_position: (usize, usize), distance: isize) -> (usize, usize) {
        let (old_x, old_y) = old_position;
    
        let new_x = match orientation {
            Orientation::Horizontal => {
                let temp_x = old_x as isize + distance;
                temp_x.clamp(0, GRID_WIDTH as isize - size.0 as isize) as usize
            },
            _ => old_x,
        };
    
        let new_y = match orientation {
            Orientation::Vertical => {
                let temp_y = old_y as isize + distance;
                temp_y.clamp(0, GRID_HEIGHT as isize - size.1 as isize) as usize
            },
            _ => old_y,
        };
    
        (new_x, new_y)
    } 

    fn clear_vehicle_position(&mut self, position: (usize, usize), size: (u8, u8), orientation: Orientation) {
        match orientation {
            Orientation::Horizontal => {
                // Debug: Print the range of horizontal positions being cleared
                println!("Clearing horizontal positions from ({}, {}) to ({}, {})", position.0, position.1, position.0 + size.0 as usize - 1, position.1);
    
                // Clear all horizontal positions occupied by the vehicle
                for i in 0..size.0 as usize {
                    if position.0 + i < GRID_WIDTH {
                        // Debug: Print each position being cleared
                        println!("Clearing position: ({}, {})", position.0 + i, position.1);
                        self.grid[position.1][position.0 + i] = None;
                    }
                }
            },
            Orientation::Vertical => {
                // Debug: Print the range of vertical positions being cleared
                println!("Clearing vertical positions from ({}, {}) to ({}, {})", position.0, position.1, position.0, position.1 + size.1 as usize - 1);
    
                // Clear all vertical positions occupied by the vehicle
                for i in 0..size.1 as usize {
                    if position.1 + i < GRID_HEIGHT {
                        // Debug: Print each position being cleared
                        println!("Clearing position: ({}, {})", position.0, position.1 + i);
                        self.grid[position.1 + i][position.0] = None;
                    }
                }
            },
        }
    }    
    
    fn update_vehicle_position(&mut self, vehicle_index: usize, new_x: u8, new_y: u8, orientation: Orientation) {
        match orientation {
            Orientation::Horizontal => {
                for i in 0..self.vehicles[vehicle_index].size.0 as usize {
                    if new_x as usize + i < GRID_WIDTH {
                        self.grid[new_y as usize][new_x as usize + i] = Some(vehicle_index);
                    }
                }
            },
            Orientation::Vertical => {
                for i in 0..self.vehicles[vehicle_index].size.1 as usize {
                    if new_y as usize + i < GRID_HEIGHT {
                        self.grid[new_y as usize + i][new_x as usize] = Some(vehicle_index);
                    }
                }
            },
        }
    } 
    
    /*fn update_grid_after_move(&mut self, vehicle_index: usize, new_x: u8, new_y: u8) {
        // Clear the old position of the vehicle on the grid
        let vehicle = self.vehicles[vehicle_index];
        /*self.clear_vehicle_position((vehicle.position.0 as usize, vehicle.position.1 as usize), vehicle.size, vehicle.orientation);*/
    
        // Set the new position of the vehicle on the grid
        match vehicle.orientation {
            Orientation::Horizontal => {
                for i in 0..vehicle.size.0 as usize {
                    if new_x as usize + i < GRID_WIDTH {
                        self.grid[new_y as usize][new_x as usize + i] = Some(vehicle_index);
                    }
                }
            },
            Orientation::Vertical => {
                for i in 0..vehicle.size.1 as usize {
                    if new_y as usize + i < GRID_HEIGHT {
                        self.grid[new_y as usize + i][new_x as usize] = Some(vehicle_index);
                    }
                }
            },
        }
    }*/

    // Method to undo the last move
    pub fn undo_last_move(&mut self) {
        if let Some(last_move) = self.move_history.pop() {
            // Code to reverse the last move
            // This might involve moving the vehicle back by the inverse of the last move's distance
        }
    }

    // Method to calculate the total complexity of a series of moves
    pub fn calculate_total_complexity(&self) -> usize {
        self.move_history.iter().map(|game_move| {
            // Complexity calculation logic per move
            // Example: game_move.distance as usize
            game_move.distance as usize // Example
        }).sum()
    }

    // === end of movement code ===
}

// === start of movement code ===

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MoveType {
    Placement, // For initial placement of vehicles
    Movement,  // For subsequent movements of vehicles
}

// === end of movement code ===