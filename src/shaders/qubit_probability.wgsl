// Probability that one qubit measures as 1: the sum of |amplitude|^2 over every
// basis state with that qubit's bit set. Used for mid-circuit measurement.
//
// A fixed, small grid of threads walks the whole state (each thread loops with a
// stride), each workgroup adds its threads' totals in shared memory, and writes
// one partial sum. The host reads back those few partials and adds them in f64,
// so no second reduction pass is needed.

struct Params {
    target_qubit: u32,
    num_pairs: u32,       // 2^(n-1): one term per pair, from its i1 (bit set) half
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<storage, read> state: array<vec2<f32>>;
@group(0) @binding(1) var<uniform> p: Params;
@group(0) @binding(2) var<storage, read_write> partials: array<f32>;

var<workgroup> sums: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(num_workgroups) nwg: vec3<u32>,
) {
    let stride = 1u << p.target_qubit;
    let threads = nwg.x * 256u;
    var acc = 0.0;
    // Same pair numbering as apply_gate.wgsl: insert the target bit, here set to 1.
    for (var k = gid.x; k < p.num_pairs; k += threads) {
        let low = k & (stride - 1u);
        let a = state[(((k - low) << 1u) | low) | stride];
        acc += dot(a, a);
    }
    sums[lid] = acc;
    workgroupBarrier();
    for (var s = 128u; s > 0u; s >>= 1u) {
        if (lid < s) {
            sums[lid] += sums[lid + s];
        }
        workgroupBarrier();
    }
    if (lid == 0u) {
        partials[wid.x] = sums[0];
    }
}
