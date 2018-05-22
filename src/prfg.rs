pub struct PseudorandomFloatGenerator {
    seed: i64,
}

/// Generates floats (f64) between 0.0 (inclusive) and 1.0 (exclusive).
impl PseudorandomFloatGenerator {
    pub fn new(seed: i64) -> PseudorandomFloatGenerator {
        PseudorandomFloatGenerator {
            seed: PseudorandomFloatGenerator::sanitize_seed(seed),
        }
    }

    fn sanitize_seed(seed: i64) -> i64 {
        const MAX32: i64 = 2147483647;

        let seed = seed % MAX32;

        if seed < 0 {
            seed + MAX32
        } else {
            seed
        }
    }

    pub fn next(&mut self) -> f64 {
        const MAX32: i64 = 2147483647;
        const MAGIC_NUMBER: i64 = 16807;

        self.seed = self.seed * MAGIC_NUMBER % MAX32;

        (self.seed as f64 - 1.0) / (MAX32 as f64 - 1.0)
    }
}
