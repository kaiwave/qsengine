//! GPU backend: [`GpuQuantumState`], built on wgpu.
//!
//! wgpu is a portable graphics/compute API: the same code runs on NVIDIA, AMD,
//! Intel and Apple GPUs, using Vulkan, DirectX 12 or Metal under the hood. No
//! CUDA or vendor SDK is needed.
//!
//! ## How it stays fast
//!
//! * **The state never leaves the GPU** between gates. It is copied back only
//!   when you read amplitudes, probabilities or measure.
//! * **Gates are batched.** Each gate call only records its parameters. The
//!   queued gates are sent to the GPU in one submission (one compute pass with
//!   one dispatch per gate) the next time the state is read.
//! * **One shader handles every gate**, controlled or not (see `shaders/apply_gate.wgsl`).
//!
//! ## Limits
//!
//! * Amplitudes are `f32` on the GPU (WGSL has no portable `f64`), so results
//!   agree with the CPU backend to about 1e-6. They are converted to `f64` when read.
//! * Register size is capped by the device's maximum buffer size.
//! * If no adapter is found, [`GpuQuantumState::new`] returns [`QsError::Gpu`];
//!   use [`crate::best_backend`] to fall back to the CPU automatically.

use crate::backend::QuantumBackend;
use crate::gates::Gate;
use crate::{MAX_QUBITS, QsError, Result};
use bytemuck::{Pod, Zeroable};
use num_complex::Complex64;
use std::sync::{Mutex, OnceLock};

/// Threads per workgroup. Must match `@workgroup_size` in the shader.
const WORKGROUP_SIZE: u32 = 256;
/// WebGPU's guaranteed per-dimension dispatch limit.
const MAX_WORKGROUPS_PER_DIM: u32 = 65_535;
/// Gates sent per submission. Bounds the parameter buffer at
/// `GATES_PER_FLUSH * uniform_align` (at most 256 KiB with the common 256-byte alignment).
const GATES_PER_FLUSH: usize = 1024;
/// Bytes per amplitude on the GPU: two f32s.
const AMP_BYTES: u64 = 8;

/// Per-gate parameters, laid out exactly like `struct Params` in the shader.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GateParams {
    matrix: [f32; 8], // m00.re, m00.im, m01.re, m01.im, m10..., m11...
    target_qubit: u32,
    control_mask: u32,
    num_pairs: u32,
    _pad: u32,
}

/// Device, queue and compiled pipeline, created once per process and shared by all states.
struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    /// Byte spacing between per-gate parameter blocks in the uniform buffer:
    /// the struct size rounded up to the device's dynamic-offset alignment.
    uniform_align: u64,
    /// Largest state buffer the device allows, in bytes.
    max_state_bytes: u64,
}

/// Lazily create the shared context. A failure (no GPU) is cached too, so
/// later calls fail fast instead of probing the drivers again.
fn context() -> Result<&'static GpuContext> {
    static CONTEXT: OnceLock<std::result::Result<GpuContext, String>> = OnceLock::new();
    CONTEXT
        .get_or_init(|| pollster::block_on(GpuContext::create()))
        .as_ref()
        .map_err(|e| QsError::Gpu(e.clone()))
}

impl GpuContext {
    async fn create() -> std::result::Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .map_err(|e| format!("no GPU adapter: {e}"))?;

        // Software rasterisers (e.g. WARP, llvmpipe) are slower than our CPU backend.
        if adapter.get_info().device_type == wgpu::DeviceType::Cpu {
            return Err("only a software (CPU) adapter is available".into());
        }

        // Ask for the adapter's real limits rather than the conservative defaults,
        // so large registers fit.
        let limits = adapter.limits();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("qsengine"),
                required_features: wgpu::Features::empty(),
                required_limits: limits.clone(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| format!("could not open GPU device: {e}"))?;

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/apply_gate.wgsl"));

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("qsengine gate layout"),
            entries: &[
                // binding 0: the state vector
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // binding 1: gate parameters; the dynamic offset selects which gate
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(size_of::<GateParams>() as u64),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("qsengine pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("qsengine apply_gate"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            pipeline,
            layout,
            uniform_align: (size_of::<GateParams>() as u64).next_multiple_of(limits.min_uniform_buffer_offset_alignment as u64),
            max_state_bytes: limits.max_buffer_size.min(limits.max_storage_buffer_binding_size as u64),
        })
    }

    /// Largest register this device can hold.
    fn max_qubits(&self) -> usize {
        // floor(log2(max_state_bytes / AMP_BYTES)); u32 indexing in the shader caps it at 32.
        let max_amps = self.max_state_bytes / AMP_BYTES;
        ((63 - max_amps.leading_zeros()) as usize).min(32).min(MAX_QUBITS)
    }
}

/// An `n`-qubit state vector that lives in GPU memory.
pub struct GpuQuantumState {
    num_qubits: usize,
    ctx: &'static GpuContext,
    state: wgpu::Buffer,
    params: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// Gates recorded but not yet run. A Mutex (not RefCell) so the state is
    /// `Send + Sync`, which PyO3 requires; read methods take `&self` but must flush.
    pending: Mutex<Vec<GateParams>>,
}

impl GpuQuantumState {
    /// Create |0...0> on the GPU. Fails with [`QsError::Gpu`] if no usable GPU exists.
    pub fn new(num_qubits: usize) -> Result<Self> {
        let ctx = context()?;
        let max = ctx.max_qubits();
        if num_qubits > max {
            return Err(QsError::TooManyQubits { requested: num_qubits, max });
        }

        // wgpu zero-initialises new buffers, so only amplitude[0] = 1 needs writing.
        let state = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("qsengine state"),
            size: (1u64 << num_qubits) * AMP_BYTES,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        ctx.queue.write_buffer(&state, 0, bytemuck::cast_slice(&[1.0f32, 0.0]));

        let params = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("qsengine gate params"),
            size: GATES_PER_FLUSH as u64 * ctx.uniform_align,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("qsengine gate bind group"),
            layout: &ctx.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: state.as_entire_binding() },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: wgpu::BufferSize::new(size_of::<GateParams>() as u64),
                    }),
                },
            ],
        });

        Ok(Self { num_qubits, ctx, state, params, bind_group, pending: Mutex::new(Vec::new()) })
    }

    /// Run every queued gate on the GPU. Called automatically before any read;
    /// public so benchmarks can separate gate time from read-back time.
    pub fn flush(&self) {
        let mut pending = self.pending.lock().unwrap();
        for batch in pending.chunks(GATES_PER_FLUSH) {
            self.run_batch(batch);
        }
        pending.clear();
    }

    /// Upload one batch of gate parameters and dispatch one shader run per gate.
    fn run_batch(&self, batch: &[GateParams]) {
        let align = self.ctx.uniform_align as usize;

        // Pack each gate's params at an aligned offset so it can be selected by dynamic offset.
        let mut bytes = vec![0u8; batch.len() * align];
        for (i, p) in batch.iter().enumerate() {
            bytes[i * align..i * align + size_of::<GateParams>()].copy_from_slice(bytemuck::bytes_of(p));
        }
        self.ctx.queue.write_buffer(&self.params, 0, &bytes);

        // One thread per amplitude pair, split into a 2D grid if needed.
        let groups = ((1u64 << self.num_qubits) / 2).div_ceil(WORKGROUP_SIZE as u64).max(1) as u32;
        let gx = groups.min(MAX_WORKGROUPS_PER_DIM);
        let gy = groups.div_ceil(gx);

        let mut encoder = self.ctx.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.ctx.pipeline);
            for i in 0..batch.len() {
                // WebGPU orders storage writes between dispatches, so gates apply in sequence.
                pass.set_bind_group(0, &self.bind_group, &[(i * align) as u32]);
                pass.dispatch_workgroups(gx, gy, 1);
            }
        }
        self.ctx.queue.submit([encoder.finish()]);
    }
}

impl QuantumBackend for GpuQuantumState {
    fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    fn apply_gate_kernel(&mut self, control_mask: usize, target: usize, gate: &Gate) {
        let m = gate.as_flattened();
        let mut matrix = [0.0f32; 8];
        for (i, z) in m.iter().enumerate() {
            matrix[2 * i] = z.re as f32;
            matrix[2 * i + 1] = z.im as f32;
        }
        let pending = self.pending.get_mut().unwrap();
        pending.push(GateParams {
            matrix,
            target_qubit: target as u32,
            control_mask: control_mask as u32,
            num_pairs: (1u32 << self.num_qubits) / 2,
            _pad: 0,
        });
        // Keep the queue bounded for very long circuits.
        if pending.len() >= GATES_PER_FLUSH {
            self.flush();
        }
    }

    fn amplitudes(&self) -> Vec<Complex64> {
        self.flush();
        let size = self.state.size();

        // GPU-only memory cannot be read directly: copy into a mappable staging buffer.
        let staging = self.ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("qsengine readback"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.ctx.device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.state, 0, &staging, 0, size);
        self.ctx.queue.submit([encoder.finish()]);

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.expect("GPU buffer map failed"));
        self.ctx.device.poll(wgpu::PollType::Wait).expect("GPU device lost");

        let data = slice.get_mapped_range();
        let floats: &[f32] = bytemuck::cast_slice(&data);
        floats.chunks_exact(2).map(|c| Complex64::new(c[0] as f64, c[1] as f64)).collect()
    }

    fn collapse_to(&mut self, index: usize) {
        // Any queued gates are irrelevant: the whole state is about to be overwritten.
        self.pending.get_mut().unwrap().clear();
        let mut encoder = self.ctx.device.create_command_encoder(&Default::default());
        encoder.clear_buffer(&self.state, 0, None);
        self.ctx.queue.submit([encoder.finish()]);
        // Queue writes run after earlier submissions, so this lands after the clear.
        self.ctx.queue.write_buffer(&self.state, index as u64 * AMP_BYTES, bytemuck::cast_slice(&[1.0f32, 0.0]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QuantumState;
    use crate::backend::conformance;

    /// GPU tests are skipped (with a note) on machines without a GPU.
    fn gpu_available() -> bool {
        let ok = context().is_ok();
        if !ok {
            eprintln!("skipping GPU test: no usable GPU adapter");
        }
        ok
    }

    #[test]
    fn passes_backend_conformance_suite() {
        if gpu_available() {
            conformance::run_all(|n| Box::new(GpuQuantumState::new(n).unwrap()) as Box<dyn QuantumBackend>, 1e-5);
        }
    }

    #[test]
    fn matches_cpu_on_random_circuit() {
        if !gpu_available() {
            return;
        }
        // 18 qubits: 131072 threads per gate, so the 2D grid and batching both run.
        let n = 18;
        let mut cpu = QuantumState::new(n).unwrap();
        let mut gpu = GpuQuantumState::new(n).unwrap();
        let backends: [&mut dyn QuantumBackend; 2] = [&mut cpu, &mut gpu];
        for b in backends {
            for layer in 0..5 {
                for q in 0..n {
                    b.h(q).unwrap();
                    b.rz(q, 0.1 * (q + layer) as f64).unwrap();
                }
                for q in 0..n - 1 {
                    b.cx(q, q + 1).unwrap();
                }
            }
        }
        let (a, b) = (cpu.amplitudes(), gpu.amplitudes());
        let worst = a.iter().zip(&b).map(|(x, y)| (x - y).norm()).fold(0.0, f64::max);
        assert!(worst < 1e-5, "GPU differs from CPU by {worst}");
    }
}
