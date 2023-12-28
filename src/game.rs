use crate::vehicle_struct::{Vehicle, Orientation};

use crate::GlGraphics;

use crate::CELL_SIZE;

use crate::puzzle_generator::GRID_WIDTH;
use crate::puzzle_generator::GRID_HEIGHT;

pub struct Game {
    pub vehicles: Vec<Vehicle>,
    pub grid: [[Option<usize>; 7]; 7],
    pub selected_vehicle_index: Option<usize>,
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

    pub fn is_occupied_by_vehicle(&self, x: usize, y: usize, vehicle_index: usize) -> bool {
        match self.grid[y][x] {
            Some(id) => id == vehicle_index,
            None => false,
        }
    }

    pub fn vehicle_at_position(&self, x: f64, y: f64) -> Option<usize> {
        let grid_x = (x / CELL_SIZE).floor() as u8;
        let grid_y = (y / CELL_SIZE).floor() as u8;

        for (index, vehicle) in self.vehicles.iter().enumerate() {
            let (vehicle_x, vehicle_y) = vehicle.position;
            if vehicle.orientation == Orientation::Horizontal && vehicle_y == grid_y && grid_x >= vehicle_x && grid_x < vehicle_x + vehicle.size.0 as u8 {
                return Some(index);
            } else if vehicle.orientation == Orientation::Vertical && vehicle_x == grid_x && grid_y >= vehicle_y && grid_y < vehicle_y + vehicle.size.1 as u8 {
                return Some(index);
            }
        }

        None
    }

    pub fn is_path_clear(&self, vehicle_index: usize, new_x: u8, new_y: u8) -> bool {
        let vehicle = &self.vehicles[vehicle_index];
    
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
                    if let Some(other_vehicle_index) = self.grid[vehicle.position.1 as usize][x as usize] {
                        let other_vehicle = &self.vehicles[other_vehicle_index];
                        if other_vehicle.position != vehicle.position {
                            return false;
                        }
                    }
                }
            }
            Orientation::Vertical => {
                for y in start_y..=end_y {
                    if let Some(other_vehicle_index) = self.grid[y as usize][vehicle.position.0 as usize] {
                        let other_vehicle = &self.vehicles[other_vehicle_index];
                        if other_vehicle.position != vehicle.position {
                            return false;
                        }
                    }
                }
            }
        }
    
        true
    }    

    pub fn handle_mouse_click(&mut self, x: f64, y: f64) {
        let grid_x = (x / CELL_SIZE).floor() as u8;
        let grid_y = (y / CELL_SIZE).floor() as u8;
    
        if let Some(vehicle_index) = self.selected_vehicle_index {
            let vehicle = &self.vehicles[vehicle_index];
    
            if self.grid[grid_y as usize][grid_x as usize].is_none() {
                let new_position = match vehicle.orientation {
                    Orientation::Horizontal if grid_y == vehicle.position.1 => {
                        if grid_x < vehicle.position.0 {
                            (grid_x, vehicle.position.1)
                        } else {
                            (grid_x - vehicle.size.0 as u8 + 1, vehicle.position.1)
                        }
                    }
                    Orientation::Vertical if grid_x == vehicle.position.0 => {
                        if grid_y < vehicle.position.1 {
                            (vehicle.position.0, grid_y)
                        } else {
                            (vehicle.position.0, grid_y - vehicle.size.1 as u8 + 1)
                        }
                    }
                    _ => (vehicle.position.0, vehicle.position.1),
                };
    
                if new_position.0 < 7 && new_position.1 < 7 && self.is_path_clear(vehicle_index, new_position.0, new_position.1) {
                    self.move_vehicle(vehicle_index, new_position.0, new_position.1);
                }
            }
    
            // Check if the clicked position has another vehicle to select
            if let Some(new_vehicle_index) = self.vehicle_at_position(x, y) {
                if new_vehicle_index != vehicle_index {
                    self.selected_vehicle_index = Some(new_vehicle_index);
                }
            }
        } else {
            self.selected_vehicle_index = self.vehicle_at_position(x, y);
        }
    }

    pub fn check_win_condition(&self) -> bool {
        let red_vehicle = self.vehicles.iter().find(|c| c.color == [1.0, 0.0, 0.0, 1.0]).unwrap();
        red_vehicle.position.0 == 3 && red_vehicle.position.1 == 0
    }

    pub fn move_vehicle(&mut self, vehicle_index: usize, new_x: u8, new_y: u8) {
        let vehicle = self.vehicles[vehicle_index];
        
        if self.is_path_clear(vehicle_index, new_x, new_y) {
            // Clear old positions
            let (old_x, old_y) = (vehicle.position.0 as usize, vehicle.position.1 as usize);
            
            if vehicle.orientation == Orientation::Horizontal {
                for i in 0..vehicle.size.0 {
                    self.grid[old_y as usize][((old_x + i as usize) as usize) as usize] = None;
                }
            } else {
                for i in 0..vehicle.size.1 {
                    self.grid[((old_y + i as usize) as usize) as usize][old_x as usize] = None;
                }
            }

            // Update the vehicle's position in the vehicles vector
            self.vehicles[vehicle_index].position = (new_x, new_y);

            // Update new positions in the grid
            let (x, y) = (new_x as usize, new_y as usize);

            if vehicle.orientation == Orientation::Horizontal {
                for i in 1..vehicle.size.0 {
                    self.grid[y as usize][x + i as usize] = Some(vehicle_index);
                }
            } else {
                for i in 1..vehicle.size.1 {
                    self.grid[y + i as usize][x as usize] = Some(vehicle_index);
                }
            }

            println!("Vehicle moved to new position: {}, {}", new_x, new_y);

            // Check for win condition after the move
            if self.check_win_condition() {
                println!("You've won the game!");
            }
        } else {
            println!("Move is not valid. Another vehicle is in the way.");
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
}