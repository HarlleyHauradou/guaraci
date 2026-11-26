//! Random number generators.
//!
//! Histories are grouped in fixed blocks; block `b` always uses stream `b`, so
//! results do not depend on the number of threads.

/// Source of uniform random numbers.
pub trait Rng {
    fn next_u64(&mut self) -> u64;

    /// Uniform in the open interval (0, 1).
    #[inline]
    fn uniform(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 0.5) * (1.0 / (1u64 << 53) as f64)
    }
}

/// A generator that can create independent streams from (seed, stream id).
pub trait StreamRng: Rng + Sized {
    fn stream(seed: u64, stream: u64) -> Self;
}

/// SplitMix64, used to derive well-mixed seeds.
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

// ---- Philox4x32-10 (Salmon et al., SC'11) ----------------------------------

/// Counter-based generator. Key = seed, counter = (word index, stream).
/// The same algorithm runs on the GPU, so CPU and GPU produce the same numbers.
pub struct Philox {
    key: [u32; 2],
    ctr: [u32; 4],
    buf: [u32; 4],
    idx: usize,
}

const PHILOX_M0: u32 = 0xD251_1F53;
const PHILOX_M1: u32 = 0xCD9E_8D57;
const PHILOX_W0: u32 = 0x9E37_79B9;
const PHILOX_W1: u32 = 0xBB67_AE85;

/// One Philox4x32-10 block.
pub fn philox4x32_10(mut ctr: [u32; 4], mut key: [u32; 2]) -> [u32; 4] {
    for round in 0..10 {
        if round > 0 {
            key[0] = key[0].wrapping_add(PHILOX_W0);
            key[1] = key[1].wrapping_add(PHILOX_W1);
        }
        let p0 = (PHILOX_M0 as u64) * (ctr[0] as u64);
        let p1 = (PHILOX_M1 as u64) * (ctr[2] as u64);
        ctr = [
            ((p1 >> 32) as u32) ^ ctr[1] ^ key[0],
            p1 as u32,
            ((p0 >> 32) as u32) ^ ctr[3] ^ key[1],
            p0 as u32,
        ];
    }
    ctr
}

impl Rng for Philox {
    #[inline]
    fn next_u64(&mut self) -> u64 {
        if self.idx >= 4 {
            self.buf = philox4x32_10(self.ctr, self.key);
            self.idx = 0;
            // 64-bit counter in ctr[0..2]; stream id in ctr[2..4]
            let (lo, carry) = self.ctr[0].overflowing_add(1);
            self.ctr[0] = lo;
            if carry {
                self.ctr[1] = self.ctr[1].wrapping_add(1);
            }
        }
        let v = ((self.buf[self.idx] as u64) << 32) | self.buf[self.idx + 1] as u64;
        self.idx += 2;
        v
    }
}

impl StreamRng for Philox {
    fn stream(seed: u64, stream: u64) -> Self {
        Philox {
            key: [seed as u32, (seed >> 32) as u32],
            ctr: [0, 0, stream as u32, (stream >> 32) as u32],
            buf: [0; 4],
            idx: 4,
        }
    }
}

// ---- Xoshiro256++ (Blackman & Vigna) ----------------------------------------

pub struct Xoshiro256pp {
    s: [u64; 4],
}

impl Xoshiro256pp {
    pub fn from_state(s: [u64; 4]) -> Self {
        Xoshiro256pp { s }
    }
}

impl Rng for Xoshiro256pp {
    #[inline]
    fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
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
}

impl StreamRng for Xoshiro256pp {
    fn stream(seed: u64, stream: u64) -> Self {
        let mut sm = seed ^ stream.wrapping_mul(0xD1B5_4A32_D192_ED03);
        let s = [
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
        ];
        Xoshiro256pp { s }
    }
}

// ---- PCG64 (XSL-RR 128/64, O'Neill) -----------------------------------------

pub struct Pcg64 {
    state: u128,
    inc: u128,
}

const PCG_MULT: u128 = 0x2360_ED05_1FC6_5DA4_4385_DF64_9FCC_F645;

impl Pcg64 {
    pub fn new(seed: u128, seq: u128) -> Self {
        let mut p = Pcg64 {
            state: 0,
            inc: (seq << 1) | 1,
        };
        p.step();
        p.state = p.state.wrapping_add(seed);
        p.step();
        p
    }
    #[inline]
    fn step(&mut self) {
        self.state = self.state.wrapping_mul(PCG_MULT).wrapping_add(self.inc);
    }
}

impl Rng for Pcg64 {
    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.step();
        let s = self.state;
        let rot = (s >> 122) as u32;
        (((s >> 64) as u64) ^ (s as u64)).rotate_right(rot)
    }
}

impl StreamRng for Pcg64 {
    fn stream(seed: u64, stream: u64) -> Self {
        let mut sm = seed;
        let s = ((splitmix64(&mut sm) as u128) << 64) | splitmix64(&mut sm) as u128;
        Pcg64::new(s, stream as u128)
    }
}

// ---- MT19937 (Matsumoto & Nishimura), 32-bit -------------------------------

pub struct Mt19937 {
    mt: [u32; 624],
    idx: usize,
}

impl Mt19937 {
    pub fn new(seed: u32) -> Self {
        let mut mt = [0u32; 624];
        mt[0] = seed;
        for i in 1..624 {
            mt[i] = 1_812_433_253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Mt19937 { mt, idx: 624 }
    }

    fn twist(&mut self) {
        for i in 0..624 {
            let y = (self.mt[i] & 0x8000_0000) | (self.mt[(i + 1) % 624] & 0x7fff_ffff);
            let mut v = self.mt[(i + 397) % 624] ^ (y >> 1);
            if y & 1 != 0 {
                v ^= 0x9908_B0DF;
            }
            self.mt[i] = v;
        }
        self.idx = 0;
    }

    pub fn next_u32(&mut self) -> u32 {
        if self.idx >= 624 {
            self.twist();
        }
        let mut y = self.mt[self.idx];
        self.idx += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9D2C_5680;
        y ^= (y << 15) & 0xEFC6_0000;
        y ^ (y >> 18)
    }
}

impl Rng for Mt19937 {
    #[inline]
    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | self.next_u32() as u64
    }
}

impl StreamRng for Mt19937 {
    fn stream(seed: u64, stream: u64) -> Self {
        let mut sm = seed ^ stream.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        Mt19937::new(splitmix64(&mut sm) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn philox_known_answers() {
        // Random123 known-answer tests for philox4x32-10
        assert_eq!(
            philox4x32_10([0; 4], [0; 2]),
            [0x6627_e8d5, 0xe169_c58d, 0xbc57_ac4c, 0x9b00_dbd8]
        );
        assert_eq!(
            philox4x32_10([u32::MAX; 4], [u32::MAX; 2]),
            [0x408f_276d, 0x41c8_3b0e, 0xa20b_c7c6, 0x6d54_51fd]
        );
        assert_eq!(
            philox4x32_10(
                [0x243f_6a88, 0x85a3_08d3, 0x1319_8a2e, 0x0370_7344],
                [0xa409_3822, 0x299f_31d0]
            ),
            [0xd16c_fe09, 0x94fd_cceb, 0x5001_e420, 0x2412_6ea1]
        );
    }

    #[test]
    fn xoshiro_known_answer() {
        let mut x = Xoshiro256pp::from_state([1, 2, 3, 4]);
        assert_eq!(x.next_u64(), 41_943_041);
    }

    #[test]
    fn mt19937_known_answer() {
        let mut m = Mt19937::new(5489);
        assert_eq!(m.next_u32(), 3_499_211_612);
        // 10000th output for the default seed (C++ std::mt19937 requirement)
        let mut m = Mt19937::new(5489);
        let mut v = 0;
        for _ in 0..10_000 {
            v = m.next_u32();
        }
        assert_eq!(v, 4_123_659_995);
    }

    fn moments<R: StreamRng>() {
        let mut r = R::stream(12345, 7);
        let n = 200_000;
        let (mut s, mut s2) = (0.0, 0.0);
        for _ in 0..n {
            let u = r.uniform();
            assert!(u > 0.0 && u < 1.0);
            s += u;
            s2 += u * u;
        }
        let mean = s / n as f64;
        let var = s2 / n as f64 - mean * mean;
        assert!((mean - 0.5).abs() < 0.005, "{mean}");
        assert!((var - 1.0 / 12.0).abs() < 0.002, "{var}");
        // different streams differ
        assert_ne!(R::stream(1, 0).next_u64(), R::stream(1, 1).next_u64());
    }

    #[test]
    fn all_generators_uniform() {
        moments::<Philox>();
        moments::<Xoshiro256pp>();
        moments::<Pcg64>();
        moments::<Mt19937>();
    }
}
