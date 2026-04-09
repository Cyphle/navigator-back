use std::ops::{Add, Div, Mul, Sub};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundingMode {
    HalfUp,
    HalfDown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BigDecimal {
    value: f64,
    scale: u8,
    mode: RoundingMode,
}

impl BigDecimal {
    pub fn new(value: f64, scale: u8, mode: RoundingMode) -> Self {
        Self {
            value: Self::round_to_scale(value, scale, mode),
            scale,
            mode,
        }
    }

    pub fn from(value: f64) -> Self {
        Self::new(value, 2, RoundingMode::HalfUp)
    }

    pub fn to_f64(&self) -> f64 {
        self.value
    }

    pub fn scale(&self) -> u8 {
        self.scale
    }

    pub fn set_scale(&self, scale: u8) -> Self {
        Self::new(self.value, scale, self.mode)
    }

    fn round_to_scale(value: f64, scale: u8, mode: RoundingMode) -> f64 {
        let factor = 10f64.powi(scale as i32);
        let multiplied = value * factor;

        let rounded = match mode {
            RoundingMode::HalfUp => {
                // .5 rounds away from zero (standard arithmetic rounding)
                multiplied.round()
            }
            RoundingMode::HalfDown => {
                // .5 rounds towards zero.
                // Use a relative tolerance to absorb f64 representation noise:
                // e.g. 1.235 * 100 == 123.50000000000001 in f64, which is only
                // ~1.4e-14 above 0.5 — well within the tolerance below.
                let abs_m = multiplied.abs();
                let floor_m = abs_m.floor();
                let frac = abs_m - floor_m;
                let tolerance = f64::EPSILON * abs_m.max(1.0) * 4.0;

                let res = if frac <= 0.5 + tolerance { floor_m } else { abs_m.ceil() };

                if multiplied < 0.0 { -res } else { res }
            }
        };

        rounded / factor
    }

    pub fn add(&self, other: BigDecimal) -> BigDecimal {
        let scale = self.scale.max(other.scale);
        BigDecimal::new(self.value + other.value, scale, self.mode)
    }

    pub fn minus(&self, other: BigDecimal) -> BigDecimal {
        let scale = self.scale.max(other.scale);
        BigDecimal::new(self.value - other.value, scale, self.mode)
    }

    pub fn multiply(&self, other: BigDecimal) -> BigDecimal {
        // scale1 + scale2 mirrors Java BigDecimal semantics
        let scale = self.scale + other.scale;
        BigDecimal::new(self.value * other.value, scale, self.mode)
    }

    pub fn multiply_f64(&self, other: f64) -> BigDecimal {
        BigDecimal::new(self.value * other, self.scale, self.mode)
    }

    pub fn divide(&self, other: BigDecimal) -> BigDecimal {
        BigDecimal::new(self.value / other.value, self.scale, self.mode)
    }

    pub fn divide_f64(&self, other: f64) -> BigDecimal {
        BigDecimal::new(self.value / other, self.scale, self.mode)
    }
}

impl Add for BigDecimal {
    type Output = BigDecimal;
    fn add(self, rhs: Self) -> Self::Output {
        BigDecimal::add(&self, rhs)
    }
}

impl Sub for BigDecimal {
    type Output = BigDecimal;
    fn sub(self, rhs: Self) -> Self::Output {
        BigDecimal::minus(&self, rhs)
    }
}

impl Mul for BigDecimal {
    type Output = BigDecimal;
    fn mul(self, rhs: Self) -> Self::Output {
        BigDecimal::multiply(&self, rhs)
    }
}

impl Mul<f64> for BigDecimal {
    type Output = BigDecimal;
    fn mul(self, rhs: f64) -> Self::Output {
        BigDecimal::multiply_f64(&self, rhs)
    }
}

impl Div for BigDecimal {
    type Output = BigDecimal;
    fn div(self, rhs: Self) -> Self::Output {
        BigDecimal::divide(&self, rhs)
    }
}

impl Div<f64> for BigDecimal {
    type Output = BigDecimal;
    fn div(self, rhs: f64) -> Self::Output {
        BigDecimal::divide_f64(&self, rhs)
    }
}

pub fn to_big_decimal(d: Decimal) -> BigDecimal {
    BigDecimal::new(d.to_f64().unwrap_or(0.0), 2, RoundingMode::HalfUp)
}

pub fn bd(value: f64) -> BigDecimal {
    BigDecimal::new(value, 2, RoundingMode::HalfUp)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Construction & rounding ---

    #[test]
    fn test_half_up_rounds_away_from_zero_at_half() {
        assert_eq!(BigDecimal::new(1.235, 2, RoundingMode::HalfUp).to_f64(), 1.24);
        assert_eq!(BigDecimal::new(1.234, 2, RoundingMode::HalfUp).to_f64(), 1.23);
        assert_eq!(BigDecimal::new(-1.235, 2, RoundingMode::HalfUp).to_f64(), -1.24);
    }

    #[test]
    fn test_half_down_rounds_towards_zero_at_half() {
        assert_eq!(BigDecimal::new(1.235, 2, RoundingMode::HalfDown).to_f64(), 1.23);
        assert_eq!(BigDecimal::new(1.236, 2, RoundingMode::HalfDown).to_f64(), 1.24);
        assert_eq!(BigDecimal::new(-1.235, 2, RoundingMode::HalfDown).to_f64(), -1.23);
    }

    #[test]
    fn test_scale_is_preserved() {
        let bd = BigDecimal::new(1.23456, 2, RoundingMode::HalfUp);
        assert_eq!(bd.to_f64(), 1.23);
        assert_eq!(bd.scale(), 2);
    }

    #[test]
    fn test_set_scale() {
        let bd = BigDecimal::new(1.23456, 4, RoundingMode::HalfUp);
        let rescaled = bd.set_scale(2);
        assert_eq!(rescaled.to_f64(), 1.23);
        assert_eq!(rescaled.scale(), 2);
    }

    // --- add ---

    #[test]
    fn test_add() {
        assert_eq!((bd(1.23) + bd(1.27)).to_f64(), 2.50);
    }

    #[test]
    fn test_add_avoids_f64_precision_error() {
        // 0.1 + 0.2 in raw f64 != 0.3
        let a = BigDecimal::new(0.1, 1, RoundingMode::HalfUp);
        let b = BigDecimal::new(0.2, 1, RoundingMode::HalfUp);
        assert_eq!((a + b).to_f64(), 0.3);
    }

    // --- minus ---

    #[test]
    fn test_minus() {
        assert_eq!((bd(2.50) - bd(1.27)).to_f64(), 1.23);
    }

    // --- multiply ---

    #[test]
    fn test_multiply_bigdecimal_doubles_scale() {
        let a = BigDecimal::new(1.2, 1, RoundingMode::HalfUp);
        let b = BigDecimal::new(1.2, 1, RoundingMode::HalfUp);
        let res = a * b;
        assert_eq!(res.to_f64(), 1.44);
        assert_eq!(res.scale(), 2);
    }

    #[test]
    fn test_multiply_f64() {
        assert_eq!((bd(1.23) * 2.0).to_f64(), 2.46);
    }

    #[test]
    fn test_multiply_then_set_scale() {
        // multiply two scale-2 values → scale 4; re-scale back to 2
        let res = (bd(10.00) * bd(1.15)).set_scale(2);
        assert_eq!(res.to_f64(), 11.50);
        assert_eq!(res.scale(), 2);
    }

    // --- divide ---

    #[test]
    fn test_divide_bigdecimal() {
        // 1.0 / 3.0 = 0.333... rounded to 2 decimals → 0.33
        assert_eq!((bd(1.0) / bd(3.0)).to_f64(), 0.33);
    }

    #[test]
    fn test_divide_f64() {
        assert_eq!((bd(9.99) / 3.0).to_f64(), 3.33);
    }

    #[test]
    fn test_divide_by_two() {
        assert_eq!((bd(1.00) / 2.0).to_f64(), 0.50);
    }
}
