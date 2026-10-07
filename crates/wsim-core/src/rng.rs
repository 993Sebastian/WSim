//! Deterministic random numbers (Lastenheft §16.1).
//!
//! Every subsystem and every company draws from its own stream, derived from the game
//! seed and a stream number. A new company therefore never shifts the random numbers
//! of other companies. The generator is xoshiro256++ (pure integer arithmetic, so the
//! results are identical on every platform), seeded via SplitMix64.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimRng {
    state: [u64; 4],
}

/// Stream numbers. Company streams use the upper half of the number range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    World,
    Company(u32),
    /// New plots of a country in a year (M35).
    Plots {
        country: u16,
        year: u16,
    },
    /// Names of a company for a product (M42).
    ProductName {
        company: u32,
        product: u32,
    },
    /// The market of managers in a month (MA1).
    ManagerMarket {
        month: u32,
    },
    /// A manager's checks on a day (MA1).
    Manager {
        id: u32,
        day: u32,
    },
    /// A company's strategy review at a month end (MA5).
    Review {
        company: u32,
        month: u32,
    },
    /// A manager's experience, satisfaction and hiring at a month start (MA6).
    ManagerMonth {
        id: u32,
        month: u32,
    },
    /// The expertise a manager can reach (MA6).
    ManagerPotential {
        id: u32,
    },
    /// A company's hiring and poaching at a month start (MA6).
    Staffing {
        company: u32,
        month: u32,
    },
    /// New start-ups in a month (SU1).
    Ventures {
        month: u32,
    },
    /// A start-up's round and phase in a month (SU1).
    Venture {
        id: u32,
        month: u32,
    },
    /// Founder and draw of a spun-off start-up (SU3).
    SpinOff {
        id: u32,
    },
    /// An AI company's look at the start-ups in a month (SU3).
    VentureBids {
        company: u32,
        month: u32,
    },
    /// How an AI company misjudges cost and margin of a product in a year (B2).
    Estimate {
        company: u32,
        product: u32,
        year: u16,
    },
    /// The change of the tariffs at the start of a year after the data (W3).
    Tariffs {
        year: u16,
    },
    /// Proposals of AI companies for supply contracts in a month (W4).
    Contracts {
        month: u32,
    },
}

impl Stream {
    fn number(self) -> u64 {
        match self {
            Stream::World => 1,
            Stream::Company(id) => (1 << 32) | u64::from(id),
            Stream::Plots { country, year } => {
                (2 << 32) | (u64::from(year) << 16) | u64::from(country)
            }
            Stream::ProductName { company, product } => {
                (3 << 56) | (u64::from(company) << 24) | u64::from(product)
            }
            Stream::ManagerMarket { month } => (4 << 56) | u64::from(month),
            Stream::Manager { id, day } => (5 << 56) | (u64::from(id) << 24) | u64::from(day),
            Stream::Review { company, month } => {
                (6 << 56) | (u64::from(company) << 24) | u64::from(month)
            }
            Stream::ManagerMonth { id, month } => {
                (7 << 56) | (u64::from(id) << 24) | u64::from(month)
            }
            Stream::ManagerPotential { id } => (8 << 56) | u64::from(id),
            Stream::Staffing { company, month } => {
                (9 << 56) | (u64::from(company) << 24) | u64::from(month)
            }
            Stream::Ventures { month } => (10 << 56) | u64::from(month),
            Stream::Venture { id, month } => (11 << 56) | (u64::from(id) << 24) | u64::from(month),
            Stream::SpinOff { id } => (12 << 56) | u64::from(id),
            Stream::VentureBids { company, month } => {
                (13 << 56) | (u64::from(company) << 24) | u64::from(month)
            }
            Stream::Estimate {
                company,
                product,
                year,
            } => {
                (14 << 56)
                    | (u64::from(company) << 32)
                    | (u64::from(product & 0xFFFF) << 16)
                    | u64::from(year)
            }
            Stream::Tariffs { year } => (15 << 56) | u64::from(year),
            Stream::Contracts { month } => (16 << 56) | u64::from(month),
        }
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl SimRng {
    /// Generator for one stream of a game.
    pub fn for_stream(seed: u64, stream: Stream) -> Self {
        let mut mix = stream.number();
        let mut sm = seed ^ splitmix64(&mut mix);
        Self::from_state([
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
        ])
    }

    fn from_state(state: [u64; 4]) -> Self {
        // An all-zero state would only ever produce zeros.
        let state = if state == [0; 4] { [1, 0, 0, 0] } else { state };
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.state;
        let result = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        // The top 53 bits fit the mantissa exactly.
        #[allow(clippy::cast_precision_loss)]
        let value = (self.next_u64() >> 11) as f64;
        value * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in [0, n). `n` must be greater than 0.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "empty range");
        // Rejection sampling avoids the bias of a plain modulo.
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let value = self.next_u64();
            if value < zone {
                return value % n;
            }
        }
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_reference_implementations() {
        // xoshiro256++ reference output for state {1, 2, 3, 4}.
        let mut rng = SimRng::from_state([1, 2, 3, 4]);
        assert_eq!(rng.next_u64(), 41_943_041);
        assert_eq!(rng.next_u64(), 58_720_359);
        assert_eq!(rng.next_u64(), 3_588_806_011_781_223);
        // SplitMix64 reference output for state 0.
        let mut state = 0;
        assert_eq!(splitmix64(&mut state), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn same_seed_same_numbers() {
        let mut a = SimRng::for_stream(42, Stream::World);
        let mut b = SimRng::for_stream(42, Stream::World);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn streams_and_seeds_differ() {
        let first = |seed, stream| SimRng::for_stream(seed, stream).next_u64();
        assert_ne!(first(42, Stream::World), first(43, Stream::World));
        assert_ne!(first(42, Stream::World), first(42, Stream::Company(0)));
        assert_ne!(first(42, Stream::Company(0)), first(42, Stream::Company(1)));
    }

    #[test]
    fn values_stay_in_range() {
        let mut rng = SimRng::for_stream(7, Stream::World);
        let mut counts = [0u32; 6];
        for _ in 0..60_000 {
            let f = rng.next_f64();
            assert!((0.0..1.0).contains(&f));
            counts[usize::try_from(rng.below(6)).unwrap()] += 1;
        }
        // Each face of the die close to 10 000.
        assert!(
            counts.iter().all(|&c| (9_500..10_500).contains(&c)),
            "{counts:?}"
        );
    }

    #[test]
    fn state_survives_serialization() {
        let mut rng = SimRng::for_stream(1, Stream::World);
        rng.next_u64();
        let bytes = rmp_serde::to_vec(&rng).unwrap();
        let mut restored: SimRng = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(rng.next_u64(), restored.next_u64());
    }
}
