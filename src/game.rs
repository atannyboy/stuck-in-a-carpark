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

    // Add this method to update vehicles
    /*pub fn set_vehicles(&mut self, vehicles: Vec<Vehicle>) {
        let vehicles_count = vehicles.len();
        self.vehicles = vehicles;
        println!("Setting vehicles. Vehicle count: {}", vehicles_count);
        self.update_grid(); // Update grid every time vehicles are set
    }*/

    // Add or update this method to populate the grid based on vehicles
    /*pub fn update_grid(&mut self) {
        // Clear the grid first
        self.grid = [[None; GRID_WIDTH]; GRID_WIDTH];
    
        for (index, vehicle) in self.vehicles.iter().enumerate() {
            println!("Placing vehicle on grid ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", vehicle_id, vehicle.size, vehicle.position, vehicle.orientation);
            
            let (x, y) = (vehicle.position.0 as usize, vehicle.position.1 as usize);
            /*let (width, height) = match vehicle.orientation {
                Orientation::Horizontal => (vehicle.size.0 as usize, 1),
                Orientation::Vertical => (1, vehicle.size.1 as usize),
            };*/
    
            // Check for overlap before placing the vehicle
            if self.grid[y][x].is_some() {
                // Handle the overlap scenario, e.g., log an error or adjust vehicle placement
                continue; // Skipping the current vehicle placement
            }
    
            // Placing the vehicle based on its orientation
            match vehicle.orientation {
                Orientation::Horizontal => {
                    for i in 0..vehicle.size.0 as usize {
                        if x + i < GRID_WIDTH {
                            if self.grid[y][x + i].is_some() {
                                println!("Overlap detected at position: ({}, {})", x + i, y);
                                break;
                            }
                            self.grid[y][x + i] = Some(index);
                        } else {
                            println!("Horizontal vehicle exceeds grid boundary at position: ({}, {})", x + i, y);
                            break;
                        }
                    }
                },
                Orientation::Vertical => {
                    for i in 0..vehicle.size.1 as usize {
                        if y + i < GRID_HEIGHT {
                            if self.grid[y + i][x].is_some() {
                                println!("Overlap detected at position: ({}, {})", x, y + i);
                                break;
                            }
                            self.grid[y + i][x] = Some(index);
                        } else {
                            println!("Vertical vehicle exceeds grid boundary at position: ({}, {})", x, y + i);
                            break;
                        }
                    }
                },
            }
        }
    } */ 
    
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

    pub fn update_grid_for_vehicle(&mut self, vehicle_id: usize) {
        // First, clear the old positions of the vehicle on the grid
        for row in self.grid.iter_mut() {
            for cell in row.iter_mut() {
                if *cell == Some(vehicle_id) {
                    *cell = None;
                }
            }
        }

        // Then, set the new positions of the vehicle
        if let Some(vehicle) = self.vehicles.iter().find(|v| v.id == vehicle_id) {
            let (x, y) = (vehicle.position.0 as usize, vehicle.position.1 as usize);
            match vehicle.orientation {
                Orientation::Horizontal => {
                    let end_x = std::cmp::min(x + vehicle.size.0 as usize, GRID_WIDTH);
                    for i in x..end_x {
                        self.grid[y][i] = Some(vehicle_id);
                    }
                },
                Orientation::Vertical => {
                    let end_y = std::cmp::min(y + vehicle.size.1 as usize, GRID_HEIGHT);
                    for i in y..end_y {
                        self.grid[i][x] = Some(vehicle_id);
                    }
                },
            }
        }        
    }

    /// Rebuild the occupancy grid from the authoritative vehicle list.
    /// The grid is used for collision/selection logic; vehicles are the
    /// authoritative representation of the puzzle state.
    pub fn rebuild_grid(&mut self) {
        self.grid = [[None; GRID_WIDTH]; GRID_HEIGHT];

        for (index, vehicle) in self.vehicles.iter().enumerate() {
            let (x, y) = vehicle.position;

            match vehicle.orientation {
                Orientation::Horizontal => {
                    for dx in 0..vehicle.size.0 as usize {
                        let cell_x = x as usize + dx;
                        if cell_x < GRID_WIDTH && (y as usize) < GRID_HEIGHT {
                            self.grid[y as usize][cell_x] = Some(index);
                        }
                    }
                }
                Orientation::Vertical => {
                    for dy in 0..vehicle.size.1 as usize {
                        let cell_y = y as usize + dy;
                        if cell_y < GRID_HEIGHT && (x as usize) < GRID_WIDTH {
                            self.grid[cell_y][x as usize] = Some(index);
                        }
                    }
                }
            }
        }
    }

    pub fn is_position_empty(&self, x: usize, y: usize) -> bool {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            false // If the index is out of bounds, return false
        } else {
            self.grid[y][x].is_none()
        }
    }

    pub fn is_occupied_by_vehicle(&self, x: usize, y: usize, vehicle_id: usize) -> bool {
        match self.grid[y][x] {
            Some(id) => id == vehicle_id,
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

    /// Returns every destination that this vehicle can reach in one player action.
    ///
    /// This is the shared movement rule used by both the live game and the puzzle
    /// solver. A vehicle may slide any number of cells along its own axis, but it
    /// may not pass through another vehicle.
    pub fn legal_move_destinations(&self, vehicle_index: usize) -> Vec<(u8, u8)> {
        let state: Vec<(u8, u8)> = self.vehicles.iter().map(|v| v.position).collect();
        Self::legal_move_destinations_for_state(&self.vehicles, &state, vehicle_index)
    }

    /// State-independent version of the movement rules used by the solver.
    ///
    /// Keeping this logic here means the player and BFS solver cannot silently
    /// drift apart in what they consider a legal move.
    pub fn legal_move_destinations_for_state(
        vehicles: &[Vehicle],
        state: &[(u8, u8)],
        vehicle_index: usize,
    ) -> Vec<(u8, u8)> {
        let vehicle = &vehicles[vehicle_index];
        let (current_x, current_y) = state[vehicle_index];
        let mut destinations = Vec::new();

        // Build the occupancy grid once for this state, excluding the vehicle
        // currently being moved. This preserves the game's movement semantics
        // while avoiding repeated vehicle-by-vehicle collision scans in the BFS.
        let mut occupancy = [[false; GRID_WIDTH]; GRID_HEIGHT];

        for (index, other) in vehicles.iter().enumerate() {
            if index == vehicle_index {
                continue;
            }

            let (x, y) = state[index];
            match other.orientation {
                Orientation::Horizontal => {
                    for dx in 0..other.size.0 as usize {
                        occupancy[y as usize][x as usize + dx] = true;
                    }
                }
                Orientation::Vertical => {
                    for dy in 0..other.size.1 as usize {
                        occupancy[y as usize + dy][x as usize] = true;
                    }
                }
            }
        }

        match vehicle.orientation {
            Orientation::Horizontal => {
                let max_x = GRID_WIDTH.saturating_sub(vehicle.size.0 as usize);
                let size = vehicle.size.0 as usize;

                for new_x in 0..=max_x {
                    let new_x = new_x as u8;
                    if new_x == current_x {
                        continue;
                    }

                    // A move sweeps the entire vehicle footprint from its old
                    // origin to its new origin. Any occupied cell in that swept
                    // corridor makes the player action illegal.
                    let start_x = current_x.min(new_x) as usize;
                    let end_x = current_x.max(new_x) as usize + size - 1;

                    let blocked = (start_x..=end_x)
                        .any(|x| occupancy[current_y as usize][x]);

                    if !blocked {
                        destinations.push((new_x, current_y));
                    }
                }
            }

            Orientation::Vertical => {
                let max_y = GRID_HEIGHT.saturating_sub(vehicle.size.1 as usize);
                let size = vehicle.size.1 as usize;

                for new_y in 0..=max_y {
                    let new_y = new_y as u8;
                    if new_y == current_y {
                        continue;
                    }

                    let start_y = current_y.min(new_y) as usize;
                    let end_y = current_y.max(new_y) as usize + size - 1;

                    let blocked = (start_y..=end_y)
                        .any(|y| occupancy[y][current_x as usize]);

                    if !blocked {
                        destinations.push((current_x, new_y));
                    }
                }
            }
        }

        destinations
    }

    /// Returns every legal one-action move for every vehicle in a state.
    ///
    /// This is the bulk form used by the solver and generator. It builds the
    /// occupancy grid only once for the whole state instead of rebuilding it
    /// once per vehicle. The moving vehicle's own cells are ignored when its
    /// swept corridor is checked.
    /// Returns every legal one-action move for every vehicle in a state.
    ///
    /// The 7x7 board fits in one u64 bitboard. This avoids constructing a
    /// 7x7 Option<usize> occupancy matrix and makes the hot solver loop mostly
    /// integer bit operations.
    pub fn legal_moves_for_state(
        vehicles: &[Vehicle],
        state: &[(u8, u8)],
    ) -> Vec<(usize, (u8, u8))> {
        debug_assert_eq!(vehicles.len(), state.len());

        let mut occupancy = 0u64;

        // Build the complete occupancy bitboard once.
        for (index, vehicle) in vehicles.iter().enumerate() {
            let (x, y) = state[index];

            match vehicle.orientation {
                Orientation::Horizontal => {
                    for dx in 0..vehicle.size.0 as usize {
                        let cell_x = x as usize + dx;
                        if cell_x < GRID_WIDTH && (y as usize) < GRID_HEIGHT {
                            occupancy |= 1u64 << (y as usize * GRID_WIDTH + cell_x);
                        }
                    }
                }
                Orientation::Vertical => {
                    for dy in 0..vehicle.size.1 as usize {
                        let cell_y = y as usize + dy;
                        if cell_y < GRID_HEIGHT && (x as usize) < GRID_WIDTH {
                            occupancy |= 1u64 << (cell_y * GRID_WIDTH + x as usize);
                        }
                    }
                }
            }
        }

        let mut moves = Vec::with_capacity(vehicles.len() * 3);

        for (vehicle_index, vehicle) in vehicles.iter().enumerate() {
            let (current_x, current_y) = state[vehicle_index];

            // Remove the moving vehicle's own cells from the occupancy mask.
            let own_mask = Self::vehicle_mask(vehicle, (current_x, current_y));
            let blocked = occupancy & !own_mask;

            match vehicle.orientation {
                Orientation::Horizontal => {
                    let max_x =
                        GRID_WIDTH.saturating_sub(vehicle.size.0 as usize);
                    let size = vehicle.size.0 as usize;
                    let row_offset =
                        current_y as usize * GRID_WIDTH;

                    for new_x in 0..=max_x {
                        let new_x = new_x as u8;
                        if new_x == current_x {
                            continue;
                        }

                        let start_x = current_x.min(new_x) as usize;
                        let end_x =
                            current_x.max(new_x) as usize + size - 1;

                        let length = end_x - start_x + 1;
                        let corridor_mask =
                            ((1u64 << length) - 1)
                                << (row_offset + start_x);

                        if blocked & corridor_mask == 0 {
                            moves.push((
                                vehicle_index,
                                (new_x, current_y),
                            ));
                        }
                    }
                }

                Orientation::Vertical => {
                    let max_y =
                        GRID_HEIGHT.saturating_sub(vehicle.size.1 as usize);
                    let size = vehicle.size.1 as usize;

                    for new_y in 0..=max_y {
                        let new_y = new_y as u8;
                        if new_y == current_y {
                            continue;
                        }

                        let start_y = current_y.min(new_y) as usize;
                        let end_y =
                            current_y.max(new_y) as usize + size - 1;

                        let mut corridor_mask = 0u64;
                        for y in start_y..=end_y {
                            corridor_mask |=
                                1u64 << (y * GRID_WIDTH + current_x as usize);
                        }

                        if blocked & corridor_mask == 0 {
                            moves.push((
                                vehicle_index,
                                (current_x, new_y),
                            ));
                        }
                    }
                }
            }
        }

        moves
    }

    /// Bit mask for one vehicle footprint.
    #[inline]
    fn vehicle_mask(
        vehicle: &Vehicle,
        position: (u8, u8),
    ) -> u64 {
        let (x, y) = position;
        let mut mask = 0u64;

        match vehicle.orientation {
            Orientation::Horizontal => {
                for dx in 0..vehicle.size.0 as usize {
                    let cell_x = x as usize + dx;
                    if cell_x < GRID_WIDTH && (y as usize) < GRID_HEIGHT {
                        mask |= 1u64
                            << (y as usize * GRID_WIDTH + cell_x);
                    }
                }
            }
            Orientation::Vertical => {
                for dy in 0..vehicle.size.1 as usize {
                    let cell_y = y as usize + dy;
                    if cell_y < GRID_HEIGHT && (x as usize) < GRID_WIDTH {
                        mask |= 1u64
                            << (cell_y * GRID_WIDTH + x as usize);
                    }
                }
            }
        }

        mask
    }

    /// Returns the number of legal destinations for each vehicle.
    /// Useful for heuristics that care about mobility without repeatedly
    /// rebuilding an occupancy grid.
    pub fn legal_move_counts_for_state(
        vehicles: &[Vehicle],
        state: &[(u8, u8)],
    ) -> Vec<usize> {
        let mut counts = vec![0usize; vehicles.len()];
        for (vehicle_index, _) in Self::legal_moves_for_state(vehicles, state) {
            counts[vehicle_index] += 1;
        }
        counts
    }

    /// Reproduces the click-position occupancy check performed by
    /// `handle_mouse_click`. For a right/down move the player clicks the
    /// new far end of the vehicle; for a left/up move the player clicks the
    /// new origin cell.
    fn click_cell_is_empty_for_state(
        vehicles: &[Vehicle],
        state: &[(u8, u8)],
        vehicle_index: usize,
        new_x: u8,
        new_y: u8,
    ) -> bool {
        let vehicle = &vehicles[vehicle_index];
        let (old_x, old_y) = state[vehicle_index];

        let (click_x, click_y) = match vehicle.orientation {
            Orientation::Horizontal => {
                if new_x < old_x {
                    (new_x, old_y)
                } else {
                    (new_x + vehicle.size.0 - 1, old_y)
                }
            }
            Orientation::Vertical => {
                if new_y < old_y {
                    (old_x, new_y)
                } else {
                    (old_x, new_y + vehicle.size.1 - 1)
                }
            }
        };

        for other_index in 0..vehicles.len() {
            if other_index == vehicle_index {
                continue;
            }

            let other = &vehicles[other_index];
            let (other_x, other_y) = state[other_index];

            match other.orientation {
                Orientation::Horizontal => {
                    let end_x = other_x + other.size.0;
                    if click_y == other_y && click_x >= other_x && click_x < end_x {
                        return false;
                    }
                }
                Orientation::Vertical => {
                    let end_y = other_y + other.size.1;
                    if click_x == other_x && click_y >= other_y && click_y < end_y {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Exact path-clearance rule used by both gameplay and the solver.
    /// The moving vehicle's own occupied cells are ignored; any other vehicle
    /// encountered between the old and new origins blocks the move.
    fn path_clear_for_state(
        vehicles: &[Vehicle],
        state: &[(u8, u8)],
        vehicle_index: usize,
        new_x: u8,
        new_y: u8,
    ) -> bool {
        let vehicle = &vehicles[vehicle_index];
        let (old_x, old_y) = state[vehicle_index];

        match vehicle.orientation {
            Orientation::Horizontal => {
                let (start_x, end_x) = if new_x > old_x {
                    (old_x, new_x)
                } else {
                    (new_x, old_x)
                };

                for other_index in 0..vehicles.len() {
                    if other_index == vehicle_index {
                        continue;
                    }

                    let other = &vehicles[other_index];
                    let (other_x, other_y) = state[other_index];

                    if other.orientation == Orientation::Horizontal {
                        let other_end_x = other_x + other.size.0;
                        if other_y == old_y
                            && (other_x..other_end_x).any(|x| x >= start_x && x <= end_x)
                        {
                            return false;
                        }
                    } else {
                        let other_end_y = other_y + other.size.1;
                        if other_x >= start_x
                            && other_x <= end_x
                            && old_y >= other_y
                            && old_y < other_end_y
                        {
                            return false;
                        }
                    }
                }
            }
            Orientation::Vertical => {
                let (start_y, end_y) = if new_y > old_y {
                    (old_y, new_y)
                } else {
                    (new_y, old_y)
                };

                for other_index in 0..vehicles.len() {
                    if other_index == vehicle_index {
                        continue;
                    }

                    let other = &vehicles[other_index];
                    let (other_x, other_y) = state[other_index];

                    if other.orientation == Orientation::Vertical {
                        let other_end_y = other_y + other.size.1;
                        if other_x == old_x
                            && (other_y..other_end_y).any(|y| y >= start_y && y <= end_y)
                        {
                            return false;
                        }
                    } else {
                        let other_end_x = other_x + other.size.0;
                        if old_x >= other_x
                            && old_x < other_end_x
                            && other_y >= start_y
                            && other_y <= end_y
                        {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    pub fn is_path_clear(&self, vehicle_index: usize, new_x: u8, new_y: u8) -> bool {
        let state: Vec<(u8, u8)> = self.vehicles.iter().map(|v| v.position).collect();
        Self::path_clear_for_state(
            &self.vehicles,
            &state,
            vehicle_index,
            new_x,
            new_y,
        )
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
        let red_vehicle = self.vehicles.iter()
            .find(|c| c.ansi_color == crate::vehicle_struct::AnsiColorCode::Red)
            .unwrap();

        red_vehicle.orientation == Orientation::Horizontal &&
        red_vehicle.position.1 == (GRID_HEIGHT / 2) as u8 &&
        red_vehicle.position.0 ==
            (GRID_WIDTH - red_vehicle.size.0 as usize) as u8
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
            self.grid[y][x] = Some(vehicle_index); // You need to get the updated vehicle data

            if vehicle.orientation == Orientation::Horizontal {
                for i in 1..vehicle.size.0 {
                    self.grid[y as usize][(((x + i as usize) as usize) as usize) as usize] = Some(vehicle_index);
                }
            } else {
                for i in 1..vehicle.size.1 {
                    self.grid[(((y + i as usize) as usize) as usize) as usize][x as usize] = Some(vehicle_index);
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
            clear([1.0; 4], gl);

            // Draw the 7x7 carpark background.
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    let px = x as f64 * CELL_SIZE;
                    let py = y as f64 * CELL_SIZE;

                    rectangle(
                        [0.8, 0.8, 0.8, 1.0],
                        [px, py, CELL_SIZE, CELL_SIZE],
                        c.transform,
                        gl,
                    );
                }
            }

            // Render from the authoritative vehicle list rather than from the
            // occupancy grid. This guarantees the GUI shows exactly the same
            // vehicle positions stored in game.vehicles.
            for vehicle in &self.vehicles {
                let vehicle_x = vehicle.position.0 as f64 * CELL_SIZE;
                let vehicle_y = vehicle.position.1 as f64 * CELL_SIZE;

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

                let bounds = [vehicle_x, vehicle_y, vehicle_width, vehicle_height];

                // Vehicle fill.
                rectangle(vehicle.color, bounds, c.transform, gl);

                // Explicit border so adjacent vehicles remain visually distinct.
                Rectangle::new_border([0.1, 0.1, 0.1, 1.0], 2.0)
                    .draw(bounds, &c.draw_state, c.transform, gl);
            }
        });
    }

    pub fn display_carpark(
        &self,
        vehicles: &[Vehicle],
        grid: &[[Option<usize>; GRID_WIDTH as usize]; GRID_WIDTH as usize]
    ) {
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