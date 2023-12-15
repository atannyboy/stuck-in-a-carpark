use crate::Game;
use crate::vehicle_struct::{VehicleStruct, Vehicle, Orientation, AnsiColorCode};
use rand::Rng;
use rand::prelude::SliceRandom;
use std::collections::HashSet;

pub const GRID_WIDTH: usize = 7;
pub const GRID_HEIGHT: usize = 7;

pub struct GameManager {
    pub game: Game,
    // other fields as needed
}

impl GameManager {
    pub fn new(game: Game) -> Self {
        GameManager { game }
    }

    /*pub fn update_vehicles(&mut self, vehicles: Vec<Vehicle>) {
        self.game.set_vehicles(vehicles);
        // other state updates as needed
    }*/

    // other methods to manage game state
}

pub struct PuzzleGenerator {
	complexity_measure: ComplexityMeasure,
	pub vehicle_struct: VehicleStruct,
	used_colors: HashSet<String>, // Using a HashSet to efficiently track used colors
}

impl PuzzleGenerator {
	pub fn new() -> Self {
		let complexity_measure = ComplexityMeasure::new();
		let vehicle_struct = VehicleStruct::new();
		let used_colors = HashSet::new();
		PuzzleGenerator { complexity_measure, vehicle_struct, used_colors }
	}

    // Updated to accept a reference to Game
    pub fn generate_puzzle(&mut self, game: &mut Game) -> Vec<Vehicle> {
        self.place_red_car();

        let mut vehicle_id: usize = 0;
        while !self.puzzle_generated(game) {
            vehicle_id += 1;

            // Call add_vehicle_strategically with a reference to game
            self.add_vehicle_strategically(game, vehicle_id);
            self.move_vehicles_strategically(game);
            self.verify_solvability(game);
        }
        self.vehicle_struct.vehicles.clone()
    }

	fn place_red_car(&mut self) {
		// Define the red car characteristics, assuming:
		// - ID: 0
		// - Size: (2, 1), meaning it occupies two horizontal grid spaces
		// - Color: Red (not specified here but should be defined in the Vehicle struct)
		// - Orientation: Horizontal
		// - Position: (5, 4), with the origin (0, 0) at the top-left of the grid

		let red_car = Vehicle {
			id: 0,
			size: (2, 1), // Assuming the car is 2 units long and 1 unit wide
			color: AnsiColorCode::Red.to_rgba(), // This would convert a color code to an RGBA value
			orientation: Orientation::Horizontal,
			position: (5, 3), // The position you specified
			ansi_color: AnsiColorCode::Red,
		};

		// Place the red car at the exit
		// Assuming 'game' is a mutable reference to the Game struct and has a method to add vehicles
		self.vehicle_struct.add_vehicle(red_car);
	}

    pub fn add_vehicle_strategically(&mut self, game: &mut Game, vehicle_id: usize) -> Vec<Vehicle> {
        loop {
            let vehicle_size = self.generate_random_vehicle_size();
            let random_orientation = self.generate_random_orientation();
            let mut possible_positions: Vec<((usize, usize), Orientation)> = match (vehicle_size, random_orientation) {
                ((1, 2), Orientation::Vertical) => self.generate_possible_positions_1x2(game, vehicle_size, Orientation::Vertical),
                ((2, 1), Orientation::Horizontal) => self.generate_possible_positions_1x2(game, vehicle_size, Orientation::Horizontal),
                ((1, 3), Orientation::Vertical) => self.generate_possible_positions_1x3(game, vehicle_size, Orientation::Vertical),
                ((3, 1), Orientation::Horizontal) => self.generate_possible_positions_1x3(game, vehicle_size, Orientation::Horizontal),
                _ => Vec::new(),
            };

            let mut best_position: Option<(usize, usize)> = None;
            let mut best_orientation = Orientation::Horizontal;
            let mut best_complexity = 0;
            

            for (position, orientation) in &possible_positions {
                if game.is_position_empty(position.0, position.1) {
                    let complexity = self.complexity_measure.calculate(game, *position, vehicle_size, *orientation);
                    if complexity > best_complexity {
                        best_complexity = complexity;
                        best_position = Some(*position);
                        best_orientation = *orientation;
                    }
                }
            }

            if let Some(position) = best_position {
                let vehicle = self.generate_vehicle(position, best_orientation);
                if self.vehicle_struct.add_vehicle(vehicle) {
                    println!("Successfully added vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", vehicle.id, vehicle.size, position, best_orientation);
                    // Update the game's grid to reflect the new vehicle
                    game.update_grid_with_new_vehicle(&vehicle);
                    break;
                } else {
                    println!("Failed to add vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", vehicle.id, vehicle.size, position, best_orientation);
                    // Remove the failed position from possible_positions and retry
                    possible_positions.retain(|&(p, o)| p != position || o != best_orientation);
                }
            } else {
                println!("No suitable position found for vehicle ID: {}", vehicle_id);
                break;
            }
        }
        
        self.vehicle_struct.vehicles.clone() // Return the updated list of vehicles
    }
	
	fn generate_vehicle(&mut self, position: (usize, usize), orientation: Orientation) -> Vehicle {
		let mut rng = rand::thread_rng();
	
		let color_options: Vec<AnsiColorCode> = vec![
			AnsiColorCode::Red,
			AnsiColorCode::Green,
            AnsiColorCode::Blue,
            AnsiColorCode::Cyan,
            AnsiColorCode::Magenta,
            AnsiColorCode::Yellow,
            AnsiColorCode::BrightBlack,
            AnsiColorCode::BrightWhite,
            AnsiColorCode::BrightCyan,
            AnsiColorCode::BrightMagenta,
            AnsiColorCode::BrightYellow,
			// ... other colors ...
		];
	
		// Ensure there's at least one color available
		if self.used_colors.len() == color_options.len() {
			self.used_colors.clear();
		}
	
		let available_colors: Vec<_> = color_options
			.into_iter()
			.filter(|color_code| !self.used_colors.contains(&color_code.to_string()))
			.collect();
	
		let ansi_color = available_colors[rng.gen_range(0..available_colors.len())];

        // Convert ANSI color code to RGBA
        let rgba_color = ansi_color.to_rgba();

        // Add the used color name to the HashSet of used colors
        self.used_colors.insert(ansi_color.to_string());

        // Randomly generate other attributes
        let id = 0;
        let sizes = match orientation {
            Orientation::Horizontal => [(2, 1), (3, 1)],
            Orientation::Vertical => [(1, 2), (1, 3)],
        };
        let size = sizes[rng.gen_range(0..sizes.len())];

        // Create and return the vehicle using the RGBA color and the provided position
        let vehicle = Vehicle::new(id, rgba_color, size, (position.0 as u8, position.1 as u8), orientation, ansi_color);
		println!("Generated vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", id, size, position, orientation);
		
    	vehicle
    }

	// Checks if two vehicles overlap
	pub fn check_for_overlap(vehicle_a: &Vehicle, vehicle_b: &Vehicle) -> bool {
		let positions_a: HashSet<(u8, u8)> = vehicle_a.occupied_positions().into_iter().collect();
		let positions_b: HashSet<(u8, u8)> = vehicle_b.occupied_positions().into_iter().collect();

		// Use is_disjoint on HashSet
		!positions_a.is_disjoint(&positions_b)
	}

	pub fn generate_random_vehicle_size(&self) -> (u8, u8) {
        let mut rng = rand::thread_rng();
        let sizes = [(1, 2), (2, 1), (1, 3), (3, 1)]; // The four size options
        let size_index = rng.gen_range(0..sizes.len()); // Randomly select an index
        sizes[size_index]  // Return the size
    }

    pub fn generate_random_orientation(&self) -> Orientation {
        let mut rng = rand::thread_rng();
        let orientations = [Orientation::Vertical, Orientation::Horizontal];
        *orientations.choose(&mut rng).expect("Array is non-empty")
    }

    fn generate_possible_positions_1x2(&self, game: &mut Game, vehicle_size: (u8, u8), orientation: Orientation) -> Vec<((usize, usize), Orientation)> {
        let mut all_positions = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                // Check if the vehicle fits in the grid based on its orientation
                if (orientation == Orientation::Horizontal && x + 1 < GRID_WIDTH) ||
                   (orientation == Orientation::Vertical && y + 1 < GRID_HEIGHT) {
                    // Check if starting position is suitable for placement
                    if self.can_place_vehicle(game, x, y, orientation, 2) {
                        all_positions.push(((x, y), orientation));
                    }
                }
            }
        }
        all_positions
    }

    fn generate_possible_positions_1x3(&self, game: &mut Game, vehicle_size: (u8, u8), orientation: Orientation) -> Vec<((usize, usize), Orientation)> {
        let mut all_positions = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                // Check if the vehicle fits in the grid based on its orientation
                if (orientation == Orientation::Horizontal && x + 2 < GRID_WIDTH) ||
                   (orientation == Orientation::Vertical && y + 2 < GRID_HEIGHT) {
                    // Check if starting position is suitable for placement
                    if self.can_place_vehicle(game, x, y, orientation, 3) {
                        all_positions.push(((x, y), orientation));
                    }
                }
            }
        }
        all_positions
    } 

    // This method checks if a vehicle can be placed at a given position with a given orientation
    fn can_place_vehicle(&self, game: &Game, x: usize, y: usize, orientation: Orientation, length: usize) -> bool {
        // First, check if the starting position is within bounds
        if !self.is_position_in_bounds(x, y) {
            return false;
        }

        match orientation {
            Orientation::Horizontal => {
                // Check if vehicle fits horizontally within the grid
                if x + length > GRID_WIDTH {
                    return false;
                }

                // Ensure no overlap with existing vehicles
                for xi in x..std::cmp::min(x + length, GRID_WIDTH) {
                    if !game.is_position_empty(xi, y) {
                        return false;
                    }
                }

                true
            },
            Orientation::Vertical => {
                // Check if vehicle fits vertically within the grid
                if y + length > GRID_HEIGHT {
                    return false;
                }

                // Ensure no overlap with existing vehicles
                for yi in y..std::cmp::min(y + length, GRID_HEIGHT) {
                    if !game.is_position_empty(x, yi) {
                        return false;
                    }
                }

                true
            }
        }
    }

    // This method checks if a given position is within the bounds of the grid
    fn is_position_in_bounds(&self, x: usize, y: usize) -> bool {
        x < GRID_WIDTH && y < GRID_HEIGHT
    }

    pub fn verify_solvability(&self, game: &Game) {
        // Assuming the red car needs to reach the far right of the center row to exit
        if let Some(red_car) = self.vehicle_struct.vehicles.iter().find(|v| v.ansi_color == AnsiColorCode::Red && v.orientation == Orientation::Horizontal) {
            // The center row can be calculated as GRID_HEIGHT / 2
            let center_row = GRID_HEIGHT / 2;
            
            // Check if the red car is already in the center row
            if red_car.position.1 as usize == center_row {
                // Check if the path from the red car's rightmost position to the right edge of the grid is clear
                let path_clear = ((red_car.position.0 + red_car.size.0 as u8) as usize..GRID_WIDTH).all(|x| {
                    game.is_position_empty(x, center_row)
                });

                if path_clear {
                    println!("Puzzle is solvable: Red car has a clear path to the exit.");
                } else {
                    println!("Puzzle may not be solvable: Red car's path to the exit is blocked.");
                }
            } else {
                println!("Red car is not in the center row, puzzle may not be solvable.");
            }
        } else {
            println!("Red car not found or not correctly positioned for solvability check.");
        }
    }
    
    // Note: You'll need to ensure that `is_position_empty` method is accessible and correctly implemented
    // to check if a specific grid position is empty.

	fn puzzle_generated(&self, game: &Game) -> bool {
		const MIN_VEHICLES_REQUIRED: usize = 5;
		let vehicles_placed = self.vehicle_struct.vehicles.len();
	
        if vehicles_placed >= MIN_VEHICLES_REQUIRED {
            println!("Puzzle generated with {} vehicles. Checking solvability.", vehicles_placed);
            self.verify_solvability(game);  // Call to verify solvability here
            true
        } else {
            false
        }
	}

    // === start of movement code ===

    pub fn move_vehicles_strategically(&mut self, game: &mut Game) {
        let mut moves_to_apply = Vec::new();
    
        // Collect moves
        for vehicle in &game.vehicles {
            if let Some(mv) = self.calculate_possible_moves(vehicle, game)
                .iter()
                .max_by_key(|mv| self.calculate_move_complexity(mv, game)) {
                moves_to_apply.push(*mv);
                println!("Collected move for vehicle ID {}: Move {:?} steps in direction {:?}", vehicle.id, mv.steps, mv.direction);
            }
        }
    
        // Apply moves
        for mv in moves_to_apply {
            self.apply_move(mv, game);
        }
    }    

    fn calculate_possible_moves(&self, vehicle: &Vehicle, game: &Game) -> Vec<Move> {
        let mut moves = Vec::new();

        // Check moves in the forward direction
        let forward_moves = self.check_direction(vehicle, game, MoveDirection::Forward);
        moves.extend(forward_moves);

        // Check moves in the backward direction
        let backward_moves = self.check_direction(vehicle, game, MoveDirection::Backward);
        moves.extend(backward_moves);

        moves
    }

    fn check_direction(&self, vehicle: &Vehicle, game: &Game, direction: MoveDirection) -> Vec<Move> {
        let mut moves = Vec::new();
        let (dx, dy) = match vehicle.orientation {
            Orientation::Horizontal => match direction {
                MoveDirection::Forward => (1, 0),
                MoveDirection::Backward => (-1, 0),
            },
            Orientation::Vertical => match direction {
                MoveDirection::Forward => (0, 1),
                MoveDirection::Backward => (0, -1),
            },
        };

        let mut steps = 1;
        loop {
            let new_position = self.calculate_new_position(vehicle.position, dx * steps, dy * steps);
            if self.is_valid_position(new_position, vehicle.size, vehicle.orientation, game) {
                let mut steps_u8 = steps as u8;
                moves.push(Move {
                    vehicle_id: vehicle.id,
                    direction,
                    steps: steps_u8,
                });
            } else {
                break;
            }
            steps += 1;
        }

        moves
    }

    fn calculate_new_position(&self, original_position: (u8, u8), dx: isize, dy: isize) -> (u8, u8) {
        // Calculate the new position based on the delta values (dx, dy)
        // Ensure the new position does not exceed grid boundaries
        let new_x = (original_position.0 as isize + dx).max(0).min(GRID_WIDTH as isize - 1) as u8;
        let new_y = (original_position.1 as isize + dy).max(0).min(GRID_HEIGHT as isize - 1) as u8;
        (new_x, new_y)
    }

    fn is_valid_position(&self, position: (u8, u8), size: (u8, u8), orientation: Orientation, game: &Game) -> bool {
        // Example implementation; modify as per your game's logic.
        let positions = self.get_occupied_positions(position, size, orientation);
        positions.iter().all(|&pos| pos.0 < GRID_WIDTH as u8 && pos.1 < GRID_HEIGHT as u8 && game.is_position_empty(pos.0 as usize, pos.1 as usize))
    }
    
    fn get_occupied_positions(&self, position: (u8, u8), size: (u8, u8), orientation: Orientation) -> Vec<(u8, u8)> {
        let mut positions = Vec::new();
        match orientation {
            Orientation::Horizontal => {
                for i in 0..size.0 {
                    positions.push((position.0 + i, position.1));
                }
            },
            Orientation::Vertical => {
                for i in 0..size.1 {
                    positions.push((position.0, position.1 + i));
                }
            }
        }
        positions
    }    

    fn select_best_move(&self, possible_moves: &[Move], game: &Game) -> Option<Move> {
        possible_moves.iter()
            .max_by_key(|mv| self.calculate_move_complexity(mv, game))
            .copied()
    }

    fn calculate_move_complexity(&self, r#move: &Move, game: &Game) -> usize {
        // Logic to calculate the complexity increase for a given move
        // Consider factors like blocking the path of the red car,
        // creating bottlenecks, etc.
        // Example implementation; adjust based on your game's complexity criteria.
        match r#move.direction {
            MoveDirection::Forward => 10, // Assign some complexity value
            MoveDirection::Backward => 5, // Different complexity for backward move
        }
    }

    fn apply_move(&self, r#move: Move, game: &mut Game) {
        // Logic to apply the selected move to the game state
        // Update the vehicle's position in the game grid
        if let Some(vehicle) = game.vehicles.iter_mut().find(|v| v.id == r#move.vehicle_id) {
            let (dx, dy) = match r#move.direction {
                MoveDirection::Forward => (r#move.steps as isize, 0),
                MoveDirection::Backward => (-(r#move.steps as isize), 0),
            };
            let original_position = vehicle.position;
            let new_position = self.calculate_new_position(vehicle.position, dx, dy);
            vehicle.set_position(new_position);

            // Debug statement to show movement
            println!("Moved vehicle ID {} from {:?} to {:?}", vehicle.id, original_position, new_position);
        }
        game.update_grid(); // Ensure you have this method to update the grid
        
        let grid_clone = game.grid.clone();
        let vehicles_clone = game.vehicles.clone();
        game.display_carpark(&vehicles_clone, &grid_clone);
    }

    // === end of movement code ===
}

struct ComplexityMeasure {
	// Fields and methods to calculate the complexity of the puzzle
}

impl ComplexityMeasure {
	pub fn new() -> Self {
		// initialization logic
		ComplexityMeasure {}
	}

    // Updated calculate method
    pub fn calculate(&self, game: &Game, position: (usize, usize), size: (u8, u8), orientation: Orientation) -> usize {
        let mut complexity = 0;

        // Debug message for initial complexity
        println!("Initial complexity: {}", complexity);

        // Increase complexity for positions closer to the center of the grid
        let center_x = GRID_WIDTH / 2;
        let center_y = GRID_HEIGHT / 2;
        complexity += (center_x as isize - position.0 as isize).abs() as usize;
        complexity += (center_y as isize - position.1 as isize).abs() as usize;

        // Debug message after calculating distance to center
        println!("Complexity after center distance calculation: {}", complexity);

        // Increase complexity based on the orientation and size
        match orientation {
            Orientation::Horizontal => {
                for xi in position.0..position.0 + size.0 as usize {
                    if game.is_position_empty(xi, position.1) {
                        complexity += 1; // Increase complexity for each empty space
                        // Debug message for each position check
                        println!("Complexity increased for Horizontal at position ({}, {}), current complexity: {}", xi, position.1, complexity);
                    }
                }
            },
            Orientation::Vertical => {
                for yi in position.1..position.1 + size.1 as usize {
                    if game.is_position_empty(position.0, yi) {
                        complexity += 1; // Increase complexity for each empty space
                        // Debug message for each position check
                        println!("Complexity increased for Vertical at position ({}, {}), current complexity: {}", position.0, yi, complexity);
                    }
                }
            }
        }

        // Final debug message for complexity
        println!("Final calculated complexity: {}", complexity);

        complexity
    }
}

// === start of movement code ===

#[derive(Clone, Copy, Debug)]
struct Move {
    vehicle_id: usize, // ID of the vehicle to move
    direction: MoveDirection, // Direction to move the vehicle
    steps: u8, // Number of steps to move
}

#[derive(Clone, Copy, Debug)]
enum MoveDirection {
    Forward, // Towards the end of the grid
    Backward, // Towards the start of the grid
}

// === end of movement code ===