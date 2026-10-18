use num_complex::Complex;

// Represent an $n$-qubit state as a complex vector of length $2^n$ (using `num-complex`)
use num_complex::Complex;

#[derive(Debug, Clone)]
pub struct QuantumState {
    pub num_qubits: usize,
    pub amplitudes: Vec<Complex<f64>>,
}

impl QuantumState {
    /// Create a zero-state |0...0> for n qubits.
    ///
    /// For n qubits:
    /// - state length = 2^n
    /// - index 0 corresponds to |00...0>
    /// - amplitude at index 0 = 1 + 0i
    /// - all others = 0 + 0i
    pub fn new(num_qubits: usize) -> Self {
        let dim = 1usize << num_qubits;
        let mut amplitudes = vec![Complex::new(0.0, 0.0); dim];
        amplitudes[0] = Complex::new(1.0, 0.0);

        Self {
            num_qubits,
            amplitudes,
        }
    }

    /// Return the total dimension of the state space.
    pub fn dimension(&self) -> usize {
        self.amplitudes.len()
    }

    /// Return the probability of a basis state index.
    ///
    /// This is useful for measurement and debugging.
    pub fn probability_of_index(&self, index: usize) -> f64 {
        let amp = self.amplitudes[index];
        amp.norm_sqr()
    }

    /// Apply a single-qubit gate to a given qubit.
    ///
    /// This is the first “real” quantum operation you will implement.
    /// It works by:
    /// 1. iterating over computational basis states
    /// 2. pairing states that differ on the target qubit
    /// 3. applying a 2x2 unitary matrix to each pair
    pub fn apply_single_qubit_gate(
        &mut self,
        qubit: usize,
        gate: [[Complex<f64>; 2]; 2],
    ) {
        // Pseudocode:
        // for each basis index i:
        //   if bit at qubit is 0:
        //      a = state[i]
        //      b = state[i ^ (1 << qubit)]
        //      state[i] = gate[0][0] * a + gate[0][1] * b
        //      state[i ^ (1 << qubit)] = gate[1][0] * a + gate[1][1] * b
        //
        // This is the standard statevector gate application pattern.
    }

    /// Measure all qubits in the computational basis.
    ///
    /// For each basis state |b>:
    /// - probability = |amplitude|^2
    /// - choose outcome using weighted random sampling
    /// - collapse post-measurement state to the chosen basis vector
    pub fn measure(&mut self) -> usize {
        // 1. compute probabilities
        // 2. sample index with rand::distributions::WeightedIndex
        // 3. collapse amplitudes
        // 4. return measured basis-state index
        0
    }
}

// Implement single-qubit gates ($X, Y, Z, H$) via direct state indexing or Kronecker products.

// Add controlled gates ($CNOT$, $CZ$) and a basic measurement function using weighted random sampling.