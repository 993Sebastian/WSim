//! Platform-independent math. All transcendental functions go through `libm`, so the
//! results are bit-identical on every system (Lastenheft §16.1).

use std::f64::consts::SQRT_2;

pub fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

pub fn ln(x: f64) -> f64 {
    libm::log(x)
}

pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}

pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

pub fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}

/// `x` to `digits` significant digits (zero and non-finite values unchanged).
pub fn round_significant(x: f64, digits: i32) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    #[allow(clippy::cast_possible_truncation)]
    let magnitude = libm::floor(libm::log10(x.abs())) as i32;
    let shift = digits - 1 - magnitude;
    // Scaling by an exact power of ten keeps round values such as 1e12 exact.
    if shift >= 0 {
        let scale = pow(10.0, f64::from(shift));
        libm::round(x * scale) / scale
    } else {
        let scale = pow(10.0, f64::from(-shift));
        libm::round(x / scale) * scale
    }
}

/// Great-circle distance in km between two points given in degrees (haversine).
pub fn great_circle_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6_371.0;
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dp = (lat2 - lat1).to_radians();
    let dl = (lon2 - lon1).to_radians();
    let h = sin(dp / 2.0) * sin(dp / 2.0) + cos(p1) * cos(p2) * sin(dl / 2.0) * sin(dl / 2.0);
    2.0 * EARTH_RADIUS_KM * asin(sqrt(h.clamp(0.0, 1.0)))
}

/// Position of `x` between `from` and `to` on a logarithmic scale, clamped to 0–1.
pub fn log_position(x: f64, from: f64, to: f64) -> f64 {
    if x <= from {
        return 0.0;
    }
    if x >= to {
        return 1.0;
    }
    (ln(x) - ln(from)) / (ln(to) - ln(from))
}

/// Standard normal distribution function.
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * libm::erfc(-x / SQRT_2)
}

/// Inverse of the standard normal distribution function (Acklam's approximation with
/// one Halley refinement step; accurate to about 1e-15).
pub fn normal_inverse(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const LOW: f64 = 0.024_25;

    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    let tail = |q: f64| {
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    let x = if p < LOW {
        tail(sqrt(-2.0 * ln(p)))
    } else if p <= 1.0 - LOW {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        -tail(sqrt(-2.0 * ln(1.0 - p)))
    };
    let e = normal_cdf(x) - p;
    let u = e * sqrt(2.0 * std::f64::consts::PI) * exp(x * x / 2.0);
    x - u / (1.0 + x * u / 2.0)
}

/// Income share of each fifth of households (poorest first) for a log-normal income
/// distribution with Gini coefficient `gini`.
pub fn quintile_shares(gini: f64) -> [f64; 5] {
    // Quintile boundaries of the standard normal distribution.
    const Z: [f64; 4] = [
        -0.841_621_233_572_914_3,
        -0.253_347_103_135_799_7,
        0.253_347_103_135_799_7,
        0.841_621_233_572_914_3,
    ];
    let sigma = SQRT_2 * normal_inverse((gini + 1.0) / 2.0);
    let cumulative = |i: usize| match i {
        0 => 0.0,
        5 => 1.0,
        _ => normal_cdf(Z[i - 1] - sigma),
    };
    std::array::from_fn(|q| cumulative(q + 1) - cumulative(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_functions_match_known_values() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-15);
        assert!((normal_cdf(1.96) - 0.975_002_104_851_780_1).abs() < 1e-12);
        for p in [0.001, 0.02, 0.2, 0.5, 0.7, 0.975, 0.999] {
            assert!((normal_cdf(normal_inverse(p)) - p).abs() < 1e-13, "{p}");
        }
        assert!((normal_inverse(0.8) - 0.841_621_233_572_914_3).abs() < 1e-12);
    }

    #[test]
    fn quintile_shares_fit_the_gini() {
        for gini in [0.2, 0.3, 0.45, 0.6] {
            let shares = quintile_shares(gini);
            let total: f64 = shares.iter().sum();
            assert!((total - 1.0).abs() < 1e-12);
            assert!(shares.windows(2).all(|w| w[0] < w[1]), "{shares:?}");
            // Gini of the quintile distribution is slightly lower than the true one.
            let lorenz: f64 = (0..5)
                .map(|i| shares[..=i].iter().sum::<f64>())
                .sum::<f64>();
            let gini_from_quintiles = 1.0 - (2.0 * lorenz - 1.0) / 5.0;
            assert!(
                gini_from_quintiles < gini && gini_from_quintiles > gini - 0.08,
                "{gini} {gini_from_quintiles}"
            );
        }
        let even = quintile_shares(0.01);
        assert!(even.iter().all(|s| (s - 0.2).abs() < 0.01));
    }

    #[test]
    fn rounding_to_significant_digits() {
        assert_eq!(round_significant(1.916_49, 3), 1.92);
        assert_eq!(round_significant(0.750_7, 3), 0.751);
        assert_eq!(round_significant(-1234.5, 2), -1200.0);
        assert_eq!(round_significant(999_999_999_999.9, 3), 1e12);
        assert_eq!(round_significant(0.0, 3), 0.0);
    }

    #[test]
    fn log_position_is_clamped() {
        assert_eq!(log_position(10.0, 100.0, 1000.0), 0.0);
        assert_eq!(log_position(5000.0, 100.0, 1000.0), 1.0);
        assert!((log_position(316.227_766, 100.0, 1000.0) - 0.5).abs() < 1e-6);
    }
}
