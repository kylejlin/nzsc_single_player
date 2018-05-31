pub struct PseudorandomFloatGenerator {
    state: u32,
}

/// Generates floats (f64) between 0.0 (inclusive) and 1.0 (exclusive).
///
/// Uses [Xorshift](https://en.wikipedia.org/wiki/Xorshift)
impl PseudorandomFloatGenerator {
    pub fn new(seed: u32) -> PseudorandomFloatGenerator {
        PseudorandomFloatGenerator {
            state: seed,
        }
    }

    pub fn next(&mut self) -> f64 {
        // Calculate random u32.
        // https://en.wikipedia.org/wiki/Xorshift
        let mut x = self.state;
        x ^= x << 13;
    	x ^= x >> 17;
    	x ^= x << 5;
        let x = x;

    	self.state = x;

        // Convert to u16 and divide by (2 ** 16) to get random float
        (x >> 16) as f64 / 65536.0
    }
}
