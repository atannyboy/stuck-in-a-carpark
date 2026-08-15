use crate::Game;
use crate::vehicle_struct::{VehicleStruct, Vehicle, Orientation, AnsiColorCode};
use rand::Rng;
use std::collections::{HashSet, VecDeque};
use rand::seq::SliceRandom;

pub const GRID_WIDTH: usize = 7;
pub const GRID_HEIGHT: usize = 7;

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
	pub vehicle_struct: VehicleStruct,
	used_colors: HashSet<String>, // Using a HashSet to efficiently track used colors
}

#[derive(Clone, Debug)]
pub struct SolverMove {
    pub vehicle_index: usize,
    pub from: (u8, u8),
    pub to: (u8, u8),
}

#[derive(Clone, Debug)]
pub struct PuzzleSolution {
    pub moves: Vec<SolverMove>,
    pub explored_states: usize,
    pub dead_end_states: usize,
    pub decision_states: usize,
    pub total_legal_moves: usize,
    pub difficulty_score: usize,
}

impl PuzzleSolution {
    pub fn solution_moves(&self) -> usize {
        self.moves.len()
    }

    fn average_branching(&self) -> f64 {
        if self.explored_states == 0 {
            0.0
        } else {
            self.total_legal_moves as f64 / self.explored_states as f64
        }
    }
}

impl PuzzleGenerator {
	pub fn new() -> Self {
		let vehicle_struct = VehicleStruct::new();
		let used_colors = HashSet::new();
		PuzzleGenerator { vehicle_struct, used_colors }
	}

    /// Generate puzzles by searching *backwards* from a solved state.
    ///
    /// The critical difference from a random scramble is that we explicitly
    /// search for states that are several moves away from the solved state.
    /// Because the move graph is generated with the same legal-move rules as
    /// the real game, a state found at reverse-search depth N has a legal path
    /// back toward the solved state. The exact forward BFS remains the authority
    /// on the final minimum solution length.
    pub fn generate_puzzle(&mut self, game: &mut Game) -> Vec<Vehicle> {
        // Fast stochastic deep-search generation.
        //
        // IMPORTANT: the reverse search starts from the actual solved state
        // with the red car at the exit and MUST allow the red car to move.
        // Freezing the red car at the puzzle start before the reverse search
        // creates a different move graph and makes reverse depth meaningless
        // as a proxy for actual puzzle difficulty.
        //
        // We therefore search the same state graph used by the exact solver,
        // then use the exact solver to measure the final minimum solution.
        // Search harder for the 14-move target without exploding the exact-BFS
        // workload. The main changes are:
        //   * more independent layouts per batch;
        //   * a deeper reverse search so candidate states are farther from solved;
        //   * a wider beam and more sampled moves for better coverage;
        //   * more final candidates per layout;
        //   * a vehicle-count bias toward 12-15 vehicles, which has produced
        //     more difficult boards in practice.
        const MAX_BATCHES: usize = 14;
        const LAYOUTS_PER_BATCH: usize = 20;

        const MIN_VEHICLES: usize = 12;
        const MAX_VEHICLES: usize = 15;
        const MIN_ACCEPTABLE_MOVES: usize = 11;
        const TARGET_MOVES: usize = 20;

        const REVERSE_DEPTH: usize = 24;
        const BEAM_WIDTH: usize = 700;
        const SAMPLED_MOVES_PER_STATE: usize = 24;
        const CANDIDATES_PER_LAYOUT: usize = 20;

        const MAX_EXACT_SEARCH_STATES: usize = 3_000_000;

        let mut best_global: Option<(Vec<Vehicle>, PuzzleSolution)> = None;

        for batch in 1..=MAX_BATCHES {
            println!(
                "\n=== Generating 10-move puzzle batch {}/{} ===",
                batch, MAX_BATCHES
            );

            let mut best_batch: Option<(Vec<Vehicle>, PuzzleSolution)> = None;

            for layout_number in 1..=LAYOUTS_PER_BATCH {
                game.vehicles.clear();
                game.grid = [[None; GRID_WIDTH]; GRID_HEIGHT];
                game.selected_vehicle_index = None;
                self.used_colors.clear();

                let mut vehicle_id = 0usize;

                // Build the solved layout with the red car at the exit.
                self.place_red_car_at_exit(game, &mut vehicle_id);

                let mut rng = rand::thread_rng();

                // Bias layouts toward the denser end of the new 12-15 vehicle
                // range. Denser boards create more opportunities for blocker
                // dependencies, while retaining some 12-vehicle layouts gives
                // us useful variation for comparison.
                let roll = rng.gen_range(0..100);
                let target_vehicle_count = match roll {
                    0..=14 => 12,  // 15%
                    15..=39 => 13, // 25%
                    40..=74 => 14, // 35%
                    _ => 15,       // 25%
                };

                while game.vehicles.len() < target_vehicle_count {
                    let before = game.vehicles.len();
                    self.add_vehicle_strategically(game, &mut vehicle_id);

                    if game.vehicles.len() == before {
                        break;
                    }
                }

                if game.vehicles.len() < MIN_VEHICLES {
                    println!(
                        "Layout {} could not reach {} vehicles. Skipping.",
                        layout_number, MIN_VEHICLES
                    );
                    continue;
                }

                println!(
                    "Layout {}: {} vehicles. Fast reverse search from solved state...",
                    layout_number,
                    game.vehicles.len()
                );

                let vehicle_template = game.vehicles.clone();

                let reverse_candidates = self.find_fast_reverse_candidates(
                    &vehicle_template,
                    REVERSE_DEPTH,
                    BEAM_WIDTH,
                    SAMPLED_MOVES_PER_STATE,
                    CANDIDATES_PER_LAYOUT,
                    &mut rng,
                );

                if reverse_candidates.is_empty() {
                    println!(
                        "Layout {} produced no deep reverse candidates.",
                        layout_number
                    );
                    continue;
                }

                // Keep the best candidate for this layout instead of accepting
                // the first one that reaches the target. This removes a major
                // source of run-to-run variance.
                let mut best_layout: Option<(Vec<Vehicle>, PuzzleSolution)> = None;

                for (candidate_number, (state, reverse_depth, heuristic)) in
                    reverse_candidates.into_iter().enumerate()
                {
                    game.vehicles = vehicle_template.clone();
                    Self::apply_state(&mut game.vehicles, &state);
                    game.rebuild_grid();

                    println!(
                        "Testing layout {}, candidate {}: reverse depth {}, heuristic {}.",
                        layout_number,
                        candidate_number + 1,
                        reverse_depth,
                        heuristic
                    );

                    // The starting puzzle must actually trap the red car.
                    if !self.red_car_is_blocked(game) {
                        continue;
                    }

                    // The exact BFS is the single source of truth for difficulty.
                    // There is no separate quick BFS, so a state-limit in a preliminary
                    // search can no longer accidentally allow an easy puzzle through.
                    let Some(solution) = self.solve_puzzle_with_limit(
                        game,
                        MAX_EXACT_SEARCH_STATES,
                    ) else {
                        println!("  Exact solver exceeded its search limit.");
                        continue;
                    };

                    println!(
                        "  Exact solution: {} minimum moves, score {}, explored {} states.",
                        solution.solution_moves(),
                        solution.difficulty_score,
                        solution.explored_states
                    );

                    // Only harden genuinely promising candidates. Hardening is
                    // an expensive local-search operation, so do not spend many
                    // exact solves on very easy 3-10 move puzzles. Start the
                    // hardening frontier at the minimum acceptable difficulty;
                    // once a stronger global record exists, allow a small
                    // three-move look-back below that record.
                    let current_record_moves = best_global
                        .as_ref()
                        .map(|(_, record)| record.solution_moves())
                        .unwrap_or(MIN_ACCEPTABLE_MOVES);

                    let hardening_floor =
                        MIN_ACCEPTABLE_MOVES.max(current_record_moves.saturating_sub(3));

                    let (candidate_vehicles, candidate_solution) =
                        if solution.solution_moves() >= hardening_floor
                            && solution.solution_moves() < TARGET_MOVES
                        {
                            self.harden_candidate(
                                game,
                                &solution,
                                TARGET_MOVES,
                                28,
                                MAX_EXACT_SEARCH_STATES,
                            )
                            .unwrap_or((game.vehicles.clone(), solution.clone()))
                        } else {
                            (game.vehicles.clone(), solution.clone())
                        };

                    game.vehicles = candidate_vehicles.clone();
                    game.rebuild_grid();

                    if best_layout
                        .as_ref()
                        .map(|(_, current)| Self::solution_is_better(&candidate_solution, current))
                        .unwrap_or(true)
                    {
                        best_layout =
                            Some((candidate_vehicles.clone(), candidate_solution.clone()));
                    }

                    let is_better_than_batch = best_batch
                        .as_ref()
                        .map(|(_, current)| {
                            Self::solution_is_better(&candidate_solution, current)
                        })
                        .unwrap_or(true);

                    if is_better_than_batch {
                        best_batch =
                            Some((candidate_vehicles.clone(), candidate_solution.clone()));
                    }
                }

                // Only accept after all candidates for this layout have been
                // evaluated. This makes the result much more stable and ensures
                // that a mediocre 10-move candidate cannot hide a harder one.
                if let Some((vehicles, solution)) = &best_layout {
                    println!(
                        "Best puzzle in batch {}: {} vehicles, {} minimum moves, difficulty score {}.",
                        batch,
                        vehicles.len(),
                        solution.solution_moves(),
                        solution.difficulty_score
                    );

                    // Only stop early once the explicit target has been reached.
                    if solution.solution_moves() >= TARGET_MOVES {
                        println!(
                            "Target reached: {} minimum moves. Selecting puzzle.",
                            solution.solution_moves()
                        );
                        self.print_solution(solution, vehicles);
                        return vehicles.clone();
                    }
                }
            }

            if let Some((vehicles, solution)) = best_batch {
                println!(
                    "Hardest candidate in batch {}: {} minimum moves.",
                    batch,
                    solution.solution_moves()
                );

                let improves_global = best_global
                    .as_ref()
                    .map(|(_, current)| {
                        Self::solution_is_better(&solution, current)
                    })
                    .unwrap_or(true);

                if improves_global {
                    best_global =
                        Some((vehicles.clone(), solution.clone()));
                }
            } else {
                println!(
                    "Batch {} produced no candidates that reached the exact solver.",
                    batch
                );
            }
        }

        if let Some((vehicles, solution)) = best_global {
            println!(
                "Search budget exhausted. Selecting hardest puzzle found: {} minimum moves, difficulty score {}.",
                solution.solution_moves(),
                solution.difficulty_score
            );
            self.print_solution(&solution, &vehicles);
            return vehicles;
        }

        panic!(
            "Unable to generate any puzzle after {} batches.",
            MAX_BATCHES
        );
    }

    /// Search puzzle-space around a near-miss.
    ///
    /// The old hardening routine was a greedy hill-climber: if none of a few
    /// one-move mutations improved the exact solution, it stopped immediately.
    /// That makes it very easy to get trapped at 9 or 10 moves.
    ///
    /// This version:
    ///   * considers both one-move and two-move mutations;
    ///   * ranks mutations cheaply before exact BFS;
    ///   * allows plateau/worse exploratory moves to escape local maxima;
    ///   * keeps the globally hardest puzzle found;
    ///   * never accepts a candidate as "harder" without the exact solver.
    fn harden_candidate(
        &self,
        game: &mut Game,
        current_solution: &PuzzleSolution,
        target_moves: usize,
        attempts: usize,
        max_states: usize,
    ) -> Option<(Vec<Vehicle>, PuzzleSolution)> {
        let red_index = game
            .vehicles
            .iter()
            .position(|v| v.ansi_color == AnsiColorCode::Red)?;

        let mut rng = rand::thread_rng();

        let mut search_vehicles = game.vehicles.clone();
        let mut search_solution = current_solution.clone();

        let mut best_vehicles = search_vehicles.clone();
        let mut best_solution = current_solution.clone();

        let mut tried: HashSet<u128> = HashSet::new();
        let initial_state: Vec<(u8, u8)> =
            search_vehicles.iter().map(|v| v.position).collect();
        tried.insert(Self::encode_state(&initial_state));

        let mut stagnation = 0usize;

        for _iteration in 0..attempts {
            let state: Vec<(u8, u8)> =
                search_vehicles.iter().map(|v| v.position).collect();

            // Candidate = (state, structural score).
            let mut options: Vec<(Vec<(u8, u8)>, usize)> = Vec::new();

            // -------------------------------------------------------------
            // 1. One-move mutations.
            // -------------------------------------------------------------
            let legal_moves = Game::legal_moves_for_state(&search_vehicles, &state);

            for (vehicle_index, destination) in legal_moves {
                if vehicle_index == red_index {
                    continue;
                }

                let mut next_state = state.clone();
                next_state[vehicle_index] = destination;
                let key = Self::encode_state(&next_state);

                if tried.insert(key) {
                    let heuristic = Self::fast_reverse_heuristic(
                        &search_vehicles,
                        &next_state,
                        red_index,
                        target_moves,
                    );
                    options.push((next_state, heuristic));
                }
            }

            // -------------------------------------------------------------
            // 2. Two-move mutations.
            //
            // These are important because a single legal relocation can
            // easily make a puzzle easier, while two coordinated relocations
            // can create a new blocker chain.
            // -------------------------------------------------------------
            let mut first_moves = Game::legal_moves_for_state(&search_vehicles, &state);
            first_moves.sort_unstable_by_key(|(vehicle_index, destination)| {
                let mut probe = state.clone();
                probe[*vehicle_index] = *destination;
                std::cmp::Reverse(Self::fast_reverse_heuristic(&search_vehicles, &probe, red_index, target_moves))
            });
            first_moves.truncate(14);

            for (first_vehicle, first_destination) in first_moves {
                if first_vehicle == red_index {
                    continue;
                }

                let mut intermediate = state.clone();
                intermediate[first_vehicle] = first_destination;

                let second_moves =
                    Game::legal_moves_for_state(&search_vehicles, &intermediate);

                for (second_vehicle, second_destination) in second_moves {
                    if second_vehicle == red_index {
                        continue;
                    }

                    // Avoid immediately undoing the first mutation.
                    if second_vehicle == first_vehicle
                        && second_destination == state[first_vehicle]
                    {
                        continue;
                    }

                    let mut next_state = intermediate.clone();
                    next_state[second_vehicle] = second_destination;
                    let key = Self::encode_state(&next_state);

                    if tried.insert(key) {
                        let heuristic = Self::fast_reverse_heuristic(
                            &search_vehicles,
                            &next_state,
                            red_index,
                            target_moves,
                        );
                        options.push((next_state, heuristic));
                    }
                }
            }

            // During stagnation, add a very small three-move neighborhood.
            // This is deliberately bounded: six first moves, three second
            // moves, and two third moves per branch. It gives the hardener a
            // chance to discover coordinated dependency changes that cannot be
            // reached by a single or double mutation without blowing up the
            // exact-solver workload.
            if stagnation >= 1 || search_solution.solution_moves() + 2 < target_moves {
                let seeds = Game::legal_moves_for_state(&search_vehicles, &state);
                let mut ranked_seeds: Vec<(Vec<(u8, u8)>, usize)> = Vec::new();

                for (vehicle_index, destination) in seeds {
                    if vehicle_index == red_index {
                        continue;
                    }
                    let mut next = state.clone();
                    next[vehicle_index] = destination;
                    let key = Self::encode_state(&next);
                    if tried.contains(&key) {
                        continue;
                    }
                    let score = Self::fast_reverse_heuristic(
                        &search_vehicles, &next, red_index, target_moves
                    );
                    ranked_seeds.push((next, score));
                }

                ranked_seeds.sort_unstable_by(|a, b| b.1.cmp(&a.1));

                for (seed, _) in ranked_seeds.into_iter().take(6) {
                    let second_moves = Game::legal_moves_for_state(&search_vehicles, &seed);
                    let mut ranked_second: Vec<(Vec<(u8, u8)>, usize)> = Vec::new();

                    for (vehicle_index, destination) in second_moves {
                        if vehicle_index == red_index {
                            continue;
                        }
                        let mut mid = seed.clone();
                        mid[vehicle_index] = destination;
                        let score = Self::fast_reverse_heuristic(
                            &search_vehicles, &mid, red_index, target_moves
                        );
                        ranked_second.push((mid, score));
                    }

                    ranked_second.sort_unstable_by(|a, b| b.1.cmp(&a.1));

                    for (mid, _) in ranked_second.into_iter().take(3) {
                        let third_moves = Game::legal_moves_for_state(&search_vehicles, &mid);
                        let mut ranked_third: Vec<(Vec<(u8, u8)>, usize)> = Vec::new();

                        for (vehicle_index, destination) in third_moves {
                            if vehicle_index == red_index {
                                continue;
                            }
                            let mut final_state = mid.clone();
                            final_state[vehicle_index] = destination;
                            let key = Self::encode_state(&final_state);
                            if tried.contains(&key) {
                                continue;
                            }
                            let score = Self::fast_reverse_heuristic(
                                &search_vehicles, &final_state, red_index, target_moves
                            );
                            ranked_third.push((final_state, score));
                        }

                        ranked_third.sort_unstable_by(|a, b| b.1.cmp(&a.1));
                        options.extend(ranked_third.into_iter().take(2));
                    }
                }
            }

            if options.is_empty() {
                break;
            }

            options.sort_unstable_by(|a, b| b.1.cmp(&a.1));

            // Normally test the best few. During stagnation, deliberately
            // include one lower-ranked mutation to escape local maxima.
            let exact_tests = if stagnation >= 2 { 8usize } else { 6usize };
            let top_count = options.len().min(exact_tests);

            let mut test_indices: Vec<usize> = (0..top_count).collect();

            if stagnation >= 2 && options.len() > top_count {
                let diversity_index = top_count
                    + rng.gen_range(0..(options.len() - top_count));
                test_indices.push(diversity_index);
            }

            // Shuffle the selected indices very slightly so ties do not
            // repeatedly choose the same structural mutation.
            test_indices.shuffle(&mut rng);

            let mut best_tested_state: Option<(Vec<(u8, u8)>, PuzzleSolution, usize)> = None;

            for index in test_indices {
                let (candidate_state, heuristic) = &options[index];

                let mut candidate_game = Game::new();
                candidate_game.vehicles = search_vehicles.clone();
                Self::apply_state(&mut candidate_game.vehicles, candidate_state);
                candidate_game.rebuild_grid();

                if !self.red_car_is_blocked(&candidate_game) {
                    continue;
                }

                let Some(solution) = self.solve_puzzle_with_limit(
                    &candidate_game,
                    max_states,
                ) else {
                    continue;
                };

                println!(
                    "    Mutation exact solve: {} moves, explored {} states.",
                    solution.solution_moves(),
                    solution.explored_states
                );

                if best_tested_state
                    .as_ref()
                    .map(|(_, best, _)| Self::solution_is_better(&solution, best))
                    .unwrap_or(true)
                {
                    best_tested_state = Some((
                        candidate_state.clone(),
                        solution,
                        *heuristic,
                    ));
                }
            }

            let Some((chosen_state, chosen_solution, _chosen_heuristic)) =
                best_tested_state
            else {
                break;
            };

            let improved_search =
                Self::solution_is_better(&chosen_solution, &search_solution);

            let improved_global =
                Self::solution_is_better(&chosen_solution, &best_solution);

            // Always retain the global best.
            if improved_global {
                best_solution = chosen_solution.clone();
                best_vehicles = {
                    let mut vehicles = search_vehicles.clone();
                    Self::apply_state(&mut vehicles, &chosen_state);
                    vehicles
                };

                println!(
                    "    Hardening record: {} moves.",
                    best_solution.solution_moves()
                );

                if best_solution.solution_moves() >= target_moves {
                    game.vehicles = best_vehicles.clone();
                    game.rebuild_grid();
                    return Some((best_vehicles, best_solution));
                }
            }

            // Move the exploratory search point. This is the key difference
            // from the old greedy version: an exploratory move may keep the
            // same solution length (or even be worse), allowing the search to
            // reach a different region of puzzle-space.
            let accept_exploration =
                improved_search
                || stagnation >= 2
                || chosen_solution.solution_moves() + 1 >= search_solution.solution_moves();

            if accept_exploration {
                search_vehicles = {
                    let mut vehicles = search_vehicles.clone();
                    Self::apply_state(&mut vehicles, &chosen_state);
                    vehicles
                };
                search_solution = chosen_solution;

                if improved_search {
                    stagnation = 0;
                } else {
                    stagnation += 1;
                }
            } else {
                stagnation += 1;
            }

            // Keep stagnation bounded. A new local branch starts from the
            // globally best puzzle rather than wandering forever.
            if stagnation >= 5 {
                search_vehicles = best_vehicles.clone();
                search_solution = best_solution.clone();
                stagnation = 0;

                println!(
                    "    Hardening restart from best puzzle ({} moves).",
                    best_solution.solution_moves()
                );
            }
        }

        game.vehicles = best_vehicles.clone();
        game.rebuild_grid();

        if Self::solution_is_better(&best_solution, current_solution) {
            Some((best_vehicles, best_solution))
        } else {
            None
        }
    }

    /// Fast sampled reverse search.
    ///
    /// Unlike the previous large beam search, this does not expand every
    /// possible successor from every beam state. It samples a small number of
    /// legal moves per state and ranks the resulting states with a cheap
    /// geometry-only heuristic.
    fn find_fast_reverse_candidates<R: rand::Rng + ?Sized>(
        &self,
        vehicles: &[Vehicle],
        reverse_depth: usize,
        beam_width: usize,
        sampled_moves_per_state: usize,
        max_candidates: usize,
        rng: &mut R,
    ) -> Vec<(Vec<(u8, u8)>, usize, usize)> {
        type State = Vec<(u8, u8)>;

        let red_index = match vehicles.iter().position(|v| {
            v.ansi_color == AnsiColorCode::Red
                && v.orientation == Orientation::Horizontal
        }) {
            Some(index) => index,
            None => return Vec::new(),
        };

        let start: State = vehicles.iter().map(|v| v.position).collect();
        let mut frontier = vec![start.clone()];

        // Reserve enough space for the stochastic beam's working set.
        let mut visited: HashSet<u128> =
            HashSet::with_capacity(beam_width.saturating_mul(reverse_depth + 1));
        visited.insert(Self::encode_state(&start));

        for depth in 1..=reverse_depth {
            let mut next_frontier: Vec<(State, usize)> = Vec::new();

            for state in &frontier {
                let mut moves: Vec<(usize, (u8, u8))> = Vec::new();

                // Generate all moves with one occupancy build for this state.
                moves.extend(Game::legal_moves_for_state(vehicles, state));

                if moves.is_empty() {
                    continue;
                }

                // Sample UNIQUE legal moves. Sampling with replacement can
                // waste much of the sample on duplicates.
                moves.shuffle(rng);
                let sample_count = moves.len().min(sampled_moves_per_state);

                for &(vehicle_index, destination) in moves.iter().take(sample_count) {
                    let mut next_state = state.clone();
                    next_state[vehicle_index] = destination;

                    let key = Self::encode_state(&next_state);

                    if !visited.insert(key) {
                        continue;
                    }

                    let score = Self::fast_reverse_heuristic(
                        vehicles,
                        &next_state,
                        red_index,
                        depth,
                    );

                    next_frontier.push((next_state, score));
                }
            }

            if next_frontier.is_empty() {
                break;
            }

            // Keep only the most promising states. This is deliberately a
            // small beam; the exact solver decides the real difficulty.
            next_frontier.sort_unstable_by(|a, b| b.1.cmp(&a.1));

            if next_frontier.len() > beam_width {
                next_frontier.truncate(beam_width);
            }

            if depth == reverse_depth {
                let mut candidates = Vec::new();

                for (state, heuristic) in next_frontier {
                    if !Self::is_valid_start_state(
                        vehicles,
                        &state,
                        red_index,
                    ) {
                        continue;
                    }

                    candidates.push((
                        state,
                        depth,
                        heuristic,
                    ));
                }

                candidates.sort_unstable_by(|a, b| b.2.cmp(&a.2));

                // Keep a mixture of the strongest states and a few lower-ranked
                // states. The latter are deliberately included because a purely
                // heuristic beam can repeatedly select "good-looking" but shallow
                // puzzles. This gives the exact solver more genuinely different
                // structures to evaluate.
                let strong_count = max_candidates.saturating_mul(3) / 4;
                let diverse_count = max_candidates.saturating_sub(strong_count);

                let mut selected = Vec::with_capacity(max_candidates);
                selected.extend(candidates.iter().take(strong_count).cloned());

                if diverse_count > 0 && candidates.len() > strong_count {
                    let tail_start = strong_count;
                    let tail_len = candidates.len() - tail_start;
                    for i in 0..diverse_count {
                        let index = tail_start + (i * tail_len / diverse_count);
                        if let Some(candidate) = candidates.get(index.min(candidates.len() - 1)) {
                            if !selected.iter().any(|existing: &(State, usize, usize)| {
                                Self::encode_state(&existing.0) == Self::encode_state(&candidate.0)
                            }) {
                                selected.push(candidate.clone());
                            }
                        }
                    }
                }

                selected.truncate(max_candidates);
                return selected;
            }

            frontier = next_frontier
                .into_iter()
                .map(|(state, _)| state)
                .collect();
        }

        Vec::new()
    }

    /// Very cheap reverse-search heuristic.
    ///
    /// This intentionally does NOT call legal_move_destinations_for_state().
    /// The old heuristic did that for every state and every vehicle, which was
    /// the main reason the generator spent so long inside reverse search.
    fn fast_reverse_heuristic(
        vehicles: &[Vehicle],
        state: &[(u8, u8)],
        red_index: usize,
        depth: usize,
    ) -> usize {
        let center_row = GRID_HEIGHT / 2;
        let red = &vehicles[red_index];
        let (red_x, red_y) = state[red_index];

        let mut score = depth * 10_000;

        // Prefer denser layouts when structural quality is otherwise similar.
        // This is only a construction heuristic; the exact solver still
        // determines the real minimum solution length.
        score += match vehicles.len() {
            12 => 0,
            13 => 1_000,
            14 => 2_500,
            15 => 4_000,
            _ => 0,
        };

        // Strongly prefer states where the red car has travelled all the way
        // back to its actual starting position.
        score += (GRID_WIDTH - red_x as usize) * 3_500;

        if red_x == 0 && red_y as usize == center_row {
            score += 60_000;
        }

        let exit_start = red_x as usize + red.size.0 as usize;

        // Count occupied cells in the red-car exit corridor.
        let mut blocked_exit_cells = 0usize;
        for x in exit_start..GRID_WIDTH {
            let blocked = vehicles.iter().enumerate().any(|(index, vehicle)| {
                if index == red_index {
                    return false;
                }

                let (vx, vy) = state[index];

                match vehicle.orientation {
                    Orientation::Horizontal => {
                        vy as usize == center_row
                            && x >= vx as usize
                            && x < vx as usize + vehicle.size.0 as usize
                    }

                    Orientation::Vertical => {
                        vx as usize == x
                            && center_row >= vy as usize
                            && center_row < vy as usize + vehicle.size.1 as usize
                    }
                }
            });

            if blocked {
                blocked_exit_cells += 1;
            }
        }

        score += blocked_exit_cells * 3_000;

        // Encourage vehicles to participate in the red-car corridor, but also
        // reward spread across the board so the beam does not converge onto
        // several visually similar states.
        let mut near_corridor = 0usize;
        let mut occupied_cells = 0usize;

        for index in 0..vehicles.len() {
            if index == red_index {
                continue;
            }

            let vehicle = &vehicles[index];
            let (vx, vy) = state[index];

            let footprint_start;
            let footprint_end;

            match vehicle.orientation {
                Orientation::Horizontal => {
                    footprint_start = vx as usize;
                    footprint_end = vx as usize + vehicle.size.0 as usize;
                    occupied_cells += vehicle.size.0 as usize;

                    if vy as usize == center_row
                        && footprint_end > exit_start
                        && footprint_start < GRID_WIDTH
                    {
                        near_corridor += 1;
                    }
                }

                Orientation::Vertical => {
                    footprint_start = vy as usize;
                    footprint_end = vy as usize + vehicle.size.1 as usize;
                    occupied_cells += vehicle.size.1 as usize;

                    if vx as usize >= exit_start
                        && (vx as usize) < GRID_WIDTH
                        && footprint_start <= center_row
                        && footprint_end > center_row
                    {
                        near_corridor += 1;
                    }
                }
            }
        }

        score += near_corridor * 750;

        // Denser occupation is mildly preferred because it creates more
        // opportunities for blocker chains without being the primary criterion.
        score += occupied_cells.min(42) * 55;

        score
    }

    fn solution_is_better(
        candidate: &PuzzleSolution,
        current: &PuzzleSolution,
    ) -> bool {
        candidate.solution_moves() > current.solution_moves()
            || (candidate.solution_moves() == current.solution_moves()
                && candidate.difficulty_score > current.difficulty_score)
    }

    fn is_valid_start_state(
        vehicles: &[Vehicle],
        state: &[(u8, u8)],
        red_index: usize,
    ) -> bool {
        let red = &vehicles[red_index];
        let center_row = (GRID_HEIGHT / 2) as u8;

        if state[red_index].0 != 0 || state[red_index].1 != center_row {
            return false;
        }

        let first_exit_cell =
            state[red_index].0 as usize + red.size.0 as usize;

        (first_exit_cell..GRID_WIDTH).any(|x| {
            vehicles.iter().enumerate().any(|(index, vehicle)| {
                if index == red_index {
                    return false;
                }

                let (vx, vy) = state[index];

                match vehicle.orientation {
                    Orientation::Horizontal => {
                        vy as usize == center_row as usize
                            && (vx as usize..vx as usize + vehicle.size.0 as usize)
                                .any(|cell| cell == x)
                    }
                    Orientation::Vertical => {
                        vx as usize == x
                            && (vy as usize..vy as usize + vehicle.size.1 as usize)
                                .contains(&(center_row as usize))
                    }
                }
            })
        })
    }

    fn apply_state(vehicles: &mut [Vehicle], state: &[(u8, u8)]) {
        for (vehicle, position) in vehicles.iter_mut().zip(state.iter()) {
            vehicle.position = *position;
        }
    }

    fn place_red_car_at_exit(
        &mut self,
        game: &mut Game,
        vehicle_id: &mut usize,
    ) {
        let red_car = Vehicle {
            id: *vehicle_id,
            size: (2, 1),
            color: AnsiColorCode::Red.to_rgba(),
            orientation: Orientation::Horizontal,
            position: (
                (GRID_WIDTH - 2) as u8,
                (GRID_HEIGHT / 2) as u8,
            ),
            ansi_color: AnsiColorCode::Red,
        };

        game.update_grid_with_new_vehicle(&red_car, *vehicle_id);
        game.vehicles.push(red_car);
        *vehicle_id += 1;
    }

    fn state_allows_red_to_left(vehicles: &[Vehicle]) -> bool {
        let red_index = match vehicles.iter().position(|v| {
            v.ansi_color == AnsiColorCode::Red
                && v.orientation == Orientation::Horizontal
        }) {
            Some(index) => index,
            None => return false,
        };

        let state: Vec<(u8, u8)> =
            vehicles.iter().map(|v| v.position).collect();

        Game::legal_move_destinations_for_state(
            vehicles,
            &state,
            red_index,
        )
        .contains(&(0, (GRID_HEIGHT / 2) as u8))
    }

    fn scramble_candidate<R: rand::Rng + ?Sized>(
        &self,
        game: &mut Game,
        target_moves: usize,
        rng: &mut R,
    ) -> bool {
        let red_index = match game.vehicles.iter().position(|v| {
            v.ansi_color == AnsiColorCode::Red
        }) {
            Some(index) => index,
            None => return false,
        };

        let mut seen_states = HashSet::new();
        let initial_state: Vec<(u8, u8)> =
            game.vehicles.iter().map(|v| v.position).collect();
        seen_states.insert(Self::encode_state(&initial_state));

        let mut last_move: Option<(usize, (u8, u8), (u8, u8))> = None;

        for step in 0..target_moves {
            let state: Vec<(u8, u8)> =
                game.vehicles.iter().map(|v| v.position).collect();

            let mut options: Vec<(usize, (u8, u8), usize)> = Vec::new();

            for (vehicle_index, destination) in
                Game::legal_moves_for_state(&game.vehicles, &state)
            {
                if vehicle_index == red_index {
                    continue;
                }

                // Never immediately undo the previous scramble move.
                if let Some((last_vehicle, last_from, _last_to)) = last_move {
                    if vehicle_index == last_vehicle && destination == last_from {
                        continue;
                    }
                }

                let mut candidate_state = state.clone();
                candidate_state[vehicle_index] = destination;
                let key = Self::encode_state(&candidate_state);

                if !seen_states.contains(&key) {
                    let old_position = game.vehicles[vehicle_index].position;
                    game.vehicles[vehicle_index].position = destination;
                    game.rebuild_grid();

                    let heuristic =
                        self.scramble_move_score(game, red_index, step, target_moves);

                    game.vehicles[vehicle_index].position = old_position;
                    game.rebuild_grid();

                    options.push((vehicle_index, destination, heuristic));
                }
            }

            if options.is_empty() {
                return false;
            }

            // Keep the search stochastic, but heavily favour moves that create
            // direct red blockers, constrain those blockers, or create deeper
            // dependency chains.
            options.sort_by(|a, b| b.2.cmp(&a.2));
            let top_n = options.len().min(8);
            let &(vehicle_index, destination, _) = options[..top_n]
                .choose(rng)
                .expect("top candidate slice is non-empty");

            let old_position = game.vehicles[vehicle_index].position;
            game.vehicles[vehicle_index].position = destination;
            game.rebuild_grid();

            let new_state: Vec<(u8, u8)> =
                game.vehicles.iter().map(|v| v.position).collect();
            seen_states.insert(Self::encode_state(&new_state));

            last_move = Some((vehicle_index, old_position, destination));
        }

        // Prefer a final arrangement in which the red car is blocked. If it is
        // not blocked yet, greedily choose a legal move that increases the
        // dependency score; this is still a legal reverse move from the solved state.
        for _ in 0..16 {
            if self.red_car_is_blocked(game) {
                return true;
            }

            let current_score = self.dependency_score(game);
            let state: Vec<(u8, u8)> =
                game.vehicles.iter().map(|v| v.position).collect();
            let mut best_moves: Vec<(usize, (u8, u8), usize)> = Vec::new();

            for vehicle_index in 0..game.vehicles.len() {
                if vehicle_index == red_index {
                    continue;
                }

                for destination in Game::legal_move_destinations_for_state(
                    &game.vehicles,
                    &state,
                    vehicle_index,
                ) {
                    let old_position = game.vehicles[vehicle_index].position;
                    game.vehicles[vehicle_index].position = destination;
                    game.rebuild_grid();

                    let new_score = self.dependency_score(game);
                    if new_score > current_score {
                        best_moves.push((vehicle_index, destination, new_score));
                    }

                    game.vehicles[vehicle_index].position = old_position;
                    game.rebuild_grid();
                }
            }

            if best_moves.is_empty() {
                break;
            }

            let best_score = best_moves.iter().map(|m| m.2).max().unwrap();
            let top: Vec<_> = best_moves
                .into_iter()
                .filter(|m| m.2 == best_score)
                .collect();
            let &(vehicle_index, destination, _) = top.choose(rng).unwrap();

            game.vehicles[vehicle_index].position = destination;
            game.rebuild_grid();
        }

        self.red_car_is_blocked(game)
    }


    /// Construction-time heuristic for favouring positions with deeper
    /// blocker dependencies around the red-car exit.
    ///
    /// This is NOT the authoritative difficulty measure. The exact BFS
    /// remains responsible for determining the true minimum solution length.
    fn dependency_score(&self, game: &Game) -> usize {
        let red_index = match game.vehicles.iter().position(|v| {
            v.ansi_color == AnsiColorCode::Red
                && v.orientation == Orientation::Horizontal
        }) {
            Some(index) => index,
            None => return 0,
        };

        let state: Vec<(u8, u8)> =
            game.vehicles.iter().map(|v| v.position).collect();

        let red = &game.vehicles[red_index];
        let center_row = GRID_HEIGHT / 2;
        let exit_start = red.position.0 as usize + red.size.0 as usize;

        let mut score = 0usize;

        // Compute every vehicle's mobility in one occupancy pass.
        let mobility_counts =
            Game::legal_move_counts_for_state(&game.vehicles, &state);

        // Direct blockers of the red car's exit are highly valuable.
        for x in exit_start..GRID_WIDTH {
            if !game.is_position_empty(x, center_row) {
                score += 100;
            }
        }

        // Direct blockers that themselves have little mobility are useful
        // because they are more likely to form blocker-of-blocker chains.
        for index in 0..game.vehicles.len() {
            if index == red_index {
                continue;
            }

            let vehicle = &game.vehicles[index];
            let (vx, vy) = state[index];

            let occupies_exit = match vehicle.orientation {
                Orientation::Horizontal => {
                    vy as usize == center_row
                        && (vx as usize
                            ..vx as usize + vehicle.size.0 as usize)
                            .any(|x| x >= exit_start && x < GRID_WIDTH)
                }

                Orientation::Vertical => {
                    (vx as usize) >= exit_start
                        && (vx as usize) < GRID_WIDTH
                        && (vy as usize) <= center_row
                        && (vy as usize + vehicle.size.1 as usize) > center_row
                }
            };

            if !occupies_exit {
                continue;
            }

            score += 75;

            let mobility = mobility_counts[index];

            score += match mobility {
                0 => 100,
                1 => 60,
                2 => 30,
                _ => 0,
            };
        }

        // More constrained vehicles throughout the board provide additional
        // opportunities for dependencies between blockers.
        for index in 0..game.vehicles.len() {
            if index == red_index {
                continue;
            }

            let mobility = mobility_counts[index];

            score += match mobility {
                0 => 2,
                1 => 1,
                _ => 0,
            };
        }

        score
    }

    fn scramble_move_score(
        &self,
        game: &mut Game,
        red_index: usize,
        step: usize,
        target_moves: usize,
    ) -> usize {
        let mut score = self.dependency_score(game);

        // In the final third of the scramble, strongly reward actually blocking
        // the exit. Earlier moves are allowed to develop the dependency structure.
        if step * 3 >= target_moves * 2 && self.red_car_is_blocked(game) {
            score += 500;
        }

        // Reward moves that constrain the vehicle just moved. This creates a more
        // layered dependency graph instead of simply parking vehicles near the exit.
        let state: Vec<(u8, u8)> =
            game.vehicles.iter().map(|v| v.position).collect();
        let mobility = Game::legal_move_destinations_for_state(
            &game.vehicles,
            &state,
            red_index,
        ).len();

        // Fewer red-car destinations means stronger immediate obstruction.
        score += (8usize.saturating_sub(mobility)) * 5;
        score
    }

	fn place_red_car(&mut self, game: &mut Game, vehicle_id: &mut usize){
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
			position: (0, (GRID_HEIGHT / 2) as u8),
			ansi_color: AnsiColorCode::Red,
		};

        println!("Successfully added vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}", vehicle_id, red_car.size, red_car.position, red_car.orientation);

		// Place the red car at the exit
		// Assuming 'game' is a mutable reference to the Game struct and has a method to add vehicles
        game.update_grid_with_new_vehicle(&red_car, *vehicle_id);
	    game.vehicles.push(red_car);

        *vehicle_id += 1;
	}

    pub fn add_vehicle_strategically(
        &mut self,
        game: &mut Game,
        vehicle_id: &mut usize,
    ) -> Vec<Vehicle> {
        // Explicitly bias the generator toward 1x2 cars. This makes the
        // generated layouts roughly 70% cars / 30% trucks when both kinds
        // are available, while still falling back gracefully when one class
        // has no legal placements.
        //
        // Cars:   35% horizontal + 35% vertical = 70%
        // Trucks: 15% horizontal + 15% vertical = 30%
        let car_types = [
            ((2, 1), Orientation::Horizontal),
            ((1, 2), Orientation::Vertical),
        ];
        let truck_types = [
            ((3, 1), Orientation::Horizontal),
            ((1, 3), Orientation::Vertical),
        ];

        let mut rng = rand::thread_rng();

        // First determine which size class we want. If the chosen class has
        // no legal placements, we fall back to the other class.
        let prefer_cars = rng.gen_range(0..100) < 70;
        let preferred_types = if prefer_cars {
            &car_types
        } else {
            &truck_types
        };

        let fallback_types = if prefer_cars {
            &truck_types
        } else {
            &car_types
        };

        let collect_candidates = |
            types: &[((u8, u8), Orientation); 2],
            game: &Game,
            vehicle_id: usize,
        | -> Vec<((usize, usize), Orientation, (u8, u8))> {
            let mut candidates = Vec::new();

            for &(vehicle_size, orientation) in types {
                let possible_positions = match vehicle_size {
                    (2, 1) | (1, 2) => self.generate_possible_positions_1x2(
                        game,
                        vehicle_size,
                        vehicle_id,
                        orientation,
                    ),
                    (3, 1) | (1, 3) => self.generate_possible_positions_1x3(
                        game,
                        vehicle_size,
                        vehicle_id,
                        orientation,
                    ),
                    _ => Vec::new(),
                };

                for (position, position_orientation) in possible_positions {
                    candidates.push((
                        position,
                        position_orientation,
                        vehicle_size,
                    ));
                }
            }

            candidates
        };

        let mut candidates = collect_candidates(
            preferred_types,
            game,
            *vehicle_id,
        );

        // If the preferred size class cannot be placed, use the other class.
        // This prevents the car-heavy bias from making valid boards impossible
        // to complete when the remaining free spaces only fit trucks (or vice versa).
        if candidates.is_empty() {
            candidates = collect_candidates(
                fallback_types,
                game,
                *vehicle_id,
            );
        }

        if candidates.is_empty() {
            println!(
                "No legal vehicle placement found for vehicle ID {}",
                vehicle_id
            );

            return game.vehicles.clone();
        }

        let &(position, orientation, size) =
            candidates.choose(&mut rng).unwrap();

        let vehicle = self.generate_vehicle(
            position,
            orientation,
            size,
            vehicle_id,
        );

        println!(
            "Generated vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}",
            vehicle.id,
            vehicle.size,
            vehicle.position,
            vehicle.orientation
        );

        game.update_grid_with_new_vehicle(&vehicle, *vehicle_id);
        game.vehicles.push(vehicle);

        *vehicle_id += 1;

        game.vehicles.clone()
    }
	
    fn generate_vehicle(
        &mut self,
        position: (usize, usize),
        orientation: Orientation,
        size: (u8, u8),
        vehicle_id: &mut usize,
    ) -> Vehicle {
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
        ];

        if self.used_colors.len() == color_options.len() {
            self.used_colors.clear();
        }

        let available_colors: Vec<_> = color_options
            .into_iter()
            .filter(|color_code| {
                !self.used_colors.contains(&color_code.to_string())
            })
            .collect();

        let ansi_color =
            available_colors[rng.gen_range(0..available_colors.len())];

        let rgba_color = ansi_color.to_rgba();

        self.used_colors.insert(ansi_color.to_string());

        let vehicle = Vehicle::new(
            *vehicle_id,
            rgba_color,
            size,
            (position.0 as u8, position.1 as u8),
            orientation,
            ansi_color,
        );

        println!(
            "Generated vehicle ID: {}, Size: {:?}, Position: {:?}, Orientation: {:?}",
            vehicle_id,
            size,
            position,
            orientation
        );

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

    fn generate_possible_positions_1x2(
        &self,
        game: &Game,
        _vehicle_size: (u8, u8),
        _vehicle_id: usize,
        orientation: Orientation,
    ) -> Vec<((usize, usize), Orientation)> {
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

    fn generate_possible_positions_1x3(
        &self,
        game: &Game,
        _vehicle_size: (u8, u8),
        _vehicle_id: usize,
        orientation: Orientation,
    ) -> Vec<((usize, usize), Orientation)> {
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

    fn print_solution(&self, solution: &PuzzleSolution, vehicles: &[Vehicle]) {
        println!("Minimum solution ({} moves):", solution.solution_moves());

        for (step, solver_move) in solution.moves.iter().enumerate() {
            if let Some(vehicle) = vehicles.get(solver_move.vehicle_index) {
                println!(
                    "  {}. Vehicle {}: ({}, {}) -> ({}, {})",
                    step + 1,
                    vehicle.id,
                    solver_move.from.0,
                    solver_move.from.1,
                    solver_move.to.0,
                    solver_move.to.1,
                );
            }
        }
    }

    /// Solve the puzzle using the exact same movement rules used by the game.
    /// A single player action can move a vehicle any number of cells along its axis.
    /// BFS therefore returns the true minimum number of player moves.
    pub fn solve_puzzle(&self, game: &Game) -> Option<PuzzleSolution> {
        self.solve_puzzle_with_limit(game, 3_000_000)
    }

    fn solve_puzzle_with_limit(
        &self,
        game: &Game,
        max_search_states: usize,
    ) -> Option<PuzzleSolution> {
        let red_index = game.vehicles.iter().position(|v| {
            v.ansi_color == AnsiColorCode::Red
                && v.orientation == Orientation::Horizontal
        })?;

        let center_row = (GRID_HEIGHT / 2) as u8;
        let red_car = &game.vehicles[red_index];

        if red_car.position.1 != center_row {
            return None;
        }

        type State = Vec<(u8, u8)>;

        let start_state: State =
            game.vehicles.iter().map(|v| v.position).collect();

        // Pre-allocate the main BFS containers when practical. This avoids repeated
        // reallocation as difficult candidates generate tens of thousands of states.
        let reserve = max_search_states.min(1_000_000);
        let mut states: Vec<State> = Vec::with_capacity(reserve);
        states.push(start_state.clone());

        let mut queue: VecDeque<usize> = VecDeque::with_capacity(reserve.min(65_536));
        queue.push_back(0);

        let mut visited: HashSet<u128> = HashSet::with_capacity(reserve);
        let mut parent: Vec<Option<(usize, SolverMove)>> = Vec::with_capacity(reserve);
        parent.push(None);

        visited.insert(Self::encode_state(&start_state));

        let mut total_legal_moves = 0usize;
        let mut dead_end_states = 0usize;
        let mut decision_states = 0usize;

        while let Some(state_index) = queue.pop_front() {
            let state = states[state_index].clone();
            let red_position = state[red_index];

            if red_position.1 == center_row
                && red_position.0 as usize + red_car.size.0 as usize
                    == GRID_WIDTH
            {
                let mut moves = Vec::new();
                let mut current = state_index;

                while let Some((parent_index, solver_move)) = &parent[current] {
                    moves.push(solver_move.clone());
                    current = *parent_index;
                }

                moves.reverse();

                let explored_states = states.len();

                let difficulty_score =
                    moves.len() * 1000
                    + decision_states * 10
                    + dead_end_states * 5
                    + explored_states / 100;

                return Some(PuzzleSolution {
                    moves,
                    explored_states,
                    dead_end_states,
                    decision_states,
                    total_legal_moves,
                    difficulty_score,
                });
            }

            if states.len() >= max_search_states {
                println!(
                    "Solver search limit reached at {} states. Rejecting candidate.",
                    states.len()
                );
                return None;
            }

            // Generate the entire state\'s legal move set with one occupancy
            // construction rather than once per vehicle.
            let legal_moves =
                Game::legal_moves_for_state(&game.vehicles, &state);

            let legal_moves_for_state = legal_moves.len();

            for (vehicle_index, new_position) in legal_moves {
                self.enqueue_solver_state(
                    &mut states,
                    &mut queue,
                    &mut visited,
                    &mut parent,
                    state_index,
                    &state,
                    vehicle_index,
                    new_position,
                );
            }

            total_legal_moves += legal_moves_for_state;

            if legal_moves_for_state == 0 {
                dead_end_states += 1;
            } else if legal_moves_for_state > 1 {
                decision_states += 1;
            }
        }

        println!("Puzzle is unsolvable: no solution state was found.");
        None
    }

    fn encode_state(state: &[(u8, u8)]) -> u128 {
        // 7x7 = 49 cells, so six bits encode one vehicle position.
        // Twelve vehicles use only 72 of the 128 available bits.
        let mut key = 0u128;
        for (index, (x, y)) in state.iter().enumerate() {
            let cell = (*y as u128) * GRID_WIDTH as u128 + (*x as u128);
            key |= cell << (index * 6);
        }
        key
    }

    fn enqueue_solver_state(
        &self,
        states: &mut Vec<Vec<(u8, u8)>>,
        queue: &mut VecDeque<usize>,
        visited: &mut HashSet<u128>,
        parent: &mut Vec<Option<(usize, SolverMove)>>,
        parent_index: usize,
        state: &[(u8, u8)],
        vehicle_index: usize,
        new_position: (u8, u8),
    ) {
        let mut next_state = state.to_vec();
        let old_position = next_state[vehicle_index];
        next_state[vehicle_index] = new_position;

        if !visited.insert(Self::encode_state(&next_state)) {
            return;
        }

        let new_index = states.len();
        states.push(next_state);
        parent.push(Some((
            parent_index,
            SolverMove {
                vehicle_index,
                from: old_position,
                to: new_position,
            },
        )));
        queue.push_back(new_index);
    }

    fn red_car_is_blocked(&self, game: &Game) -> bool {
        let red_car = match game.vehicles.iter().find(|vehicle| {
            vehicle.ansi_color == AnsiColorCode::Red
                && vehicle.orientation == Orientation::Horizontal
        }) {
            Some(car) => car,
            None => return false,
        };

        let center_row = (GRID_HEIGHT / 2) as u8;

        if red_car.position.1 != center_row {
            return false;
        }

        let first_exit_cell =
            red_car.position.0 as usize + red_car.size.0 as usize;

        (first_exit_cell..GRID_WIDTH)
            .any(|x| !game.is_position_empty(x, center_row as usize))
    }

}
