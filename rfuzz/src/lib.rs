/// A basic random number generator based on xorshift64 with 64-bits of state
struct Rng {
    /// The RNG's seed and state
    seed: u64,
}

impl Rng {
    /// Generate a random number
    #[inline]
    fn next(&mut self) -> u64 {
        let val = self.seed;
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 43;
        val
    }

    /// Generates a random number with uniform distribution in the range of
    /// [min, max]
    #[inline]
    fn rand(&mut self, min: usize, max: usize) -> usize {
        // Make sure the range is sane
        assert!(max >= min, "Bad range specified for rand()");

        // If there is no range, just return `min`
        if min == max {
            return min;
        }
        // If the range is unbounded, just return a random number
        if min == 0 && max == core::usize::MAX {
            return self.next() as usize;
        }

        // Pick a random number in the range
        min + (self.next() as usize % (max - min + 1))
    }
}

pub struct Mutator {
    pub input: Vec<u8>,
    rng: Rng,
}

impl Mutator {
    pub fn new(input: Vec<u8>, seed: u64) -> Self {
        Mutator {
            input: input,
            rng: Rng {
                seed: seed,
            },
        }
    }

    pub fn bitflip(&mut self, flip_rate: f64) {
        let magic_vals = vec![
            (1, 255),
            (1, 255),
            (1, 127),
            (1, 0),
            (2, 255),
            (2, 0),
            (4, 255),
            (4, 0),
            (4, 128),
            (4, 64),
            (4, 127),
        ];

        let num_of_flip = (self.input.len() as f64 * flip_rate) as usize;
        let mut rand_index = vec![];
        for _ in 0..num_of_flip {
            rand_index.push(self.rng.rand(0, self.input.len() - 1));
        }

        for index in rand_index {
            let (index_len, index_val) = magic_vals[self.rng.rand(0, magic_vals.len() - 1)];
            if index + index_len <= self.input.len() {
                self.input[index] = index_val;
                for i in 1..index_len {
                    self.input[index + i] = 0;
                }
            }
        }
    }
}
