// solver.rs

use std::collections::{HashMap, HashSet, VecDeque};
use crate::Vehicle;
use crate::vehicle_struct::Orientation;
use crate::puzzle_generator::GRID_WIDTH;
use crate::puzzle_generator::GRID_HEIGHT;

const EXIT_X: usize = 5; // Adjust according to your game's grid

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct State {
    pub vehicles: Vec<Vehicle>,  // Assuming you have a Vehicle struct defined
    // Add any other relevant game state information here
}

impl State {
    // Constructor
    pub fn new(vehicles: &Vec<Vehicle>) -> Self {
        State { vehicles: vehicles.to_vec() }
    }

    fn generate_neighbors(state: &State) -> Vec<(State, Move)> {
        let mut neighbors = Vec::new();
    
        for (index, vehicle) in state.vehicles.iter().enumerate() {
            // Generate moves for each direction
            if vehicle.can_move_up(&state) {
                let mut new_state = state.clone();
                new_state.vehicles[index].move_up();
                neighbors.push((State::new(&new_state.vehicles), Move::new(index, Direction::Up, 1)));
            }
            if vehicle.can_move_down(&state) {
                let mut new_state = state.clone();
                new_state.vehicles[index].move_down();
                neighbors.push((State::new(&new_state.vehicles), Move::new(index, Direction::Down, 1)));
            }
            if vehicle.can_move_left(&state) {
                let mut new_state = state.clone();
                new_state.vehicles[index].move_left();
                neighbors.push((State::new(&new_state.vehicles), Move::new(index, Direction::Left, 1)));
            }
            if vehicle.can_move_right(&state) {
                let mut new_state = state.clone();
                new_state.vehicles[index].move_right();
                neighbors.push((State::new(&new_state.vehicles), Move::new(index, Direction::Right, 1)));
            }
        }
    
        neighbors
    }

    fn special_red_car_handling(&self, current_state: &State) -> Option<Vec<(State, Move)>> {
        let mut red_car_moves = Vec::new();
        let red_car = &current_state.vehicles[0]; // Assuming the red car is always at index 0
    
        // Check if the red car can move towards the exit (to the right)
        if self.is_exit_direction(red_car, Direction::Right) && red_car.can_move_right(&self) {
            let mut new_state = current_state.clone();
            new_state.vehicles[0].move_right(); // Move the red car to the right
            red_car_moves.push((new_state, Move::new(0, Direction::Right, 1)));
        }
    
        // Optionally, add logic for other directions if needed
    
        if red_car_moves.is_empty() {
            None
        } else {
            Some(red_car_moves)
        }
    }

    fn is_exit_direction(&self, vehicle: &Vehicle, direction: Direction) -> bool {
        // Implement logic to check if moving in the specified direction
        // brings the vehicle closer to the exit
        // For simplicity, assuming exit is always to the right
        let is_towards_exit = direction == Direction::Right;
        println!("Is moving {:?} towards the exit for vehicle at position {:?}: {}", direction, vehicle.position, is_towards_exit);
        is_towards_exit
    }

    fn is_solution(state: &State) -> bool {
        let red_car = &state.vehicles[0]; // Assuming the red car is always at index 0
        println!("Checking if the red car is in a winning position...");
    
        if red_car.orientation == Orientation::Horizontal {
            let is_solution = usize::from(red_car.position.0/* + red_car.size.0 as u8 - 1*/) == EXIT_X;
            println!("Red car position: {:?}, Size: {:?}, Exit position: {}, Is solution: {}", red_car.position, red_car.size, EXIT_X, is_solution);
            
            is_solution
        } else {
            // Additional handling if the red car can be vertical
            println!("Red car is not horizontal. Current orientation: {:?}", red_car.orientation);
            false
        }
    }    

    pub fn solve_puzzle(&mut self, initial_state: State) -> Option<Vec<Move>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut predecessors: HashMap<State, (State, Move)> = HashMap::new();
    
        queue.push_back(initial_state.clone());
        visited.insert(initial_state);
    
        while let Some(state) = queue.pop_front() {
            println!("Exploring State: {:?}", state); // Debug statement
            if Self::is_solution(&state.clone()) {
                println!("Solution found for State: {:?}", state); // Debug statement
                let solution_path = Self::reconstruct_path(state, &predecessors);
                Self::display_solution_steps(&self.vehicles, &solution_path);
                return Some(solution_path);
            }
    
            // Special case handling for red car
            if let Some(red_car_moves) = self.special_red_car_handling(&state) {
                for (red_car_state, red_car_move) in red_car_moves {
                    if !visited.contains(&red_car_state) {
                        visited.insert(red_car_state.clone());
                        predecessors.insert(red_car_state.clone(), (state.clone(), red_car_move));
                        queue.push_back(red_car_state);
                    }
                }
            }
    
            for (next_state, game_move) in Self::generate_neighbors(&state) {
                println!("Generated neighbor state: {:?}", next_state);
                if !visited.contains(&next_state) {
                    println!("New state, adding to queue and visited: {:?}", next_state);
                    visited.insert(next_state.clone());
                    predecessors.insert(next_state.clone(), (state.clone(), game_move));
                    queue.push_back(next_state);
                } else {
                    println!("State already visited: {:?}", next_state);
                }
            }
        }
        println!("No solution found");
        None // No solution found
    }
    
    fn reconstruct_path(mut current_state: State, predecessors: &HashMap<State, (State, Move)>) -> Vec<Move> {
        let mut path = Vec::new();
        while let Some(&(ref prev_state, ref move_)) = predecessors.get(&current_state) {
            path.push(move_.clone());
            current_state = prev_state.clone();
        }
        path.reverse();
        path
    }  

    fn display_solution_steps(initial_vehicles: &[Vehicle], solution: &[Move]) {
        let mut vehicles = initial_vehicles.to_vec();
        let mut grid = Self::create_initial_grid(&vehicles);
    
        for (step, game_move) in solution.iter().enumerate() {
            // Apply the move to update vehicle positions
            Self::apply_move(&mut vehicles, game_move);
            // Update the grid
            Self::update_grid(&mut grid, &vehicles);
            // Display the carpark for this step
            println!("Step {}: Applying move {:?}", step + 1, game_move);
            Self::display_carpark(&vehicles, &grid);
        }
    }

    pub fn display_carpark(vehicles: &[Vehicle], grid: &[[Option<usize>; GRID_WIDTH]; GRID_HEIGHT]) {
        let mut output = String::new();
    
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                match grid[y][x] {
                    Some(index) => {
                        let vehicle = &vehicles[index];
                        output.push_str(&format!("{}▓▓", vehicle.ansi_color.to_ansi())); // Use vehicle's ANSI color
                    },
                    None => output.push_str("\x1b[90m░░"), // Grey for empty spaces
                }
            }
            output.push_str("\x1b[0m\n"); // Reset color and new line
        }
    
        println!("{}", output);
    }

    pub fn create_initial_grid(vehicles: &[Vehicle]) -> [[Option<usize>; GRID_WIDTH]; GRID_HEIGHT] {
        let mut grid = [[None; GRID_WIDTH]; GRID_HEIGHT];
    
        for (index, vehicle) in vehicles.iter().enumerate() {
            // Determine the grid cells occupied by the vehicle
            match vehicle.orientation {
                Orientation::Horizontal => {
                    for x in vehicle.position.0..vehicle.position.0 + u8::from(vehicle.size.0) {
                        if x < GRID_WIDTH as u8 {
                            grid[vehicle.position.1 as usize][x as usize] = Some(index);
                        }
                    }
                },
                Orientation::Vertical => {
                    for y in vehicle.position.1..vehicle.position.1 + u8::from(vehicle.size.1) {
                        if y < GRID_HEIGHT as u8 {
                            grid[y as usize][vehicle.position.0 as usize] = Some(index);
                        }
                    }
                },
            }
        }
    
        grid
    }
    
    fn apply_move(vehicles: &mut [Vehicle], game_move: &Move) {
        let vehicle = &mut vehicles[game_move.vehicle_index];
        // Depending on the direction of the move, update the vehicle's position
        match game_move.direction {
            Direction::Up => vehicle.move_up(),
            Direction::Down => vehicle.move_down(),
            Direction::Left => vehicle.move_left(),
            Direction::Right => vehicle.move_right(),
        }
    }

    fn update_grid(grid: &mut [[Option<usize>; GRID_WIDTH]; GRID_HEIGHT], vehicles: &[Vehicle]) {
        // Clear the grid
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                grid[y][x] = None;
            }
        }
    
        // Repopulate the grid with the current positions of the vehicles
        for (index, vehicle) in vehicles.iter().enumerate() {
            match vehicle.orientation {
                Orientation::Horizontal => {
                    for x in vehicle.position.0..vehicle.position.0 + u8::from(vehicle.size.0) {
                        if x < GRID_WIDTH as u8 {
                            grid[vehicle.position.1 as usize][x as usize] = Some(index);
                        }
                    }
                },
                Orientation::Vertical => {
                    for y in vehicle.position.1..vehicle.position.1 + u8::from(vehicle.size.1) {
                        if y < GRID_HEIGHT as u8 {
                            grid[y as usize][vehicle.position.0 as usize] = Some(index);
                        }
                    }
                },
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Move {
    vehicle_index: usize, // Index of the vehicle in the State's vehicle list
    direction: Direction, // Could be an enum representing directions
    distance: isize,      // How far the vehicle moves
}

impl Move {
    pub fn new(vehicle_index: usize, direction: Direction, distance: isize) -> Self {
        Move {
            vehicle_index,
            direction,
            distance,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    // Add other directions if applicable
}

/*fn reconstruct_path(end_state: State, predecessors: HashMap<State, (State, Move)>) -> Vec<Move> {
    let mut path = Vec::new();
    let mut current_state = end_state;

    while let Some((prev_state, game_move)) = predecessors.get(&current_state) {
        path.push(game_move.clone());
        current_state = prev_state.clone();
    }

    path.reverse(); // The path is constructed in reverse order, so it needs to be reversed
    path
}*/