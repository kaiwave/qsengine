// Apply one (optionally controlled) 2x2 gate to a state vector on the GPU.
//
// This is the GPU twin of `QuantumState::apply_gate_kernel` in engine.rs.
// One GPU thread handles one amplitude pair (i0, i1) that differs only in the
// target bit, so a 2^n state launches 2^(n-1) threads.
//
// Complex numbers are vec2<f32>: x = real part, y = imaginary part.
// WGSL has no portable f64, so the GPU backend runs in single precision.

struct Params {
    m00: vec2<f32>,       // gate matrix, row-major
    m01: vec2<f32>,
    m10: vec2<f32>,
    m11: vec2<f32>,
    target_qubit: u32,
    control_mask: u32,    // bits that must all be 1 for the gate to act
    num_pairs: u32,       // 2^(n-1): number of threads that do real work
    _pad: u32,            // uniform structs must be 16-byte multiples
};

@group(0) @binding(0) var<storage, read_write> state: array<vec2<f32>>;
@group(0) @binding(1) var<uniform> p: Params;

// (a.x + i a.y) * (b.x + i b.y)
fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(num_workgroups) nwg: vec3<u32>,
) {
    // The dispatch may be 2D (x is capped at 65535 workgroups), so flatten it.
    let k = gid.x + gid.y * nwg.x * 256u;
    if (k >= p.num_pairs) {
        return;
    }

    // Turn pair number k into i0 by inserting a 0 bit at the target position:
    // keep the bits below the target, shift the bits above it left by one.
    let stride = 1u << p.target_qubit;
    let low = k & (stride - 1u);
    let i0 = ((k - low) << 1u) | low;
    if ((i0 & p.control_mask) != p.control_mask) {
        return;
    }
    let i1 = i0 | stride;

    let a = state[i0];
    let b = state[i1];
    state[i0] = cmul(p.m00, a) + cmul(p.m01, b);
    state[i1] = cmul(p.m10, a) + cmul(p.m11, b);
}
