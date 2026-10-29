//! Demo: build a Bell state, sample it, then time the CPU and GPU backends.
//!
//! Run with `cargo run --release` (debug builds are ~20x slower).

use qsengine::{QuantumBackend, QuantumState, best_backend, format_bitstring};
use std::time::Instant;

fn main() -> qsengine::Result<()> {
    // --- Bell state: H on q0, then CNOT q0 -> q1 ------------------------------
    let mut bell = best_backend(2)?;
    bell.h(0)?;
    bell.cx(0, 1)?;

    let mut rng = rand::rng();
    let mut counts: Vec<_> = bell.sample(1000, &mut rng).into_iter().collect();
    counts.sort();
    println!("Bell state, 1000 shots:");
    for (index, n) in counts {
        println!("  |{}>  {n}", format_bitstring(index, 2));
    }

    // --- Benchmark ------------------------------------------------------------
    let n = 22;
    let layers = 10;
    println!("\nBenchmark: {n} qubits, {layers} layers of H on every qubit + CNOT chain");

    let cpu_time = time_circuit(&mut QuantumState::new(n)?, n, layers)?;
    println!("  CPU (f64, multithreaded): {cpu_time:>8.1} ms");

    #[cfg(feature = "gpu")]
    match qsengine::GpuQuantumState::new(n) {
        Ok(mut gpu) => {
            let gpu_time = time_circuit(&mut gpu, n, layers)?;
            println!("  GPU (f32, wgpu):          {gpu_time:>8.1} ms  ({:.1}x)", cpu_time / gpu_time);
        }
        Err(e) => println!("  GPU skipped: {e}"),
    }
    Ok(())
}

/// Apply the benchmark circuit and read back probabilities (so GPU timing
/// includes the work actually finishing). Returns milliseconds.
fn time_circuit(state: &mut dyn QuantumBackend, n: usize, layers: usize) -> qsengine::Result<f64> {
    let start = Instant::now();
    for _ in 0..layers {
        for q in 0..n {
            state.h(q)?;
        }
        for q in 0..n - 1 {
            state.cx(q, q + 1)?;
        }
    }
    let p = state.probabilities();
    std::hint::black_box(p);
    Ok(start.elapsed().as_secs_f64() * 1000.0)
}
