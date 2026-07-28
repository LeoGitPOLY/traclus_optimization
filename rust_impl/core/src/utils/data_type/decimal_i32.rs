#[derive(Debug, Clone, Copy)]
pub struct DecimalI32(i32);

pub const SCALE: u32 = 10; // 1 decimal place of precision

impl DecimalI32 {
    pub fn from_f64(value: f64) -> Self {
        let scaled: i32 = (value * SCALE as f64).round() as i32;
        Self(scaled)
    }

    pub fn to_f64(self) -> f64 {
        self.0 as f64 / SCALE as f64
    }

    pub fn raw(self) -> i32 {
        self.0
    }

    pub fn power_u64(self) -> u64 {
        let val: i64 = self.raw() as i64;
        let squared: i64 = val * val;
        let scale: u32 = SCALE * SCALE;
        (squared as u64) / scale as u64
    }
}

impl std::ops::Sub for DecimalI32 {
    type Output = DecimalI32;

    fn sub(self, other: DecimalI32) -> DecimalI32 {
        DecimalI32(self.raw() - other.raw())
    }
}
