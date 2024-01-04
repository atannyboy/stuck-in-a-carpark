// solver.rs

use std::collections::{HashMap, HashSet, VecDeque};
use crate::Vehicle;
use crate::vehicle_struct::Orientation;

const EXIT_X: usize = 6; // Adjust according to your game's grid

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct State {
    vehicles: Vec<Vehicle>,  // Assuming you have a Vehicle struct defined
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
            // Assuming you have defined methods like `can_move_up`, `move_up` etc. in your Vehicle struct
            if vehicle.can_move_up(&self) {
                let mut new_vehicles = self.vehicles.clone();
                new_vehicles[index].move_up(); // Modify the vehicle position
                neighbors.push((State::new(new_vehicles), Move { vehicle_index: index, direction: Direction::Up, distance: 1 }));
            }
            // Similarly, check for other directions (down, left, right)
        }

        neighbors
    }

    fn is_solution(&self) -> bool {
        // Assuming the red car is the first vehicle in the list and the exit is on the right
        let red_car = &self.vehicles[0];
        usize::from(red_car.position.0) == EXIT_X && red_car.orientation == Orientation::Horizontal
    }

    pub fn solve_puzzle(&self, initial_state: State) -> Option<Vec<Move>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut predecessors: HashMap<State, (State, Move)> = HashMap::new();
    
        queue.push_back(initial_state.clone());
        visited.insert(initial_state);
    
        while let Some(state) = queue.pop_front() {
            if self.is_solution() {
                return Some(reconstruct_path(state, predecessors));
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
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Move {
    vehicle_index: usize, // Index of the vehicle in the State's vehicle list
    direction: Direction, // Could be an enum representing directions
    distance: isize,      // How far the vehicle moves
}

#[derive(Clone, Eq, PartialEq, Hash)]
enum Direction {
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