//! # Fixed-Point Mathematics Module
//!
//! This module provides deterministic mathematical functions for on-chain computation.
//! All calculations use integer arithmetic to ensure consensus across nodes.
//!
//! ## Key Functions
//!
//! - `exp_approximation`: Computes e^x using Taylor series
//! - `ln_approximation`: Computes ln(x) using Newton-Raphson method
//! - `integer_sqrt`: Computes √x for diminishing returns in Q_net
//!
//! ## Precision
//!
//! We use 10^12 (PRECISION) as our fixed-point scaling factor.
//! This gives us 12 decimal places of precision which is sufficient for
//! financial calculations while avoiding overflow in u128.

/// Precision factor for fixed-point arithmetic (10^12)
pub const PRECISION: u128 = 1_000_000_000_000;

/// Maximum iterations for Taylor series
const MAX_ITERATIONS: u32 = 20;

/// Approximate e^x using Taylor series expansion.
///
/// e^x = 1 + x + x²/2! + x³/3! + x⁴/4! + ...
///
/// # Arguments
/// * `x` - The exponent (already scaled by PRECISION)
///
/// # Returns
/// * e^x scaled by PRECISION
///
/// # Note
/// For very large x (>20), this will overflow or be inaccurate.
/// For negative exponents, compute e^(-x) = 1/e^x instead.
pub fn exp_approximation(x: u128) -> u128 {
    if x == 0 {
        return PRECISION;
    }

    // For very large x, return a large number (will be capped)
    // e^20 ≈ 485 million, which is manageable
    // e^50 would overflow u128
    if x > PRECISION * 50 {
        return u128::MAX / 2; // Effectively infinity for our purposes
    }

    // Taylor series: e^x = Σ(x^n / n!) for n = 0 to ∞
    let mut result: u128 = PRECISION; // Term for n=0
    let mut term: u128 = PRECISION; // Current term

    for n in 1..=MAX_ITERATIONS {
        // term(n) = term(n-1) * x / n
        // Using checked ops to prevent overflow
        let next_term = term
            .checked_mul(x)
            .and_then(|v| v.checked_div(PRECISION))
            .and_then(|v| v.checked_div(n as u128));

        match next_term {
            Some(t) if t > 0 => {
                term = t;
                result = result.saturating_add(term);
            }
            _ => break, // Term too small or overflow, stop
        }
    }

    result
}

/// Approximate natural logarithm ln(x) using iterative method.
///
/// Uses the identity: ln(x) = 2 * arctanh((x-1)/(x+1))
/// and Taylor series for arctanh.
///
/// # Arguments
/// * `x` - The value (scaled by PRECISION, must be > 0)
///
/// # Returns
/// * ln(x) scaled by PRECISION
pub fn ln_approximation(x: u128) -> u128 {
    if x == 0 {
        return 0; // ln(0) is undefined, return 0 as fallback
    }

    if x == PRECISION {
        return 0; // ln(1) = 0
    }

    // For x close to 1, use Taylor series directly
    // ln(1+y) ≈ y - y²/2 + y³/3 - y⁴/4 + ...
    // where y = (x - PRECISION) / PRECISION

    if x > PRECISION / 2 && x < PRECISION * 3 {
        // x is between 0.5 and 3, use direct series
        let y = if x > PRECISION {
            x.saturating_sub(PRECISION)
        } else {
            PRECISION.saturating_sub(x)
        };
        let y_scaled = y.saturating_mul(PRECISION) / x;

        let mut result: u128 = 0;
        let mut term = y_scaled;
        let mut sign_positive = x > PRECISION;

        for n in 1..=MAX_ITERATIONS {
            let contribution = term / n as u128;
            if contribution == 0 {
                break;
            }

            if sign_positive {
                result = result.saturating_add(contribution);
            } else {
                result = result.saturating_sub(contribution);
            }

            // Next term: multiply by y and alternate sign
            term = term.saturating_mul(y_scaled) / PRECISION;
            sign_positive = !sign_positive;
        }

        return result;
    }

    // For x > 3 or x < 0.5, use reduction
    // ln(a*b) = ln(a) + ln(b)
    // Reduce x to range [1, 2] and count powers of 2

    let mut value = x;
    let mut log2_count: i32 = 0;

    // Normalize to [PRECISION, 2*PRECISION]
    while value > PRECISION * 2 {
        value /= 2;
        log2_count += 1;
    }
    while value > 0 && value < PRECISION {
        value *= 2;
        log2_count -= 1;
    }

    // ln(x) = ln(normalized) + log2_count * ln(2)
    let ln_normalized = ln_approximation(value);

    // ln(2) ≈ 0.693147... × PRECISION
    const LN_2: u128 = 693_147_180_559;

    if log2_count >= 0 {
        ln_normalized.saturating_add(LN_2.saturating_mul(log2_count as u128))
    } else {
        ln_normalized.saturating_sub(LN_2.saturating_mul((-log2_count) as u128))
    }
}

/// Compute integer square root using binary search.
///
/// Returns floor(√x).
///
/// # Arguments
/// * `x` - The value to take the square root of
///
/// # Returns
/// * floor(√x)
pub fn integer_sqrt(x: u128) -> u128 {
    if x == 0 {
        return 0;
    }
    if x == 1 {
        return 1;
    }

    // Binary search for sqrt
    let mut low: u128 = 1;
    let mut high: u128 = x;
    let mut result: u128 = 0;

    while low <= high {
        let mid = low + (high - low) / 2;

        // Check if mid*mid <= x
        if let Some(square) = mid.checked_mul(mid) {
            if square == x {
                return mid;
            } else if square < x {
                result = mid;
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        } else {
            // Overflow, mid is too large
            high = mid - 1;
        }
    }

    result
}

/// Compute x^y using repeated squaring.
///
/// # Arguments
/// * `base` - The base (scaled by PRECISION)
/// * `exp` - The exponent (not scaled, integer)
///
/// # Returns
/// * base^exp scaled by PRECISION
pub fn pow(base: u128, exp: u32) -> u128 {
    if exp == 0 {
        return PRECISION;
    }
    if exp == 1 {
        return base;
    }

    let mut result = PRECISION;
    let mut b = base;
    let mut e = exp;

    while e > 0 {
        if e & 1 == 1 {
            result = result.saturating_mul(b) / PRECISION;
        }
        e >>= 1;
        if e > 0 {
            b = b.saturating_mul(b) / PRECISION;
        }
    }

    result
}

/// Compute the sigmoid function S(x) = 1 / (1 + e^(-x)).
///
/// # Arguments
/// * `x` - The input value (scaled by PRECISION)
/// * `is_negative` - Whether x should be treated as negative
///
/// # Returns
/// * S(x) scaled by PRECISION (value between 0 and PRECISION)
pub fn sigmoid(x: u128, is_negative: bool) -> u128 {
    // S(x) = 1 / (1 + e^(-x))
    // For positive x: S(x) = 1 / (1 + e^(-x))
    // For negative x: S(-|x|) = 1 / (1 + e^|x|) = e^(-|x|) / (1 + e^(-|x|))

    let e_power = exp_approximation(x);

    if is_negative {
        // x is negative: 1 / (1 + e^|x|)
        let denominator = PRECISION.saturating_add(e_power);
        if denominator == 0 {
            return 0;
        }
        PRECISION.saturating_mul(PRECISION) / denominator
    } else {
        // x is positive: 1 / (1 + e^(-|x|)) = e^|x| / (e^|x| + 1)
        if e_power > u128::MAX / 2 {
            return PRECISION; // For large x, sigmoid ≈ 1
        }
        let denominator = e_power.saturating_add(PRECISION);
        if denominator == 0 {
            return PRECISION;
        }
        e_power.saturating_mul(PRECISION) / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exp_zero() {
        // e^0 = 1
        assert_eq!(exp_approximation(0), PRECISION);
    }

    #[test]
    fn test_exp_one() {
        // e^1 ≈ 2.718...
        let result = exp_approximation(PRECISION);
        // Should be approximately 2.718 * PRECISION
        let expected = 2_718_281_828_459u128;
        let diff = if result > expected {
            result - expected
        } else {
            expected - result
        };
        // Allow 1% error
        assert!(diff < expected / 100);
    }

    #[test]
    fn test_exp_two() {
        // e^2 ≈ 7.389...
        let result = exp_approximation(PRECISION * 2);
        let expected = 7_389_056_098_930u128;
        let diff = if result > expected {
            result - expected
        } else {
            expected - result
        };
        assert!(diff < expected / 50); // 2% tolerance
    }

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(9), 3);
        assert_eq!(integer_sqrt(16), 4);
        assert_eq!(integer_sqrt(100), 10);
        assert_eq!(integer_sqrt(10000), 100);
        assert_eq!(integer_sqrt(99), 9); // floor(√99) = 9
    }

    #[test]
    fn test_pow() {
        // 2^3 = 8
        let result = pow(PRECISION * 2, 3);
        assert_eq!(result, PRECISION * 8);

        // 2^0 = 1
        assert_eq!(pow(PRECISION * 2, 0), PRECISION);

        // 2^1 = 2
        assert_eq!(pow(PRECISION * 2, 1), PRECISION * 2);
    }

    #[test]
    fn test_sigmoid_zero() {
        // S(0) = 0.5
        let result = sigmoid(0, false);
        // Should be PRECISION / 2
        let expected = PRECISION / 2;
        let diff = if result > expected {
            result - expected
        } else {
            expected - result
        };
        assert!(diff < PRECISION / 100); // 1% tolerance
    }

    #[test]
    fn test_sigmoid_large_positive() {
        // S(large) ≈ 1
        let result = sigmoid(PRECISION * 10, false);
        assert!(result > PRECISION * 99 / 100); // > 99%
    }

    #[test]
    fn test_sigmoid_large_negative() {
        // S(-large) ≈ 0
        let result = sigmoid(PRECISION * 10, true);
        assert!(result < PRECISION / 100); // < 1%
    }
}
