use crate::Game;
use crate::vehicle_struct::{VehicleStruct, Vehicle, Orientation, AnsiColorCode};
use rand::{thread_rng, Rng};
use rand::prelude::SliceRandom;
use std::collections::HashSet;
use crate::vehicle_struct::Move;
use crate::game::MoveType;
use crate::solver;
use crate::solver::State;

pub const GRID_WIDTH: usize = 7;
pub const GRID_HEIGHT: usize = 7;
const DESIRED_COMPLEXITY_THRESHOLD: usize = 50;
const NUMBER_OF_MOVES_TO_CONSIDER: usize = 20;
const MAX_MOVE_DISTANCE: isize = 5;

pub struct PuzzleGenerator {
    complexity_measure: ComplexityMeasure,
    desired_complexity_threshold: usize,
    current_complexity: usize, // New field to store the current complexity
    pub vehicle_struct: VehicleStruct,
    used_colors: HashSet<String>,
}

impl PuzzleGenerator {
    pub fn new() -> Self {
        let complexity_measure = ComplexityMeasure::new();
        let desired_complexity_threshold = DESIRED_COMPLEXITY_THRESHOLD;
        let vehicle_struct = VehicleStruct::new();
        let used_colors = HashSet::new();
        PuzzleGenerator {
            complexity_measure,
            desired_complexity_threshold,
            current_complexity: 0, // Initialize current complexity to 0
            vehicle_struct,
            used_colors,
        }
    }

    pub fn generate_puzzle(&mut self, game: &mut Game) -> Vec<Vehicle> {
        let mut vehicle_id: usize = 0;
        self.place_red_car(game, &mut vehicle_id);
    
        while !self.is_puzzle_complex_enough(game) {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.5) {
                println!("Attempting to place a new vehicle");
                self.add_vehicle_strategically(game, &mut vehicle_id);
            } else {
                println!("Attempting to move an existing vehicle");
                self.add_vehicle_movements(game);
            }
            
            // Debug: Print current state of the game
            println!("Current game state");
            // Invoke the solver
            let current_state = State::new(game.vehicles.clone());
            let solution = current_state.solve_puzzle(current_state.clone());
            // ... handle the solution ...
    
            // Debug output
            for vehicle in &game.vehicles {
                println!("Vehicle ID: {}, Position: {:?}", vehicle.id, vehicle.position);
            }
        }
    
        println!("Puzzle generation complete");
        game.vehicles.clone()
    }    
    
    fn calculate_current_puzzle_complexity(&self, game: &Game) -> usize {
        let mut total_complexity = 0;
        for vehicle in &game.vehicles {
            // Skip the main red vehicle with id: 0
            if vehicle.id != 0 {
                let complexity = self.complexity_measure.calculate_for_vehicle_placement(
                    game,
                    (vehicle.position.0 as usize, vehicle.position.1 as usize),
                    vehicle.size,
                    vehicle.orientation
                );
                /*println!("Debug: Vehicle ID: {}, Position: {:?}, Size: {:?}, Orientation: {:?}, Complexity: {}", vehicle.id, vehicle.position, vehicle.size, vehicle.orientation, complexity);*/
                total_complexity += complexity;
            }
        }
        /*println!("Total Complexity: {}", total_complexity);*/
        total_complexity
    }
    
    fn should_increase_complexity(&self, game: &Game, current_complexity: usize) -> bool {
        // Determine if the current complexity is below a certain threshold
        // and decide if you need to make more moves to increase complexity
        current_complexity < self.desired_complexity_threshold
    }
    
    fn is_puzzle_complex_enough(&self, game: &Game) -> bool {
        let current_complexity = self.calculate_current_puzzle_complexity(game);
        /*println!("Desired Complexity Threshold: {}", self.desired_complexity_threshold);*/
        /*println!("Current Puzzle Complexity: {}", current_complexity);*/
    
        // Check if the puzzle meets the complexity criteria you've set
        current_complexity >= self.desired_complexity_threshold
    }

	fn place_red_car(&mut self, game: &mut Game, vehicle_id: &mut usize) {
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

        println!("Successfully added vehicle ID: {}, Position: {:?}, Size: {:?}, Orientation: {:?}", vehicle_id, red_car.position, red_car.size, red_car.orientation);

		// Place the red car at the exit
		// Assuming 'game' is a mutable reference to the Game struct and has a method to add vehicles
        game.update_grid_with_new_vehicle(&red_car, *vehicle_id);
	    game.vehicles.push(red_car);

        *vehicle_id += 1;
	}

    pub fn add_vehicle_strategically(&mut self, game: &mut Game, vehicle_id: &mut usize) -> Vec<Vehicle> {
        let vehicle_size = self.generate_random_vehicle_size();
        let random_orientation = self.generate_random_orientation();
    
        /*println!("Debug: Generating vehicle - ID: {}, Size: {:?}, Orientation: {:?}", vehicle_id, vehicle_size, random_orientation);*/
    
        let mut possible_positions: Vec<((usize, usize), Orientation)> = match (vehicle_size, random_orientation) {
            ((1, 2), Orientation::Vertical) => self.generate_possible_positions_1x2(game, vehicle_size, *vehicle_id, Orientation::Vertical),
            ((2, 1), Orientation::Horizontal) => self.generate_possible_positions_1x2(game, vehicle_size, *vehicle_id, Orientation::Horizontal),
            ((1, 3), Orientation::Vertical) => self.generate_possible_positions_1x3(game, vehicle_size, *vehicle_id, Orientation::Vertical),
            ((3, 1), Orientation::Horizontal) => self.generate_possible_positions_1x3(game, vehicle_size, *vehicle_id, Orientation::Horizontal),
            _ => Vec::new(),
        };
    
        /*println!("Debug: Possible positions generated for Vehicle ID {}: {:?}", vehicle_id, possible_positions);*/
    
        let mut new_possible_positions: Vec<((usize, usize), (usize, usize), Orientation)> = Vec::new();
        // Iterate and update each element
        for possible_position in possible_positions.iter_mut() {
            let (pos, ori) = *possible_position; // Destructure the tuple to get its values
            new_possible_positions.push((pos, (vehicle_size.0 as usize, vehicle_size.1 as usize), ori));
        }
    
        /*println!("Debug: New possible positions updated for Vehicle ID {}: {:?}", vehicle_id, new_possible_positions);*/
    
        let mut best_position: (usize, usize) = (0, 0);
        let mut best_orientation: Orientation = Orientation::Horizontal;
        let mut best_complexity: usize = 0;
        let mut best_size: (usize, usize) = (3, 1);
        
        // Pick the best positioned vehicle out of all possible positions
        for (position, size, orientation) in &new_possible_positions {
            let mut position_valid = true;

            match orientation {
                Orientation::Horizontal => {
                    for x in position.0..(position.0 + size.0) {
                        if !game.is_position_empty(x, position.1) {
                            /*println!("Debug: Position ({}, {}) is not empty for Horizontal Orientation", x, position.1);*/
                            position_valid = false;
                            break;
                        } else {
                            /*println!("Debug: Position ({}, {}) is empty for Horizontal Orientation", x, position.1);*/
                        }
                    }
                },
                Orientation::Vertical => {
                    for y in position.1..(position.1 + size.1) {
                        if !game.is_position_empty(position.0, y) {
                            /*println!("Debug: Position ({}, {}) is not empty for Vertical Orientation", position.0, y);*/
                            position_valid = false;
                            break;
                        } else {
                            /*println!("Debug: Position ({}, {}) is empty for Vertical Orientation", position.0, y);*/
                        }
                    }
                }
            }

            if position_valid {
                /*println!("Debug: Valid position found at ({}, {}) with Orientation {:?}", position.0, position.1, orientation);*/

                let complexity = self.complexity_measure.calculate_for_vehicle_placement(game, *position, (size.0 as u8, size.1 as u8), *orientation);

                // Debug message for comparing vehicle complexities
                /*println!("Comparing Vehicle ID: {} - Position: {:?}, Size: {:?}, Orientation: {:?}, Complexity: {}", vehicle_id, position, size, orientation, complexity);*/

                if complexity > best_complexity {
                    best_complexity = complexity;
                    best_position = *position;
                    best_size = *size;
                    best_orientation = *orientation;

                    // Debug message for the best choice at this point
                    /*println!("New Best Vehicle ID: {} - Position: {:?}, Size: {:?}, Orientation: {:?}, Complexity: {}", vehicle_id, best_position, best_size, best_orientation, best_complexity);*/
                }
            } else {
                /*println!("Debug: Invalid position at ({}, {}) with Orientation {:?}", position.0, position.1, orientation);*/
            }
        }

        // Add the best positioned vehicle to the carpark grid
        for (index, (position, size, orientation)) in new_possible_positions.iter().enumerate() {

            if (*position == best_position) && (*size == best_size) && (*orientation == best_orientation)  {
                let vehicle = self.generate_vehicle(*position, *size, *orientation, vehicle_id);
                println!("Suitable placement found for vehicle ID: {}, position: ({}, {})", vehicle_id, position.0, position.1);

                // Debug message for the vehicle being added
                println!("Adding Vehicle - ID: {}, Position: {:?}, Size: {:?}, Orientation: {:?}", vehicle_id, position, size, orientation);

                // Update the game's grid to reflect the new vehicle
                game.update_grid_with_new_vehicle(&vehicle, *vehicle_id);
                game.vehicles.push(vehicle);

                // Calculate the complexity for this vehicle placement
                let placement_complexity = self.complexity_measure.calculate_for_vehicle_placement(
                    game,
                    *position,
                    vehicle.size,
                    best_orientation
                );
                // Add this complexity to the current complexity of the puzzle
                self.current_complexity += placement_complexity;

                // Debug message for the total complexity after adding this vehicle
                println!("Total Complexity after adding vehicle ID {}: {}", vehicle_id, self.current_complexity);
                
            } else {
                println!("No suitable placement found for vehicle ID: {}, position: ({}, {}), size: ({}, {}), orientation: {:?}", vehicle_id, position.0, position.1, size.0, size.1, orientation);
            }

            if index >= new_possible_positions.len() - 1 {
                *vehicle_id += 1;
            }
        }
        
        game.vehicles.clone() // Return the updated list of vehicles
    }
	
	fn generate_vehicle(&mut self, position: (usize, usize), size: (usize, usize), orientation: Orientation, vehicle_id: &mut usize) -> Vehicle {
		let mut rng = rand::thread_rng();
	
		let color_options: Vec<AnsiColorCode> = vec![
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

        // Create and return the vehicle using the RGBA color and the provided position
        let vehicle = Vehicle::new(*vehicle_id, rgba_color, (size.0 as u8, size.1 as u8), (position.0 as u8, position.1 as u8), orientation, ansi_color);
		println!("Generated vehicle ID: {}, Position: {:?}, Size: {:?}, Orientation: {:?}", vehicle_id, position, size, orientation);
		
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

    fn generate_possible_positions_1x2(&self, game: &mut Game, vehicle_size: (u8, u8), vehicle_id: usize, orientation: Orientation) -> Vec<((usize, usize), Orientation)> {
        let mut all_positions = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                // Check if the vehicle fits in the grid based on its orientation
                if (orientation == Orientation::Horizontal && x + 1 < GRID_WIDTH) ||
                   (orientation == Orientation::Vertical && y + 1 < GRID_HEIGHT) {
                    /*println!("Debug: Checking position ({}, {}) for 1x2 vehicle, Orientation: {:?}", x, y, orientation);*/
                    // Check if starting position is suitable for placement
                    if self.can_place_vehicle(game, x, y, orientation, 2) {
                        /*println!("Debug: Position ({}, {}) is suitable for 1x2 vehicle, Orientation: {:?}", x, y, orientation);*/
                        all_positions.push(((x, y), orientation));
                    }
                }
            }
        }
        all_positions
    }    

    fn generate_possible_positions_1x3(&self, game: &mut Game, vehicle_size: (u8, u8), vehicle_id: usize, orientation: Orientation) -> Vec<((usize, usize), Orientation)> {
        let mut all_positions = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                // Check if the vehicle fits in the grid based on its orientation
                if (orientation == Orientation::Horizontal && x + 2 < GRID_WIDTH) ||
                   (orientation == Orientation::Vertical && y + 2 < GRID_HEIGHT) {
                    /*println!("Debug: Checking position ({}, {}) for 1x3 vehicle, Orientation: {:?}", x, y, orientation);*/
                    // Check if starting position is suitable for placement
                    if self.can_place_vehicle(game, x, y, orientation, 3) {
                        /*println!("Debug: Position ({}, {}) is suitable for 1x3 vehicle, Orientation: {:?}", x, y, orientation);*/
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
            /*println!("Debug: Position ({}, {}) is out of bounds", x, y);*/
            return false;
        }

        match orientation {
            Orientation::Horizontal => {
                // Check if vehicle fits horizontally within the grid
                if x + length > GRID_WIDTH {
                    /*println!("Debug: Horizontal vehicle at ({}, {}) with length {} does not fit in grid", x, y, length);*/
                    return false;
                }

                // Ensure no overlap with existing vehicles
                for xi in x..std::cmp::min(x + length, GRID_WIDTH) {
                    if !game.is_position_empty(xi, y) {
                        /*println!("Debug: Position ({}, {}) is not empty or occupied for Horizontal orientation", xi, y);*/
                        return false;
                    }
                }
                /*println!("Debug: Position ({}, {}) is suitable for Horizontal orientation", x, y);*/
                true
            },
            Orientation::Vertical => {
                // Check if vehicle fits vertically within the grid
                if y + length > GRID_HEIGHT {
                    /*println!("Debug: Vertical vehicle at ({}, {}) with length {} does not fit in grid", x, y, length);*/
                    return false;
                }

                // Ensure no overlap with existing vehicles
                for yi in y..std::cmp::min(y + length, GRID_HEIGHT) {
                    if !game.is_position_empty(x, yi) {
                        /*println!("Debug: Position ({}, {}) is not empty or occupied for Vertical orientation", x, yi);*/
                        return false;
                    }
                }
                /*println!("Debug: Position ({}, {}) is suitable for Vertical orientation", x, y);*/
                true
            }
        }
    }

    // This method checks if a given position is within the bounds of the grid
    fn is_position_in_bounds(&self, x: usize, y: usize) -> bool {
        x < GRID_WIDTH && y < GRID_HEIGHT
    }

    // === start of movement code ===

    pub fn add_vehicle_movements(&mut self, game: &mut Game) {
        let mut all_valid_moves = Vec::new();

        // Step 1: List all possible moves for each vehicle
        for (index, vehicle) in game.vehicles.iter().enumerate() {
            let vehicle_moves = self.generate_possible_moves_for_vehicle(vehicle, game);
            all_valid_moves.extend(vehicle_moves);
        }

        // Step 3: Evaluate and select moves
        if let Some(best_move) = self.evaluate_and_select_best_move(&all_valid_moves, game) {
            // Step 4: Apply the selected move
            println!("Applying move: Vehicle Index: {}, Move Type: {:?}, Distance: {}, Position X: {:?}, Position Y: {:?}", 
            best_move.vehicle_index, best_move.move_type, best_move.distance, best_move.position_x, best_move.position_y);

            let best_move_clone = best_move.clone(); // Clone the move
            game.apply_move(best_move_clone);
            game.record_puzzle_generation_move(best_move);

            println!("After applying move:");
            for vehicle in &game.vehicles {
                println!("Vehicle ID: {}, Position: ({}, {})", vehicle.id, vehicle.position.0, vehicle.position.1);
            }
        }
    }

    // Generate all possible moves for a given vehicle
    fn generate_possible_moves_for_vehicle(&self, vehicle: &Vehicle, game: &Game) -> Vec<Move> {
        let mut possible_moves = Vec::new();

        let max_distance = match vehicle.orientation {
            Orientation::Horizontal => GRID_WIDTH as isize - vehicle.size.0 as isize,
            Orientation::Vertical => GRID_HEIGHT as isize - vehicle.size.1 as isize,
        };

        // Generate moves in the forward and backward directions
        for distance in 1..=max_distance {
            // Forward move
            let forward_move = self.generate_move_if_valid(vehicle, distance, game);
            if let Some(mv) = forward_move {
                possible_moves.push(mv);
            }

            // Backward move (negative distance)
            let backward_move = self.generate_move_if_valid(vehicle, -distance, game);
            if let Some(mv) = backward_move {
                possible_moves.push(mv);
            }
        }

        possible_moves
    }

    fn generate_move_if_valid(&self, vehicle: &Vehicle, distance: isize, game: &Game) -> Option<Move> {
        let new_position = self.calculate_new_position(vehicle, distance);
    
        if self.check_movement_validity(new_position, vehicle.size, vehicle.orientation, game) {
            Some(Move {
                vehicle_index: vehicle.id,
                move_type: MoveType::Movement,
                distance,
                position_x: None,
                position_y: None,
                new_position_x: Some(new_position.0),
                new_position_y: Some(new_position.1),
            })
        } else {
            None
        }
    }
    
    fn evaluate_and_select_best_move(&self, moves: &[Move], game: &mut Game) -> Option<Move> {
        let mut best_move = None;
        let mut highest_complexity = 0;
    
        for move_candidate in moves {
            if let Some(vehicle) = game.vehicles.get(move_candidate.vehicle_index) {
                let new_position = self.calculate_new_position(vehicle, move_candidate.distance);
    
                let mut move_with_position = move_candidate.clone();
                move_with_position.new_position_x = Some(new_position.0);
                move_with_position.new_position_y = Some(new_position.1);
    
                let complexity = self.calculate_move_complexity(game, &move_with_position);
                println!("Evaluating move: Vehicle Index: {}, Complexity: {}", move_candidate.vehicle_index, complexity);
    
                if complexity > highest_complexity {
                    highest_complexity = complexity;
                    best_move = Some(move_with_position);
                }
            }
        }
    
        if let Some(ref best_move) = best_move {
            println!("Selected best move: Vehicle Index: {}, Complexity: {}", best_move.vehicle_index, highest_complexity);
            // Update best_move with the new position before returning
            if let Some(vehicle) = game.vehicles.get(best_move.vehicle_index) {
                let new_position = self.calculate_new_position(vehicle, best_move.distance);
                return Some(Move {
                    new_position_x: Some(new_position.0),
                    new_position_y: Some(new_position.1),
                    ..best_move.clone()
                });
            }
        }
    
        best_move
    }       

    // --- start of complexity functions ---

    // Function to calculate the complexity of a move
    fn calculate_move_complexity(&self, game: &Game, move_candidate: &Move) -> usize {
        let mut complexity = 0;

        if let (Some(new_x), Some(new_y)) = (move_candidate.new_position_x, move_candidate.new_position_y) {
            // If new position is provided, calculate complexity based on the new position
            let new_position = (new_x, new_y);

            // Add complexity based on distance moved
            complexity += move_candidate.distance.abs() as usize;

            // Add complexity based on proximity to critical points like exit or main vehicle
            complexity += self.calculate_proximity_complexity(new_position, game);

            // Add complexity based on blocking potential (e.g., blocking the path of the main vehicle)
            let vehicle = &game.vehicles[move_candidate.vehicle_index];
            complexity += self.calculate_blocking_potential(new_position, vehicle.size, vehicle.orientation, game);
        } else {
            // Handle the case where new position is not provided
            // You may want to log an error or provide default behavior
            println!("Error: Move does not have a valid new position for complexity calculation.");
        }

        complexity
    }

    fn calculate_new_position(&self, vehicle: &Vehicle, distance: isize) -> (usize, usize) {
        let new_x = match vehicle.orientation {
            Orientation::Horizontal => {
                let temp_x = (vehicle.position.0 as isize) + distance;
                temp_x.clamp(0, GRID_WIDTH as isize - vehicle.size.0 as isize) as usize
            },
            _ => vehicle.position.0 as usize,
        };
    
        let new_y = match vehicle.orientation {
            Orientation::Vertical => {
                let temp_y = (vehicle.position.1 as isize) + distance;
                temp_y.clamp(0, GRID_HEIGHT as isize - vehicle.size.1 as isize) as usize
            },
            _ => vehicle.position.1 as usize,
        };
    
        (new_x, new_y)
    }    

    fn calculate_proximity_complexity(&self, position: (usize, usize), game: &Game) -> usize {
        let mut complexity = 0;
    
        // Example: Increase complexity based on closeness to the exit
        let exit_position = (GRID_WIDTH - 1, GRID_HEIGHT / 2); // Assuming exit is at a specific grid location
        let distance_to_exit = ((exit_position.0 as isize - position.0 as isize).abs() + 
                                (exit_position.1 as isize - position.1 as isize).abs()) as usize;
        complexity += distance_to_exit;
    
        // Example: Increase complexity if the move is near the main vehicle
        if let Some(main_vehicle) = game.vehicles.iter().find(|&v| v.id == 0) { // Assuming main vehicle has id 0
            let distance_to_main_vehicle = ((main_vehicle.position.0 as isize - position.0 as isize).abs() + 
                                            (main_vehicle.position.1 as isize - position.1 as isize).abs()) as usize;
            if distance_to_main_vehicle < 3 { // Arbitrary threshold for proximity
                complexity += 5; // Arbitrary complexity increase
            }
        }
    
        complexity
    }

    fn calculate_blocking_potential(&self, position: (usize, usize), size: (u8, u8), orientation: Orientation, game: &Game) -> usize {
        let mut complexity = 0;
    
        // Example: Increase complexity if the move blocks the path to the exit
        // Adjust the logic based on your game's grid and rules
        if orientation == Orientation::Vertical {
            if let Some(main_vehicle) = game.vehicles.iter().find(|&v| v.id == 0) {
                let main_vehicle_end_x = main_vehicle.position.0 as usize + main_vehicle.size.0 as usize;
                let is_to_the_right = position.0 >= main_vehicle_end_x;
                let is_in_same_row = (position.1..(position.1 + size.1 as usize)).any(|y| y == main_vehicle.position.1 as usize);
    
                if is_in_same_row && is_to_the_right {
                    complexity += 10; // Increase complexity for blocking the exit
                }
            }
        }
    
        complexity
    }

    // --- end of complexity functions ---

    fn check_movement_validity(&self, new_position: (usize, usize), size: (u8, u8), orientation: Orientation, game: &Game) -> bool {
        println!("Checking movement validity for new position: {:?}, size: {:?}, orientation: {:?}", new_position, size, orientation);
    
        // Check if the new position is within the grid bounds
        if new_position.0 + size.0 as usize > GRID_WIDTH || new_position.1 + size.1 as usize > GRID_HEIGHT {
            println!("Invalid move: new position is out of grid bounds.");
            return false;
        }
    
        // Check for collisions with other vehicles
        for vehicle in &game.vehicles {
            if self.vehicles_overlap(new_position, size, orientation, vehicle) {
                println!("Invalid move: collision detected with vehicle ID: {}", vehicle.id);
                return false;
            }
        }
    
        println!("Move is valid.");
        true
    }

    fn vehicles_overlap(&self, position: (usize, usize), size: (u8, u8), orientation: Orientation, vehicle: &Vehicle) -> bool {
        let occupied_by_first = self.get_occupied_positions(position, size, orientation);
        let occupied_by_second = self.get_occupied_positions((vehicle.position.0 as usize, vehicle.position.1 as usize), vehicle.size, vehicle.orientation);
    
        println!("Checking overlap:");
        println!("\tVehicle 1: Position: {:?}, Size: {:?}, Orientation: {:?}", position, size, orientation);
        println!("\tVehicle 2 (ID: {}): Position: {:?}, Size: {:?}, Orientation: {:?}", vehicle.id, vehicle.position, vehicle.size, vehicle.orientation);
    
        // Check for any overlap between the two sets of occupied cells
        let overlap = occupied_by_first.iter().any(|pos| occupied_by_second.contains(pos));
    
        if overlap {
            println!("\tOverlap detected between vehicles.");
        } else {
            println!("\tNo overlap detected.");
        }
    
        overlap
    }
    
    fn get_occupied_positions(&self, position: (usize, usize), size: (u8, u8), orientation: Orientation) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        match orientation {
            Orientation::Horizontal => {
                for x in position.0..(position.0 + size.0 as usize) {
                    positions.push((x, position.1));
                }
            },
            Orientation::Vertical => {
                for y in position.1..(position.1 + size.1 as usize) {
                    positions.push((position.0, y));
                }
            }
        }
        positions
    }

    // === end of movement code ===
}

struct ComplexityMeasure {
	// Fields and methods to calculate_for_vehicle_placement the complexity of the puzzle
}

impl ComplexityMeasure {
	pub fn new() -> Self {
		// initialization logic
		ComplexityMeasure {}
	}

    pub fn calculate_for_vehicle_placement(&self, game: &Game, position: (usize, usize), size: (u8, u8), orientation: Orientation) -> usize {
        let mut complexity = 0;
    
        // Proximity to critical points (e.g., exit, main vehicle)
        complexity += self.calculate_proximity_complexity(position, game);
    
        // Blocking potential
        complexity += self.calculate_blocking_potential(position, size, orientation, game);
    
        // Vehicle size and orientation
        complexity += self.calculate_size_orientation_complexity(size, orientation);
    
        // Maneuverability and future movement implications
        complexity += self.calculate_maneuverability_complexity(position, size, orientation, game);
    
        complexity
    }

    fn calculate_proximity_complexity(&self, position: (usize, usize), game: &Game) -> usize {
        let mut complexity = 0;
        let exit_position = (GRID_WIDTH - 1, GRID_HEIGHT / 2); // Assuming exit is at the far right of the middle row
    
        // Calculate distance to exit
        let distance_to_exit = ((exit_position.0 as isize - position.0 as isize).abs() 
                                + (exit_position.1 as isize - position.1 as isize).abs()) as usize;
        complexity += distance_to_exit;
    
        // Additional complexity if near the main vehicle (e.g., the red car)
        if let Some(main_vehicle) = game.vehicles.iter().find(|&v| v.id == 0) { // Assuming main vehicle has id 0
            let distance_to_main_vehicle = ((main_vehicle.position.0 as isize - position.0 as isize).abs() 
                                            + (main_vehicle.position.1 as isize - position.1 as isize).abs()) as usize;
            if distance_to_main_vehicle < 3 { // Arbitrary threshold for proximity
                complexity += 5; // Arbitrary complexity increase
            }
        }
    
        complexity
    }   
    
    fn calculate_blocking_potential(&self, position: (usize, usize), size: (u8, u8), orientation: Orientation, game: &Game) -> usize {
        let mut complexity = 0;
    
        if orientation == Orientation::Vertical {
            if let Some(main_vehicle) = game.vehicles.iter().find(|&v| v.id == 0) { // Assuming main vehicle has id 0
                // Calculate the rightmost column of the main vehicle
                let red_car_end_x = main_vehicle.position.0 as usize + main_vehicle.size.0 as usize;
    
                // Check if the vertical vehicle's column is to the right of the main vehicle's rightmost column
                let is_to_the_right = position.0 >= red_car_end_x;
    
                // Check if the vertical vehicle is in the same row as the main vehicle
                let is_in_row = (position.1..(position.1 + size.1 as usize - 1)).any(|y| y == main_vehicle.position.1 as usize);
    
                if is_in_row && is_to_the_right {
                    // Increase complexity if the vertical vehicle is blocking the path to the exit
                    complexity += 10; // Adjust complexity value as needed
                }
            }
        }
    
        complexity
    }

    fn calculate_size_orientation_complexity(&self, size: (u8, u8), orientation: Orientation) -> usize {
        let base_complexity = 1; // Base complexity for any vehicle
        let size_complexity = size.0 as usize * size.1 as usize; // Larger vehicles are more complex
        let orientation_complexity = if orientation == Orientation::Horizontal { 2 } else { 1 }; // Example: horizontal vehicles are more complex
    
        base_complexity + size_complexity + orientation_complexity
    }

    fn calculate_maneuverability_complexity(&self, position: (usize, usize), size: (u8, u8), orientation: Orientation, game: &Game) -> usize {
        let mut complexity = 0;
    
        // Calculate the number of moves required to move this vehicle out of the way of the main vehicle
        // This requires a more detailed understanding of your game's mechanics and might involve pathfinding algorithms
        complexity
    }
}

// === start of movement code ===

// Example of how to use enums for direction, if needed
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Move {
    // Updated constructor for creating a new Move
    pub fn new(vehicle_index: usize, distance: isize, new_position_x: Option<usize>, new_position_y: Option<usize>) -> Self {
        Move {
            vehicle_index,
            move_type: MoveType::Placement, // or Movement, as per your game logic
            distance,
            position_x: None,
            position_y: None,
            new_position_x,
            new_position_y,
        }
    }

    // Additional methods as needed for your game logic
}

// === end of movement code ===