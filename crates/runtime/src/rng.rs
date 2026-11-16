#[derive(Clone, Debug)]
pub struct Rng {
    state: u32,
    forced: Option<u32>,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed.max(1),
            forced: None,
        }
    }

    pub fn force(&mut self, value: u32) {
        self.forced = Some(value);
    }

    pub fn next_u32(&mut self) -> u32 {
        if let Some(value) = self.forced.take() {
            return value;
        }
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        if x == 0 {
            x = 0xA341_316C;
        }
        self.state = x;
        x
    }

    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.next_u32() % n
        }
    }

    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let span = (hi as i64 - lo as i64) as u32 + 1;
        lo.saturating_add(self.below(span) as i32)
    }

    pub fn chance(&mut self, percent: i32) -> bool {
        (self.below(100) as i32) < percent.clamp(0, 100)
    }

    /// 0..=100, used by the damage variance roll.
    pub fn percent(&mut self) -> i32 {
        self.below(101) as i32
    }
}
