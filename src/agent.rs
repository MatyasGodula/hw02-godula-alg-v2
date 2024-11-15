pub mod Agent {
    use std::{i32, io};
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    #[derive(EnumIter, Debug)]
    enum Directions {
        North,
        South,
        East,
        West,
        NorthEast,
        SouthEast,
        NorthWest,
        SouthWest,
    }

    enum DataEnumeration {
        Height = 0,
        Width = 1,
    }

    pub struct Agent {
        probes: Vec<i32>,
        probes_max_vis: Vec<i32>,
        board: Vec<Vec<i32>>,
        probe_visions: Vec<Vec<u64>>, // stores the visibilities for each position for every probe 
        board_width: i32,
        board_height: i32,
    }

    struct State {
        occupied_positions: u64,
        visited_positions: u64,
        remaining_probes: u8,
        starting_position: usize
    }

    impl Agent {
        pub fn new() -> Self {
            let mut new_read = Self::read_input();
            new_read.analyze_probes();
            new_read
        }

        fn read_input() -> Self {
            use DataEnumeration::{Height, Width};

            let mut dimensions = String::new();
            io::stdin()
                .read_line(&mut dimensions)
                .expect("Faulty dimension input read");
            let parsed_dimensions: Vec<i32> = dimensions
                .split_whitespace()
                .map(|s| s.parse::<i32>().expect("Faulty dimension input parse"))
                .collect();

            let mut matrix: Vec<Vec<i32>> = Vec::new();
            for _ in 0..parsed_dimensions[Height as usize] {
                let mut line = String::new();
                io::stdin()
                    .read_line(&mut line)
                    .expect("Faulty matrix read");
                let parsed_line: Vec<i32> = line
                    .split_whitespace()
                    .map(|s| s.parse::<i32>().expect("Faulty matrix line parse"))
                    .collect();
                matrix.push(parsed_line);
            }

            let mut number_of_probes = String::new();
            io::stdin()
                .read_line(&mut number_of_probes)
                .expect("Faulty number of probes input");

            let mut probes_string = String::new();
            io::stdin()
                .read_line(&mut probes_string)
                .expect("Faulty probes list read");
            let probes = probes_string
                .split_whitespace()
                .map(|s| s.parse::<i32>().expect("Faulty probe read"))
                .collect();

            Agent {
                probes,
                probes_max_vis: Vec::new(),
                probe_visions: Vec::new(),
                board: matrix,
                board_width: parsed_dimensions[Width as usize],
                board_height: parsed_dimensions[Height as usize],
                
            }
        }

        // sorts the probes in descending order and 
        fn analyze_probes(&mut self) {
            self.probes.sort_by_key(|&x| -x);
            for probe_visibility_index in 0..self.probes.len() {
                let probe_visibility = &self.probes[probe_visibility_index];
                let mut probe_visions_new: Vec<u64> = Vec::new();
                let mut best_visibility = 0;
                for y_coord in 0..self.board_height {
                    for x_coord in 0..self.board_width {
                        let mut current_visited = 0;
                        let new_best_visibility = self.calculate_vision_score(&x_coord, &y_coord, &mut current_visited, *probe_visibility);
                        let new_best = current_visited.count_ones() as i32;
                        probe_visions_new.push(current_visited);
                        if new_best > best_visibility {
                            best_visibility = new_best;
                        }
                    }
                }
                self.probe_visions.push(probe_visions_new);
                self.probes_max_vis.push(best_visibility);
            }
        }

        /*
        Using bitwise operations here ensures fast lookups and updates while minimizing memory usage,
        which is important for performance-critical tasks.
        */

        pub fn can_prune(&self, remaining_probes: u8, best_visited_peaks: i32, current_visited_peaks: i32) -> bool {
            let mut potential_visited_peaks = current_visited_peaks;
            for index_probe in 0..self.probes.len() {
                if remaining_probes & (1 << index_probe) != 0 {
                    potential_visited_peaks += self.probes_max_vis[index_probe];
                }
            }
            //println!("{} < {}", potential_visited_peaks, best_visited_peaks);
            potential_visited_peaks < best_visited_peaks
        }

        pub fn coord_to_index(&self, x: i32, y: i32) -> usize {
            (y * self.board_width + x) as usize // Cast to usize for bitmap indexing
        }

        /// returns a (x, y) tuple
        pub fn index_to_coord(&self, index: usize) -> (i32, i32) {
            let x: usize = index % self.board_width as usize;
            let y: usize = index / self.board_width as usize;
            (x as i32, y as i32)
        }

        pub fn is_occupied(&self, x: i32, y: i32, occupied: &u64) -> bool {
            let index = self.coord_to_index(x, y);
            (occupied & (1 << index)) != 0 // Returns true if the bit at index is set
        }

        pub fn mark_occupied(&self, x: i32, y: i32, occupied: &mut u64) {
            let index = self.coord_to_index(x, y);
            *occupied |= 1 << index; // Marks the bit at an index
        }

        pub fn coord_is_viable(&self, x: i32, y: i32) -> bool {
            x >= 0 && y >= 0 && x < self.board_width && y < self.board_height
        }

        pub fn read_number_of_occupied(&self, occupied: &u64) -> i32 {
            occupied.count_ones() as i32 // Can cast safely because the bitmap is just going to be a u64 meaning 64 bits
        }

        pub fn translate_probes_to_u8(&self) -> u8 {
            let mut probe_storage: u8 = 0;
            for index in 0..self.probes.len() {
                probe_storage |= 1 << index;
            }
            probe_storage
            /*
            example of use:
            self.probes = [1, 1, 2, 3]
            iterates over the probes and outputs probe storage = 15 = 0 0 0 0 1 1 1 1
            */
        }

        pub fn remove_probe(&self, bitmap: &mut u8, index: usize) {
            *bitmap &= !(1 << index);
        }

        pub fn print_data(&self) {
            println!("width: {}, height: {}", self.board_width, self.board_height);
            for line in &self.board {
                println!("{:?}", line);
            }
            println!("{:?}", self.probes);
        }

        pub fn get_visible_peaks(&self, probe_index: usize, x: i32, y :i32) -> u64 {
            self.probe_visions[probe_index][self.coord_to_index(x, y)]
        }

        pub fn get_visited_altitudes(&self, visited: u64) -> i32 {
            let mut sum: i32 = 0;
            for index in 0..64 { // i know i am dealing with a 64 bit number
                if (visited & (1 << index)) != 0 {
                    let (x, y) = self.index_to_coord(index);
                    sum += self.board[y as usize][x as usize];
                }
            }
            sum
        }

        /// x and y are coordinates
        /// visited is a bitmap of already visited states, if used at the start set to 0
        /// visibility is the vision reach of each probe
        /// returns the vision score for a specific probe at a specific position
        pub fn calculate_vision_score(&self, x: &i32, y: &i32, visited: &mut u64, visibility: i32) -> i32 {
            let probe_altitude = self.board[*y as usize][*x as usize];
            let mut sum = 0;
            // add check for occupied probe location
            if !self.is_occupied(*x, *y, visited) { // if the current position is not yet visited
                sum += self.board[*y as usize][*x as usize];
                self.mark_occupied(*x, *y, visited);
            }
            for direction in Directions::iter() {
                let mut new_x = x.clone();
                let mut new_y = y.clone();
                let mut max_slope = f64::NEG_INFINITY; // set the max slope encountered
                loop {
                    match direction {
                        Directions::North => { new_y -= 1; }
                        Directions::South => { new_y += 1; }
                        Directions::East => { new_x += 1; }
                        Directions::West => { new_x -= 1; }
                        Directions::NorthEast => { new_x += 1; new_y -= 1; }
                        Directions::SouthEast => { new_x += 1; new_y += 1; }
                        Directions::NorthWest => { new_x -= 1; new_y -= 1; }
                        Directions::SouthWest => { new_x -= 1; new_y += 1; }
                    }
                    if !self.coord_is_viable(new_x, new_y) {
                        break;
                    }
                    
                    let distance = euclidean_distance(*x, *y, new_x, new_y);
                    let value_at_pos = self.board[new_y as usize][new_x as usize];
                    if distance > visibility as f64 {
                        break;
                    }
                    let slope = (value_at_pos - probe_altitude) as f64 / distance;
                    if slope < max_slope {
                        continue; // skip to the end of the current loop
                    } 
                    max_slope = slope;
                    if !self.is_occupied(new_x, new_y, visited) {
                        sum += value_at_pos;
                        self.mark_occupied(new_x, new_y, visited);
                    }  
                }
            }
            sum
        }

        pub fn dfs(&self) {
            let mut visited_peaks_best: i32 = 0; // stores the number of visited peaks in the best placement
            let mut visited_altitudes_best = 0; // stores the visited altitudes for the best placement
            let mut placed_probes_altitudes_best = i32::MAX; // stores the lowest placed probes

            let initial_state = State {
                occupied_positions: 0,
                visited_positions: 0,
                remaining_probes: self.translate_probes_to_u8(),
                starting_position: 0
            };

            let mut stack = Vec::new();

            stack.push(initial_state);

            while let Some(popped_state) = stack.pop() {
                if popped_state.remaining_probes.count_ones() == 0 { // reached the end of the dfs
                    let popped_visited_peaks = popped_state.visited_positions.count_ones() as i32;
                    let popped_visited_altitudes = self.get_visited_altitudes(popped_state.visited_positions);
                    let popped_placed_probes_altitudes = self.get_visited_altitudes(popped_state.occupied_positions);
                    if popped_visited_peaks > visited_peaks_best {
                        visited_peaks_best = popped_visited_peaks as i32;
                        visited_altitudes_best = popped_visited_altitudes;
                        placed_probes_altitudes_best = popped_placed_probes_altitudes as i32;
                    } else if popped_visited_peaks == visited_peaks_best as i32 && popped_visited_altitudes > visited_altitudes_best {
                        visited_peaks_best = popped_visited_peaks;
                        visited_altitudes_best = popped_visited_altitudes;
                        placed_probes_altitudes_best = popped_placed_probes_altitudes as i32;
                    } else if popped_visited_peaks == visited_peaks_best && popped_visited_altitudes == visited_altitudes_best && placed_probes_altitudes_best > popped_placed_probes_altitudes {
                        visited_peaks_best = popped_visited_peaks as i32;
                        visited_altitudes_best = popped_visited_altitudes;
                        placed_probes_altitudes_best = popped_placed_probes_altitudes as i32;
                    }
                    continue;
                } 
                for probe_index in popped_state.starting_position..self.probes.len() {
                    
                    if popped_state.remaining_probes & (1 << probe_index) == 0 {
                        continue; // go to the next probe if this probe has been used
                    }
                    for y_coord in 0..self.board_height { // go through all the coordinates
                        for x_coord in 0..self.board_width {
                            if self.is_occupied(x_coord, y_coord, &popped_state.occupied_positions) {
                                continue; // if the current position is already occupied go to the next
                            }
                            //let current_probe_visibility = self.probes[probe_index];
                            let mut new_occupied = popped_state.occupied_positions;
                            let mut new_visited = popped_state.visited_positions;
                            let mut new_remaining_probes = popped_state.remaining_probes;
                            self.mark_occupied(x_coord, y_coord, &mut new_occupied); // mark the current position as occupied
                            new_visited |= self.get_visible_peaks(probe_index, x_coord, y_coord); // update the new visited position
                            self.remove_probe(&mut new_remaining_probes, probe_index); // remove the current remaining probe
                            if self.can_prune(new_remaining_probes, visited_peaks_best, new_visited.count_ones() as i32) {
                                continue;
                            }

                            stack.push(State {
                                occupied_positions: new_occupied,
                                visited_positions: new_visited,
                                remaining_probes: new_remaining_probes,
                                starting_position: popped_state.starting_position + 1
                            }); // push the next state to the stack
                        }
                    }
                }
            }
            println!("{} {} {}", visited_peaks_best, visited_altitudes_best, placed_probes_altitudes_best);

        }

    }

    pub fn euclidean_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f64 {
        ((x2 as f64 - x1 as f64).powi(2) + (y2 as f64 - y1 as f64).powi(2)).sqrt()
    }

}
