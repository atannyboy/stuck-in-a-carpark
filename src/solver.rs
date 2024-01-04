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
    pub fn new(vehicles: Vec<Vehicle>) -> Self {
        State { vehicles }
    }

    fn generate_neighbors(&self) -> Vec<(State, Move)> {
        let mut neighbors = Vec::new();

        for (index, vehicle) in self.vehicles.iter().enumerate() {
            // Check and generate moves in each direction
            if vehicle.can_move_up(&self) {
                println!("Generating neighbor: Moving vehicle {} up", index);
                let mut new_vehicles = self.vehicles.clone();
                new_vehicles[index].move_up();
                neighbors.push((State::new(new_vehicles), Move::new(index, Direction::Up, 1)));
            }
            if vehicle.can_move_down(&self) {
                println!("Generating neighbor: Moving vehicle {} down", index);
                let mut new_vehicles = self.vehicles.clone();
                new_vehicles[index].move_down();
                neighbors.push((State::new(new_vehicles), Move::new(index, Direction::Down, 1)));
            }
            if vehicle.can_move_left(&self) {
                println!("Generating neighbor: Moving vehicle {} left", index);
                let mut new_vehicles = self.vehicles.clone();
                new_vehicles[index].move_left();
                neighbors.push((State::new(new_vehicles), Move::new(index, Direction::Left, 1)));
            }
            if vehicle.can_move_right(&self) {
                println!("Generating neighbor: Moving vehicle {} right", index);
                let mut new_vehicles = self.vehicles.clone();
                new_vehicles[index].move_right();
                neighbors.push((State::new(new_vehicles), Move::new(index, Direction::Right, 1)));
            }
        }

        neighbors
    }

    fn is_solution(state: State) -> bool {
        // Assuming the red car is the first vehicle in the list and the exit is on the right
        let red_car = state.vehicles[0];
        usize::from(red_car.position.0) == EXIT_X && red_car.orientation == Orientation::Horizontal
    }

    pub fn solve_puzzle(&self, initial_state: State) -> Option<Vec<Move>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut predecessors: HashMap<State, (State, Move)> = HashMap::new();
    
        queue.push_back(initial_state.clone());
        visited.insert(initial_state);
    
        while let Some(state) = queue.pop_front() {
            println!("Exploring State: {:?}", state); // Debug statement
            if Self::is_solution(state.clone()) {
                println!("Solution found for State: {:?}", state); // Debug statement
                let solution_path = reconstruct_path(state, predecessors);
                Self::display_solution_steps(&self.vehicles, &solution_path);
                return Some(solution_path);
            }
    
            for (next_state, game_move) in state.generate_neighbors() {
                if !visited.contains(&next_state) {
                    visited.insert(next_state.clone());
                    predecessors.insert(next_state.clone(), (state.clone(), game_move));
                    queue.push_back(next_state);
                }
            }
        }
    
        None // No solution found
    }

    fn reconstruct_path(end_state: State, predecessors: HashMap<State, (State, Move)>) -> Vec<Move> {
        let mut path = Vec::new();
        let mut current_state = end_state;
    
        while let Some((prev_state, game_move)) = predecessors.get(&current_state) {
            println!("Reconstructing move: {:?}", game_move); // Debug statement
            path.push(game_move.clone());
            current_state = prev_state.clone();
        }
    
        path.reverse(); // The path is constructed in reverse order
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

    pub fn display_carpark(vehicles: &[Vehicle], grid: &[[Option<usize>; GRID_WIDTH as usize]; GRID_WIDTH as usize]) {
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

    pub fn create_initial_grid(vehicles: &[Vehicle]) -> [[Option<usize>; GRID_WIDTH]; GRID_HEIGHT] {
        let mut grid = [[None; GRID_WIDTH]; GRID_HEIGHT];
    
        for (index, vehicle) in vehicles.iter().enumerate() {
            // Determine the grid cells occupied by the vehicle
            // This depends on the vehicle's position, size, and orientation
            match vehicle.orientation {
                Orientation::Horizontal => {
                    for x in vehicle.position.0..vehicle.position.0 + vehicle.size.0 as u8 {
                        if x < GRID_WIDTH as u8 {
                            grid[vehicle.position.1 as usize][x as usize] = Some(index);
                        }
                    }
                },
                Orientation::Vertical => {
                    for y in vehicle.position.1..vehicle.position.1 + vehicle.size.1 as u8 {
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
    
        // Place the vehicles on the grid
        for (index, vehicle) in vehicles.iter().enumerate() {
            // Assuming vehicles occupy one grid cell for simplicity
            // Adjust this logic based on the actual size and orientation of the vehicles
            grid[vehicle.position.1 as usize][vehicle.position.0 as usize] = Some(index);
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

fn reconstruct_path(end_state: State, predecessors: HashMap<State, (State, Move)>) -> Vec<Move> {
    let mut path = Vec::new();
    let mut current_state = end_state;

    while let Some((prev_state, game_move)) = predecessors.get(&current_state) {
        path.push(game_move.clone());
        current_state = prev_state.clone();
    }

    path.reverse(); // The path is constructed in reverse order, so it needs to be reversed
    path
}