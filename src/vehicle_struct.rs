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
        if self.orientation == Orientation::Vertical && self.position.1 > 0 {
            let new_start_y = usize::from(self.position.1) - 1;
            !state.vehicles.iter().any(|v| {
                if v.id != self.id && v.orientation == Orientation::Horizontal {
                    let v_x_range = usize::from(v.position.0)..usize::from(v.position.0) + usize::from(v.size.0);
                    let overlaps_x = v_x_range.contains(&usize::from(self.position.0));
                    let overlaps_y = new_start_y >= usize::from(v.position.1) && new_start_y < usize::from(v.position.1) + usize::from(v.size.1);
                    overlaps_x && overlaps_y
                } else {
                    false
                }
            })
        } else {
            false
        }
    }    

    pub fn can_move_down(&self, state: &State) -> bool {
        if self.orientation == Orientation::Vertical && usize::from(self.position.1) + usize::from(self.size.1) < GRID_HEIGHT {
            let new_end_y = usize::from(self.position.1) + usize::from(self.size.1);
            !state.vehicles.iter().any(|v| {
                if v.id != self.id && v.orientation == Orientation::Horizontal {
                    let v_x_range = usize::from(v.position.0)..usize::from(v.position.0) + usize::from(v.size.0);
                    let overlaps_x = v_x_range.contains(&usize::from(self.position.0));
                    let overlaps_y = new_end_y > usize::from(v.position.1) && new_end_y <= usize::from(v.position.1) + usize::from(v.size.1);
                    overlaps_x && overlaps_y
                } else {
                    false
                }
            })
        } else {
            false
        }
    }    

    pub fn can_move_left(&self, state: &State) -> bool {
        if self.orientation == Orientation::Horizontal && self.position.0 > 0 {
            let new_start_x = usize::from(self.position.0) - 1;
            !state.vehicles.iter().any(|v| {
                if v.id != self.id && v.orientation == Orientation::Vertical {
                    let v_y_range = usize::from(v.position.1)..usize::from(v.position.1) + usize::from(v.size.1);
                    let overlaps_y = v_y_range.contains(&usize::from(self.position.1));
                    let overlaps_x = new_start_x >= usize::from(v.position.0) && new_start_x < usize::from(v.position.0) + usize::from(v.size.0);
                    overlaps_x && overlaps_y
                } else {
                    false
                }
            })
        } else {
            false
        }
    }

    pub fn can_move_right(&self, state: &State) -> bool {
        if self.orientation == Orientation::Horizontal && usize::from(self.position.0) + usize::from(self.size.0) < GRID_WIDTH {
            let new_end_x = usize::from(self.position.0) + usize::from(self.size.0);
            !state.vehicles.iter().any(|v| {
                if v.id != self.id && v.orientation == Orientation::Vertical {
                    let v_y_range = usize::from(v.position.1)..usize::from(v.position.1) + usize::from(v.size.1);
                    let overlaps_y = v_y_range.contains(&usize::from(self.position.1));
                    let overlaps_x = new_end_x > usize::from(v.position.0) && new_end_x <= usize::from(v.position.0) + usize::from(v.size.0);
                    overlaps_x && overlaps_y
                } else {
                    false
                }
            })
        } else {
            false
        }
    }

    pub fn move_up(&mut self) {
        if self.orientation == Orientation::Vertical {
            self.position.1 = self.position.1.saturating_sub(1);
        }
    }

    pub fn move_down(&mut self) {
        if self.orientation == Orientation::Vertical {
            self.position.1 = (usize::from(self.position.1) + 1).min(GRID_HEIGHT - usize::from(self.size.1)) as u8;
        }
    }
    
    pub fn move_left(&mut self) {
        if self.orientation == Orientation::Horizontal {
            self.position.0 = self.position.0.saturating_sub(1);
        }
    }

    pub fn move_right(&mut self) {
        if self.orientation == Orientation::Horizontal {
            self.position.0 = (usize::from(self.position.0) + 1).min(GRID_WIDTH - usize::from(self.size.0)) as u8;
        }
    }

    //=== end of solver code ===
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum Orientation {
    Horizontal,
    Vertical,
}