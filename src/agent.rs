pub mod Agent {
    use std::{i32, io};
    use strum::IntoEnumIterator;
    use strum_macros::EnumIter;

    /*
    I agree that this code might seem like a bit of a show off stunt to show that I can solve it much faster than the reference
    but in reality i was really worried that the program would be too slow so I performed an incredibly aggressive set of optimizations
    specifically the use of bitmaps 
    */


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
        probes: Vec<i32>, // stores the read probes
        probes_max_vis: Vec<i32>, // the most covered mountain peaks, used for pruning for each probe
        board: Vec<Vec<i32>>, // stores the board with the read altitudes
        probe_visions: Vec<Vec<u64>>, // stores the visibilities for each position for every probe 
        board_width: i32, // dimensions
        board_height: i32, // dimensions
    }

    struct State {
        occupied_positions: u64, // maps the places where probes are placed
        visited_positions: u64, // bitmap storing positions that are monitored by any one of the placed probes
        remaining_probes: u8, // keeps remaining probes in binary ie. 0 0 0 0 1 1 1 1 means that there are 4 probes on indexes 0, 1, 2, 3
        next_index: usize // used in the dfs to pass the next index of a probe to be explored
    }

    impl Agent {
        pub fn new() -> Self {
            let mut new_read = Self::read_input();
            new_read.analyze_probes();
            new_read
        }

        fn read_input() -> Self { // self explanatory
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
        fn analyze_probes(&mut self) { // precomputes data that will be used for getting important info fast e.g. getting the visibility for a specific position
            self.probes.sort_by_key(|&x| -x); // sorts the array in a descending order creating a heuristic where the probes with the largest visibility will be placed first
            for probe_visibility_index in 0..self.probes.len() {
                let probe_visibility = &self.probes[probe_visibility_index]; // fetches the visibility for the current probe
                let mut probe_visions_new: Vec<u64> = Vec::new();
                let mut best_visibility = 0; // initializes the visibility as 0 
                for y_coord in 0..self.board_height {
                    for x_coord in 0..self.board_width {
                        let mut current_visited = 0; // initializes an empty bit map for storing visited positions
                        let _ = self.calculate_vision_score(&x_coord, &y_coord, &mut current_visited, *probe_visibility); // the return value is mostly a remnant of some previous tries, could be further optimized
                        let new_best = current_visited.count_ones() as i32; // counts the bits flipped to 1 which means visible from this position
                        probe_visions_new.push(current_visited); // this stores the vision maps for every position which i can afford since i am working with just 64 bits for each
                        if new_best > best_visibility {
                            best_visibility = new_best;
                        }
                    }
                }
                self.probe_visions.push(probe_visions_new); // these will be useful later
                self.probes_max_vis.push(best_visibility);
            }
        }

        /*
        Using bitwise operations here ensures fast lookups and updates while minimizing memory usage,
        which is important for performance-critical tasks.
        */

        /// if all remaining nodes visibilities added will be less than the best possible solution discovered so far then prune
        pub fn can_prune(&self, remaining_probes: u8, best_visited_peaks: i32, current_visited_peaks: i32) -> bool {
            let mut potential_visited_peaks = current_visited_peaks;
            for index_probe in 0..self.probes.len() {
                if remaining_probes & (1 << index_probe) != 0 {
                    potential_visited_peaks += self.probes_max_vis[index_probe];
                }
            }
            potential_visited_peaks < best_visited_peaks
        }

        fn coord_to_index(&self, x: i32, y: i32) -> usize {
            (y * self.board_width + x) as usize // Cast to usize for bitmap indexing
        }

        /// returns a (x, y) tuple inverse of the previous function
        fn index_to_coord(&self, index: usize) -> (i32, i32) {
            let x: usize = index % self.board_width as usize;
            let y: usize = index / self.board_width as usize;
            (x as i32, y as i32)
        }

        fn is_occupied(&self, x: i32, y: i32, occupied: &u64) -> bool {
            let index = self.coord_to_index(x, y);
            (occupied & (1 << index)) != 0 // Returns true if the bit at index is set
        }

        fn mark_occupied(&self, x: i32, y: i32, occupied: &mut u64) {
            let index = self.coord_to_index(x, y);
            *occupied |= 1 << index; // Marks the bit at an index
        }

        fn coord_is_viable(&self, x: i32, y: i32) -> bool {
            x >= 0 && y >= 0 && x < self.board_width && y < self.board_height
        }

        fn translate_probes_to_u8(&self) -> u8 {
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

        fn remove_probe(&self, bitmap: &mut u8, index: usize) { // abstracts bit logic for a more readable code
            *bitmap &= !(1 << index);
        }

        fn get_visible_peaks(&self, probe_index: usize, x: i32, y :i32) -> u64 { // returns the bitmap of vision for the current probe at a specific location
            self.probe_visions[probe_index][self.coord_to_index(x, y)]
        }

        fn get_visited_altitudes(&self, visited: u64) -> i32 { // goes through all flipped bits and sums the altitudes at these positions
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
        fn calculate_vision_score(&self, x: &i32, y: &i32, visited: &mut u64, visibility: i32) -> i32 {
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
                    if !self.is_occupied(new_x, new_y, visited) { // flips the bit at the current index to show that it can be seen by the probe
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
                next_index: 0
            };

            let mut stack = Vec::new();

            stack.push(initial_state);

            while let Some(popped_state) = stack.pop() {
                if popped_state.remaining_probes.count_ones() == 0 { // reached the end of the dfs
                    // extracts data from the popped probe for more readability
                    let popped_visited_peaks = popped_state.visited_positions.count_ones() as i32; 
                    let popped_visited_altitudes = self.get_visited_altitudes(popped_state.visited_positions);
                    let popped_placed_probes_altitudes = self.get_visited_altitudes(popped_state.occupied_positions);
                    // checks for the logic for evaluating best positions
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
                let probe_index = popped_state.next_index; // reads the current index from the popped state, it is a remnant of the code that used to be here, i just did not want to refactor it all rust will optimize it away
                if popped_state.remaining_probes & (1 << probe_index) == 0 { // checks if this probe has already been used (probably redundant now)
                    continue; // go to the next probe if this probe has been used
                }
                for y_coord in 0..self.board_height { // go through all the coordinates
                    for x_coord in 0..self.board_width {
                        if self.is_occupied(x_coord, y_coord, &popped_state.occupied_positions) {
                            continue; // if the current position is already occupied go to the next
                        }
                        let mut new_occupied = popped_state.occupied_positions;
                        let mut new_visited = popped_state.visited_positions;
                        let mut new_remaining_probes = popped_state.remaining_probes;
                        self.mark_occupied(x_coord, y_coord, &mut new_occupied); // mark the current position as occupied
                        /*
                        This code below is extremely interesting, its an incredibly fast way of combining the visibility and the already visited nodes
                        for example lets say I had two shorter bitmaps
                        visited: 0 0 1 1 0 1 1 0
                        visible_peaks: 1 1 1 0 0 0 0 0 lets say the probe was placed at index 1 and sees 1 to each side
                        the |= operator will combine these two making
                        visited: 1 1 1 1 0 1 1 0 meaning that the already flipped bits will stay flipped and the 0 ones will flip to 1 
                        */
                        new_visited |= self.get_visible_peaks(probe_index, x_coord, y_coord); // update the new visited position
                        self.remove_probe(&mut new_remaining_probes, probe_index); // remove the current remaining probe
                        if self.can_prune(new_remaining_probes, visited_peaks_best, new_visited.count_ones() as i32) {
                            continue;
                        }

                        stack.push(State {
                            occupied_positions: new_occupied,
                            visited_positions: new_visited,
                            remaining_probes: new_remaining_probes,
                            next_index: popped_state.next_index + 1
                        }); // push the next state to the stack
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
