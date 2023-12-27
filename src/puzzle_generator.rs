use crate::Game;
use crate::vehicle_struct::{VehicleStruct, Vehicle, Orientation, AnsiColorCode};
use rand::Rng;
use rand::prelude::SliceRandom;
use std::collections::HashSet;

pub const GRID_WIDTH: usize = 7;
pub const GRID_HEIGHT: usize = 7;
const DESIRED_COMPLEXITY_THRESHOLD: usize = 150;

/*pub struct GameManager {
    pub game: Game,
    // other fields as needed
}*/

/*impl GameManager {
    pub fn new(game: Game) -> Self {
        GameManager { game }
    }

    /*pub fn update_vehicles(&mut self, vehicles: Vec<Vehicle>) {
        self.game.set_vehicles(vehicles);
        // other state updates as needed
    }*/

    // other methods to manage game state
}*/

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
    
        /*while !self.puzzle_generated(game)*/ loop {
            self.add_vehicle_strategically(game, &mut vehicle_id);
            
            // Calculate combined complexity after each vehicle placement
            //let current_complexity = self.calculate_current_puzzle_complexity(game);
    
            // Move vehicles to increase puzzle complexity
            /*while self.should_increase_complexity(game, current_complexity) {
                self.move_vehicles_strategically(game);
            }*/
            
            // Check if the puzzle meets your complexity criteria
            if self.is_puzzle_complex_enough(game) {
                break;
            }
        }
    
        game.vehicles.clone()
    }
    
    fn calculate_current_puzzle_complexity(&self, game: &Game) -> usize {
        let mut total_complexity = 0;
        for vehicle in &game.vehicles {
            total_complexity += self.complexity_measure.calculate_for_vehicle_placement(
                game,
                (vehicle.position.0 as usize, vehicle.position.1 as usize),
                vehicle.size,
                vehicle.orientation
            );
        }

        println!("Total Complexity: {}", total_complexity);

        total_complexity
    }
    
    fn should_increase_complexity(&self, game: &Game, current_complexity: usize) -> bool {
        // Determine if the current complexity is below a certain threshold
        // and decide if you need to make more moves to increase complexity
        current_complexity < self.desired_complexity_threshold
    }
    
    fn is_puzzle_complex_enough(&self, game: &Game) -> bool {
        println!("Desired Complexity Threshold: {}", self.desired_complexity_threshold);

        // Check if the puzzle meets the complexity criteria you've set
        self.calculate_current_puzzle_complexity(game) >= self.desired_complexity_threshold
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

        println!("Successfully added vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", vehicle_id, red_car.size, red_car.position, red_car.orientation);

		// Place the red car at the exit
		// Assuming 'game' is a mutable reference to the Game struct and has a method to add vehicles
        game.update_grid_with_new_vehicle(&red_car, *vehicle_id);
	    game.vehicles.push(red_car);

        *vehicle_id += 1;
	}

    pub fn add_vehicle_strategically(&mut self, game: &mut Game, vehicle_id: &mut usize) -> Vec<Vehicle> {
        let vehicle_size = self.generate_random_vehicle_size();
        let random_orientation = self.generate_random_orientation();
        let mut possible_positions: Vec<((usize, usize), Orientation)> = match (vehicle_size, random_orientation) {
            ((1, 2), Orientation::Vertical) => self.generate_possible_positions_1x2(game, vehicle_size, *vehicle_id, Orientation::Vertical),
            ((2, 1), Orientation::Horizontal) => self.generate_possible_positions_1x2(game, vehicle_size, *vehicle_id, Orientation::Horizontal),
            ((1, 3), Orientation::Vertical) => self.generate_possible_positions_1x3(game, vehicle_size, *vehicle_id, Orientation::Vertical),
            ((3, 1), Orientation::Horizontal) => self.generate_possible_positions_1x3(game, vehicle_size, *vehicle_id, Orientation::Horizontal),
            _ => Vec::new(),
        };

        let mut new_possible_positions: Vec<((usize, usize), (usize, usize), Orientation)> = Vec::new();
        // Iterate and update each element
        for possible_position in possible_positions.iter_mut() {
            let (pos, ori) = *possible_position; // Destructure the tuple to get its values
            new_possible_positions.push((pos, (vehicle_size.0 as usize, vehicle_size.1 as usize), ori));
        }

        let mut best_position: (usize, usize) = (0, 0);
        let mut best_orientation: Orientation = Orientation::Horizontal;
        let mut best_complexity: usize = 0;
        let mut best_size: (usize, usize) = (3, 1);
        
        /*loop {*/
            // Pick the best positioned vehicle out of all possible positions
            for (position, size, orientation) in &new_possible_positions {
                if game.is_position_empty(position.0, position.1) {
                    let complexity = self.complexity_measure.calculate_for_vehicle_placement(game, *position, vehicle_size, *orientation);
                    
                    // Debug message for comparing vehicle complexities
                    println!("Comparing Vehicle - Position: {:?}, Size: {:?}, Orientation: {:?}, Complexity: {}", position, size, orientation, complexity);

                    if complexity > best_complexity {
                        best_complexity = complexity;
                        best_position = *position;
                        best_orientation = *orientation;
                        best_size = (vehicle_size.0 as usize, vehicle_size.1 as usize);

                        // Debug message for the best choice at this point
                        println!("New Best Vehicle - Position: {:?}, Size: {:?}, Orientation: {:?}, Complexity: {}", best_position, best_size, best_orientation, best_complexity);
                    }
                }
            }

            // Add the best positioned vehicle to the carpark grid
            for (position, size, orientation) in &new_possible_positions {
                if (*position == best_position) && (*size == best_size) && (*orientation == best_orientation)  {
                    let vehicle = self.generate_vehicle(*position, *size, *orientation, vehicle_id);
                    /*if self.vehicle_struct.add_vehicle(vehicle) {*/

                        // Debug message for the vehicle being added
                        println!("Adding Vehicle - ID: {}, Position: {:?}, Size: {:?}, Orientation: {:?}", vehicle_id, position, size, orientation);

                        // Update the game's grid to reflect the new vehicle
                        game.update_grid_with_new_vehicle(&vehicle, *vehicle_id);
                        game.vehicles.push(vehicle);

                        *vehicle_id += 1;

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

                        /*break;*/
                    /*} else {
                        println!("Failed to add vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", vehicle.id, vehicle.size, position, best_orientation);
                        // Remove the failed position from possible_positions and retry
                        possible_positions.retain(|&(p, o)| p != position || o != best_orientation);
                    }*/
                } else {
                    println!("No suitable position found for vehicle ID: {}, position: ({}, {})", vehicle_id, position.0, position.1);
                    /*break;*/
                }
            }
        /*}*/
        
        game.vehicles.clone() // Return the updated list of vehicles
    }
	
	fn generate_vehicle(&mut self, position: (usize, usize), size: (usize, usize), orientation: Orientation, vehicle_id: &mut usize) -> Vehicle {
		let mut rng = rand::thread_rng();
	
		let color_options: Vec<AnsiColorCode> = vec![
			/*AnsiColorCode::Red,*/
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
        /*let vehicle_id = 0;*/
        /*let sizes = match orientation {
            Orientation::Horizontal => [(2, 1), (3, 1)],
            Orientation::Vertical => [(1, 2), (1, 3)],
        };
        let size = sizes[rng.gen_range(0..sizes.len())];*/

        // Create and return the vehicle using the RGBA color and the provided position
        let vehicle = Vehicle::new(*vehicle_id, rgba_color, (size.0 as u8, size.1 as u8), (position.0 as u8, position.1 as u8), orientation, ansi_color);
		println!("Generated vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", vehicle_id, size, position, orientation);
		
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
                    // Check if starting position is suitable for placement
                    if self.can_place_vehicle(game, x, y, orientation, 2) {
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

    /*pub fn verify_solvability(&self, game: &Game) {
        // Assuming the red car needs to reach the far right of the center row to exit
        if let Some(red_car) = game.vehicles.iter().find(|v| v.ansi_color == AnsiColorCode::Red && v.orientation == Orientation::Horizontal) {
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
    }*/
    
    // Note: You'll need to ensure that `is_position_empty` method is accessible and correctly implemented
    // to check if a specific grid position is empty.

	/*fn puzzle_generated(&self, game: &Game) -> bool {
		const MIN_VEHICLES_REQUIRED: usize = 5;
		let vehicles_placed = game.vehicles.len();
	
        if vehicles_placed >= MIN_VEHICLES_REQUIRED {
            println!("Puzzle generated with {} vehicles. Checking solvability.", vehicles_placed);
            self.verify_solvability(game);  // Call to verify solvability here
            true
        } else {
            false
        }
	}*/

    // === start of movement code ===

    /*pub fn move_vehicles_strategically(&mut self, game: &mut Game) {
        let mut best_move = None;
        let mut max_complexity = 0;
    
        // Iterate over each vehicle to evaluate all possible moves
        for vehicle in &game.vehicles {
            let possible_moves = self.calculate_possible_moves(vehicle, game);
    
            // Find the move for this vehicle with the highest complexity
            for mv in possible_moves {
                let complexity = self.calculate_move_complexity(&mv, game);
                if complexity > max_complexity {
                    best_move = Some(mv);
                    max_complexity = complexity;
                }
            }
        }
    
        // Apply the best move found for any vehicle
        if let Some(mv) = best_move {
            self.apply_move(mv, game);
            game.display_carpark(&game.vehicles.clone(), &game.grid.clone());
        }
    }      

    fn calculate_possible_moves(&self, vehicle: &Vehicle, game: &Game) -> Vec<Move> {
        let mut moves = Vec::new();
        println!("Vehicle ID {}: Position: {:?} Size: {:?} Possible Moves: {:?}", vehicle.id, (vehicle.position.0 as usize, vehicle.position.1 as usize), (vehicle.size.0 as usize, vehicle.size.1 as usize), moves);

        // Check moves in the forward direction
        let forward_moves = self.check_direction(vehicle, game, MoveDirection::Forward);
        moves.extend(forward_moves.clone());

        // Check moves in the backward direction
        let backward_moves = self.check_direction(vehicle, game, MoveDirection::Backward);
        moves.extend(backward_moves.clone()); // Clone backward_moves here

        println!("Forward Moves: {:?}", forward_moves);
        println!("Backward Moves: {:?}", backward_moves);

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
    
        // Debug: Print the direction being checked
        println!("Checking direction {:?} for vehicle ID {}", direction, vehicle.id);
    
        let mut steps = 1;
        while let Ok(new_position) = self.calculate_new_position((vehicle.position.0 as usize, vehicle.position.1 as usize), dx * steps, dy * steps) {
            // Debug: Print the new position being checked
            println!("Checking new position {:?} for vehicle ID {}", new_position, vehicle.id);
    
            if self.is_valid_position((new_position.0 as usize, new_position.1 as usize), (vehicle.size.0 as usize, vehicle.size.1 as usize), vehicle.orientation, game, vehicle.id) {
                // Debug: Print the successful move
                println!("Valid move found for vehicle ID {}: Move {} steps in direction {:?}", vehicle.id, steps, direction);
    
                moves.push(Move {
                    vehicle_id: vehicle.id,
                    direction,
                    steps: steps as isize,
                });
            } else {
                // Debug: Print the reason for breaking the loop
                println!("Invalid position reached for vehicle ID {}: {:?}", vehicle.id, new_position);
                break;
            }
            steps += 1;
        }
    
        // Debug: Print the total number of moves found
        println!("Total moves found for vehicle ID {}: {}", vehicle.id, moves.len());
    
        moves
    }    

    fn calculate_new_position(&self, current_position: (usize, usize), dx: isize, dy: isize) -> Result<(usize, usize), &'static str> {
        let new_x = current_position.0 as isize + dx;
        let new_y = current_position.1 as isize + dy;
    
        if new_x < 0 || new_y < 0 || new_x >= GRID_WIDTH as isize || new_y >= GRID_HEIGHT as isize {
            Err("New position is outside the grid boundaries")
        } else {
            Ok((new_x as usize, new_y as usize))
        }
    }   

    fn is_valid_position(&self, new_position: (usize, usize), size: (usize, usize), orientation: Orientation, game: &Game, vehicle_id: usize) -> bool {
        // Debug: Print the position being checked for validity
        println!("Checking validity of new position: {:?}", new_position);
    
        let mut positions_to_check = Vec::new();
    
        match orientation {
            Orientation::Horizontal => {
                for x in new_position.0..new_position.0 + size.0 {
                    positions_to_check.push((x, new_position.1));
                }
            },
            Orientation::Vertical => {
                for y in new_position.1..new_position.1 + size.1 {
                    positions_to_check.push((new_position.0, y));
                }
            }
        }
    
        let is_valid = positions_to_check.iter().all(|&pos| {
            (pos.0 < GRID_WIDTH && pos.1 < GRID_HEIGHT) &&
            (game.is_position_empty(pos.0, pos.1) || game.is_occupied_by_vehicle(pos.0, pos.1, vehicle_id))
        });
    
        // Debug: Print the result of the validity check
        println!("New position {:?} is valid: {}", new_position, is_valid);
    
        is_valid
    }    
    
    fn get_occupied_positions(&self, position: (usize, usize), size: (usize, usize), orientation: Orientation) -> Vec<(usize, usize)> {
        println!("Getting occupied positions for Position: {:?}, Size: {:?}, Orientation: {:?}", position, size, orientation);
        
        let mut occupied_positions = Vec::new();
        match orientation {
            Orientation::Horizontal => {
                for i in 0..size.0 {
                    let new_x = position.0 + i;
                    if new_x < GRID_WIDTH {
                        occupied_positions.push((new_x, position.1));
                    }
                }
            },
            Orientation::Vertical => {
                for i in 0..size.1 {
                    let new_y = position.1 + i;
                    if new_y < GRID_HEIGHT {
                        occupied_positions.push((position.0, new_y));
                    }
                }
            }
        }
    
        println!("Occupied positions: {:?}", occupied_positions);
        occupied_positions
    }    

    fn select_best_move(&self, possible_moves: &[Move], game: &Game) -> Option<Move> {
        possible_moves.iter()
            .max_by_key(|mv| self.calculate_move_complexity(mv, game))
            .copied()
    }

    fn calculate_move_complexity(&self, r#move: &Move, game: &Game) -> usize {
        // Logic to calculate_for_vehicle_placement the complexity increase for a given move
        // Consider factors like blocking the path of the red car,
        // creating bottlenecks, etc.
        // Example implementation; adjust based on your game's complexity criteria.
        match r#move.direction {
            MoveDirection::Forward => 10, // Assign some complexity value
            MoveDirection::Backward => 5, // Different complexity for backward move
        }
    }

    fn apply_move(&self, r#move: Move, game: &mut Game) {
        if let Some(vehicle) = game.vehicles.iter_mut().find(|v| v.id == r#move.vehicle_id) {
            let (dx, dy) = match vehicle.orientation {
                Orientation::Horizontal => match r#move.direction {
                    MoveDirection::Forward => (r#move.steps, 0),
                    MoveDirection::Backward => (-r#move.steps, 0),
                },
                Orientation::Vertical => match r#move.direction {
                    MoveDirection::Forward => (0, r#move.steps),
                    MoveDirection::Backward => (0, -r#move.steps),
                },
            };

            println!("(dx, dy) = ({}, {})", dx, dy);
    
            if let Ok(new_position) = self.calculate_new_position((vehicle.position.0 as usize, vehicle.position.1 as usize), dx, dy) {
                vehicle.set_position(new_position);
                // Update the grid or any other necessary state here
                game.update_grid_for_vehicle(r#move.vehicle_id);
            } else {
                // Handle the case where the new position is invalid
                println!("Invalid move attempted for vehicle ID {}", vehicle.id);
            }
        }
    }*/

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

/*#[derive(Clone, Copy, Debug)]
struct Move {
    vehicle_id: usize, // ID of the vehicle to move
    direction: MoveDirection, // Direction to move the vehicle
    steps: isize, // Number of steps to move
}

impl Move {
    pub fn new() -> Self {
        let vehicle_id = 0;
        let direction = MoveDirection::Forward;
        let steps = 1;

        Move { vehicle_id, direction, steps }
    }
}

#[derive(Clone, Copy, Debug)]
enum MoveDirection {
    Forward, // Towards the end of the grid
    Backward, // Towards the start of the grid
}*/

// === end of movement code ===