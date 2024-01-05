use crate::puzzle_generator::PuzzleGenerator;
use crate::game::MoveType;

use std::collections::HashSet;
use rand::Rng;

// === start of movement code ===

use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Eq)]
pub struct Move {
    pub vehicle_index: usize,
    pub move_type: MoveType,
    pub distance: isize, // Used only for Movement
    pub position_x: Option<isize>,
    pub position_y: Option<isize>,
    // You can keep the distance field for movement-specific data
    // Add any additional fields needed for placement, like orientation
    pub new_position_x: Option<usize>,
    pub new_position_y: Option<usize>,
}

impl Hash for Move {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vehicle_index.hash(state);
        self.move_type.hash(state);
        self.distance.hash(state);
        self.position_x.hash(state);
        self.position_y.hash(state);
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.vehicle_index == other.vehicle_index &&
        self.move_type == other.move_type &&
        self.distance == other.distance &&
        self.position_x == other.position_x &&
        self.position_y == other.position_y
    }
}

// === end of movement code ===

// === start of solver code ===

use crate::solver::State;
use crate::puzzle_generator::GRID_HEIGHT;
use crate::GRID_WIDTH;

// === end of solver code ===

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum AnsiColorCode {
    Red,    // Representing "\x1b[31m"
    Green,  // Representing "\x1b[32m"
    Blue,   // Representing "\x1b[34m"
    Cyan,           // Representing "\x1b[36m"
    Magenta,        // Representing "\x1b[35m"
    Yellow,         // Representing "\x1b[33m"
    BrightBlack,    // Representing "\x1b[1;30m"
    BrightWhite,    // Representing "\x1b[1;37m"
    BrightCyan,     // Representing "\x1b[1;36m"
    BrightMagenta,  // Representing "\x1b[1;35m"
    BrightYellow,   // Representing "\x1b[1;33m"
    Default // Representing "\x1b[0m"
}

pub struct VehicleStruct {
	pub vehicles: Vec<Vehicle>,
    last_id: usize, // New field to track the last assigned ID
}

impl VehicleStruct {
    pub fn new() -> Self {
        let vehicles = Vec::new(); // Initialise the vehicles vector
        let last_id = 0;
        VehicleStruct { vehicles, last_id } // Correctly return a VehicleStruct instance
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vehicle {
    pub id: usize, // Unique identifier for each vehicle
    pub color: [f32; 4], // RGBA format
    pub size: (u8, u8), 
    pub position: (u8, u8),
    pub orientation: Orientation,
    pub ansi_color: AnsiColorCode, // ANSI color code for terminal display
}

// Implement PartialEq for custom equality checks
impl PartialEq for Vehicle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        self.position == other.position &&
        self.orientation == other.orientation
        // Skip comparing `color`
    }
}

// Since we have a custom PartialEq, we need to implement Eq as well
impl Eq for Vehicle {}

// Implement Hash for custom hash computation
impl Hash for Vehicle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.position.hash(state);
        self.orientation.hash(state);
        // Skip hashing `color`
    }
}

// In some other part of your code:
impl AnsiColorCode {
    // Method to get the ANSI escape code string for terminal display
    pub fn to_ansi(self) -> &'static str {
        match self {
            AnsiColorCode::Red => "\x1b[31m",
            AnsiColorCode::Green => "\x1b[32m",
            AnsiColorCode::Blue => "\x1b[34m",
            AnsiColorCode::Cyan => "\x1b[36m",
            AnsiColorCode::Magenta => "\x1b[35m",
            AnsiColorCode::Yellow => "\x1b[33m",
            AnsiColorCode::BrightBlack => "\x1b[90m",
            AnsiColorCode::BrightWhite => "\x1b[1;37m",
            AnsiColorCode::BrightCyan => "\x1b[1;36m",
            AnsiColorCode::BrightMagenta => "\x1b[1;35m",
            AnsiColorCode::BrightYellow => "\x1b[1;33m",
            AnsiColorCode::Default => "\x1b[1;30m",
        }
    }

    // Method to get the RGBA values for graphical rendering
    pub fn to_rgba(self) -> [f32; 4] {
        match self {
            AnsiColorCode::Red => [1.0, 0.0, 0.0, 1.0],             // Red
            AnsiColorCode::Green => [0.0, 1.0, 0.0, 1.0],           // Green
            AnsiColorCode::Blue => [0.0, 0.0, 1.0, 1.0],            // Blue
            AnsiColorCode::Cyan => [0.0, 1.0, 1.0, 1.0],            // Cyan
            AnsiColorCode::Magenta => [1.0, 0.0, 1.0, 1.0],         // Magenta
            AnsiColorCode::Yellow => [1.0, 1.0, 0.0, 1.0],          // Yellow
            AnsiColorCode::BrightBlack => [0.7, 0.7, 0.7, 1.0],     // Bright Black
            AnsiColorCode::BrightWhite => [1.0, 1.0, 1.0, 1.0],     // Bright White
            AnsiColorCode::BrightCyan => [0.5, 1.0, 1.0, 1.0],      // Bright Cyan
            AnsiColorCode::BrightMagenta => [1.0, 0.5, 1.0, 1.0],   // Bright Magenta
            AnsiColorCode::BrightYellow => [1.0, 1.0, 0.5, 1.0],    // Bright Yellow
            AnsiColorCode::Default => [0.5, 0.5, 0.5, 1.0],         // Grey/Default
        }
    }
}

impl std::fmt::Display for AnsiColorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // This will use the enum variant name as the string representation
    }
}

impl Vehicle {
    // Constructor for Vehicle that takes an ANSI color as an argument
    pub fn new(
        id: usize, 
        color: [f32; 4], 
        size: (u8, u8), 
        position: (u8, u8), 
        orientation: Orientation, 
        ansi_color: AnsiColorCode  // Accept ANSI color as a parameter
    ) -> Self {
        Vehicle { id, color, size, position, orientation, ansi_color }
    }

    pub fn set_position(&mut self, position: (usize, usize)) {
        self.position = (position.0 as u8, position.1 as u8);
    }

    // Associated function to generate a random ANSI color code
    pub fn generate_random_ansi_color() -> String {
        let mut rng = rand::thread_rng();
        let color_code = rng.gen_range(31..=36); // ANSI color codes from red to cyan
        format!("\x1b[{}m", color_code)
    }

    // Helper method to get a list of occupied coordinates for this vehicle
    pub fn occupied_positions(&self) -> HashSet<(u8, u8)> {
        let (pos_x, pos_y) = self.position;
        let mut positions = HashSet::new();
        
        match self.orientation {
            Orientation::Horizontal => {
                for x in pos_x..pos_x + u8::from(self.size.0) {
                    positions.insert((x, pos_y));
                }
            }
            Orientation::Vertical => {
                for y in pos_y..pos_y + u8::from(self.size.1) {
                    positions.insert((pos_x, y));
                }
            }
        }
        positions
    }

    // === start of movement code ===

    // Method to calculate the complexity of moving this vehicle
    pub fn move_complexity(&self, move_distance: usize) -> usize {
        // Example: Complexity is simply the distance moved
        move_distance
    }

    // === end of movement code ===

    // === start of solver code ===

    pub fn can_move_up(&self, state: &State) -> bool {
        println!("Checking if vehicle {} can move up from position {:?}", self.id, self.position);
    
        // Check if the vehicle is oriented vertically and not at the upper boundary
        if self.orientation == Orientation::Vertical {
            if self.position.1 == 0 {
                println!("Vehicle {} cannot move up: already at the top boundary", self.id);
                return false;
            }
    
            let new_positions: HashSet<(u8, u8)> = self.occupied_positions()
                .iter()
                .map(|&(x, y)| (x, y - 1)) // Shift all positions up by 1 unit
                .collect();
    
            for other_vehicle in state.vehicles.iter() {
                if other_vehicle.id != self.id && new_positions.intersection(&other_vehicle.occupied_positions()).any(|_| true) {
                    println!("Collision detected: vehicle {} cannot move up due to vehicle {}", self.id, other_vehicle.id);
                    return false;
                }
            }
    
            println!("Vehicle {} can move up from position {:?}", self.id, self.position);
            true
        } else {
            println!("Vehicle {} cannot move up: wrong orientation {:?}", self.id, self.orientation);
            false
        }
    }    

    pub fn can_move_down(&self, state: &State) -> bool {
        println!("Checking if vehicle {} can move down from position {:?}", self.id, self.position);
    
        if self.orientation != Orientation::Vertical {
            println!("Vehicle {} cannot move down: wrong orientation {:?}", self.id, self.orientation);
            return false;
        }
    
        if usize::from(self.position.1 + self.size.1 as u8) >= GRID_HEIGHT {
            println!("Vehicle {} cannot move down: at the bottom boundary", self.id);
            return false;
        }
    
        let new_positions: HashSet<(u8, u8)> = self.occupied_positions()
            .iter()
            .map(|&(x, y)| (x, y + 1))  // Shift all positions down by 1 unit
            .collect();
    
        for other_vehicle in state.vehicles.iter() {
            if other_vehicle.id != self.id && new_positions.intersection(&other_vehicle.occupied_positions()).any(|_| true) {
                println!("Collision detected: vehicle {} cannot move down due to vehicle {}", self.id, other_vehicle.id);
                return false;
            }
        }
    
        println!("Vehicle {} can move down from position {:?}", self.id, self.position);
        true
    }    

    pub fn can_move_left(&self, state: &State) -> bool {
        println!("Checking if vehicle {} can move left from position {:?}", self.id, self.position);
    
        if self.orientation != Orientation::Horizontal {
            println!("Vehicle {} cannot move left: wrong orientation {:?}", self.id, self.orientation);
            return false;
        }
    
        if self.position.0 == 0 {
            println!("Vehicle {} cannot move left: at the left boundary", self.id);
            return false;
        }
    
        let new_positions: HashSet<(u8, u8)> = self.occupied_positions()
            .iter()
            .map(|&(x, y)| (x - 1, y))  // Shift all positions left by 1 unit
            .collect();
    
        for other_vehicle in state.vehicles.iter() {
            if other_vehicle.id != self.id && new_positions.intersection(&other_vehicle.occupied_positions()).any(|_| true) {
                println!("Collision detected: vehicle {} cannot move left due to vehicle {}", self.id, other_vehicle.id);
                return false;
            }
        }
    
        println!("Vehicle {} can move left from position {:?}", self.id, self.position);
        true
    }        

    pub fn can_move_right(&self, state: &State) -> bool {
        println!("Checking if vehicle {} can move right from position {:?}", self.id, self.position);
    
        if self.orientation != Orientation::Horizontal {
            println!("Vehicle {} cannot move right: wrong orientation {:?}", self.id, self.orientation);
            return false;
        }
    
        if usize::from(self.position.0 + self.size.0 as u8) >= GRID_WIDTH {
            println!("Vehicle {} cannot move right: at the right boundary", self.id);
            return false;
        }
    
        let new_positions: HashSet<(u8, u8)> = self.occupied_positions()
            .iter()
            .map(|&(x, y)| (x + 1, y))  // Shift all positions right by 1 unit
            .collect();
    
        for other_vehicle in state.vehicles.iter() {
            if other_vehicle.id != self.id && new_positions.intersection(&other_vehicle.occupied_positions()).any(|_| true) {
                println!("Collision detected: vehicle {} cannot move right due to vehicle {}", self.id, other_vehicle.id);
                return false;
            }
        }
    
        println!("Vehicle {} can move right from position {:?}", self.id, self.position);
        true
    }

    pub fn move_up(&mut self) {
        // Debug message
        println!("Moving vehicle {} up from position {:?}", self.id, self.position);
        if self.orientation == Orientation::Vertical {
            // Check that the vehicle is not at the upper boundary
            if self.position.1 > 0 {
                self.position.1 = self.position.1.saturating_sub(1);
                /*if self.id == 0 {
                    red_car.position.1 = self.position.1;
                }*/
                println!("New position of vehicle {}: {:?}", self.id, self.position);
            }
        }
        // No action for horizontal vehicles, as they can't move up/down
    }

    pub fn move_down(&mut self) {
        // Debug message
        println!("Moving vehicle {} down from position {:?}", self.id, self.position);
        if self.orientation == Orientation::Vertical {
            // Check that the vehicle does not exceed the grid boundary at the bottom
            if usize::from(self.position.1 + self.size.1 as u8) < GRID_HEIGHT {
                self.position.1 = (self.position.1 + 1).min((GRID_HEIGHT - 1) as u8);
                /*if self.id == 0 {
                    red_car.position.1 = self.position.1;
                }*/
                println!("New position of vehicle {}: {:?}", self.id, self.position);
            }
        }
        // No action for horizontal vehicles
    }

    pub fn move_left(&mut self) {
        // Debug message
        println!("Moving vehicle {} left from position {:?}", self.id, self.position);
        if self.orientation == Orientation::Horizontal {
            // Check that the vehicle is not at the leftmost boundary
            if self.position.0 > 0 {
                self.position.0 = self.position.0.saturating_sub(1);
                /*if self.id == 0 {
                    red_car.position.0 = self.position.0;
                }*/
                println!("New position of vehicle {}: {:?}", self.id, self.position);
            }
        }
        // No action for vertical vehicles, as they can't move left/right
    }    

    pub fn move_right(&mut self) {
        // Debug message
        println!("Moving vehicle {} right from position {:?}", self.id, self.position);
        if self.orientation == Orientation::Horizontal {
            // Check that the vehicle does not exceed the grid boundary on the right
            if usize::from(self.position.0 + self.size.0 as u8) < GRID_WIDTH {
                self.position.0 = (self.position.0 + 1).min((GRID_WIDTH - 1) as u8);
                /*if self.id == 0 {
                    red_car.position.0 = self.position.0;
                }*/
                println!("New position of vehicle {}: {:?}", self.id, self.position);
            }
        }
        // No action for vertical vehicles
    }

    //=== end of solver code ===
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum Orientation {
    Horizontal,
    Vertical,
}