//! Multi-threaded CPU engine.
//!
//! Histories are split into fixed blocks of [`BLOCK`] histories; block `b`
//! always uses random stream `b`. Blocks are grouped into batches, and the
//! spread of the batch results gives the statistical uncertainty.

use guaraci_core::problem::Problem;
use guaraci_core::rng::{Mt19937, Pcg64, Philox, StreamRng, Xoshiro256pp};
use guaraci_core::transport::{run_block, Tally};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub const BLOCK: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RngChoice {
    Philox,
    Xoshiro,
    Pcg,
    Mt19937,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub histories: u64,
    pub batches: u32,
    pub seed: u64,
    pub rng: RngChoice,
    /// 0 = all cores.
    pub threads: usize,
}

/// First and second moments of the per-history batch means.
#[derive(Debug, Clone)]
pub struct Results {
    /// Sum over all histories (per probe, per cell).
    pub total: Tally,
    /// Σ over batches of (batch sum / batch histories)², per probe and cell.
    pub sum_sq_means: Vec<Vec<f64>>,
    pub batches: u32,
    pub elapsed: Duration,
    pub threads: usize,
    /// True if the run was stopped before the end.
    pub interrupted: bool,
}

impl Results {
    /// Mean per history and its standard error, for a probe cell.
    pub fn mean_and_error(&self, probe: usize, cell: usize) -> (f64, f64) {
        let n = self.total.histories.max(1) as f64;
        let mean = self.total.cells[probe][cell] / n;
        let b = self.batches as f64;
        if self.batches < 2 {
            return (mean, f64::NAN);
        }
        let var = (self.sum_sq_means[probe][cell] / b - mean * mean).max(0.0) * b / (b - 1.0);
        (mean, (var / b).sqrt())
    }
}

/// Run the simulation. `done` counts finished histories (for progress bars);
/// setting `stop` ends the run after the current batch.
pub fn run(problem: &Problem, opts: &Options, done: &AtomicU64, stop: &AtomicBool) -> Results {
    match opts.rng {
        RngChoice::Philox => run_with::<Philox>(problem, opts, done, stop),
        RngChoice::Xoshiro => run_with::<Xoshiro256pp>(problem, opts, done, stop),
        RngChoice::Pcg => run_with::<Pcg64>(problem, opts, done, stop),
        RngChoice::Mt19937 => run_with::<Mt19937>(problem, opts, done, stop),
    }
}

fn run_with<R: StreamRng>(problem: &Problem, opts: &Options, done: &AtomicU64, stop: &AtomicBool) -> Results {
    let start = Instant::now();
    let mut builder = rayon::ThreadPoolBuilder::new();
    if opts.threads > 0 {
        builder = builder.num_threads(opts.threads);
    }
    let pool = builder.build().expect("thread pool");
    let threads = pool.current_num_threads();

    let nblocks = opts.histories.div_ceil(BLOCK);
    let batches = (opts.batches as u64).clamp(1, nblocks.max(1));
    let mut total = Tally::new(problem);
    let mut sum_sq: Vec<Vec<f64>> = total.cells.iter().map(|c| vec![0.0; c.len()]).collect();
    let mut done_batches = 0u32;
    let mut interrupted = false;

    pool.install(|| {
        for b in 0..batches {
            if stop.load(Ordering::Relaxed) {
                interrupted = true;
                break;
            }
            let (first, last) = (b * nblocks / batches, (b + 1) * nblocks / batches);
            let blocks: Vec<u64> = (first..last).collect();
            let min_len = (blocks.len() / (threads * 4)).max(1);
            let batch = blocks
                .par_iter()
                .with_min_len(min_len)
                .fold(
                    || Tally::new(problem),
                    |mut t, &blk| {
                        let count = BLOCK.min(opts.histories - blk * BLOCK);
                        let mut rng = R::stream(opts.seed, blk);
                        run_block(problem, &mut rng, count, &mut t);
                        done.fetch_add(count, Ordering::Relaxed);
                        t
                    },
                )
                .reduce(
                    || Tally::new(problem),
                    |mut a, b| {
                        a.add(&b);
                        a
                    },
                );
            let n = batch.histories.max(1) as f64;
            for (s, c) in sum_sq.iter_mut().zip(&batch.cells) {
                for (x, v) in s.iter_mut().zip(c) {
                    let m = v / n;
                    *x += m * m;
                }
            }
            total.add(&batch);
            done_batches += 1;
        }
    });

    Results {
        total,
        sum_sq_means: sum_sq,
        batches: done_batches,
        elapsed: start.elapsed(),
        threads,
        interrupted,
    }
}
