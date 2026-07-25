//! fusion_gpu_kernels — GPU kernel dispatch abstraction with CPU fallbacks.
//!
//! Provides matrix multiplication (naive + tiled), activation functions
//! (ReLU, GELU, SiLU), softmax, and layer normalization kernels.
//! All kernels have CPU reference implementations; GPU dispatch is an
//! abstraction layer for future CUDA/Metal/Vulkan backends.

use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("device error: {0}")]
    Device(String),

    #[error("shape mismatch: {0}")]
    ShapeMismatch(String),

    #[error("unsupported kernel: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, KernelError>;

// ---------------------------------------------------------------------------
// Device abstraction
// ---------------------------------------------------------------------------

/// Available compute devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Cpu,
    Cuda,
    Metal,
    Vulkan,
}

/// Descriptor for a compute device.
#[derive(Debug, Clone)]
pub struct Device {
    pub device_type: DeviceType,
    pub name: String,
    pub memory_bytes: usize,
    pub compute_units: usize,
}

impl Device {
    pub fn cpu() -> Self {
        Self {
            device_type: DeviceType::Cpu,
            name: "CPU".to_string(),
            memory_bytes: usize::MAX,
            compute_units: num_cpus().min(256),
        }
    }

    pub fn cuda(id: usize, memory_bytes: usize, compute_units: usize) -> Self {
        Self {
            device_type: DeviceType::Cuda,
            name: format!("CUDA:{id}"),
            memory_bytes,
            compute_units,
        }
    }

    pub fn is_gpu(&self) -> bool {
        matches!(self.device_type, DeviceType::Cuda | DeviceType::Metal | DeviceType::Vulkan)
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Kernel dispatch backend — determines where kernels execute.
pub trait KernelBackend: Send + Sync {
    fn device(&self) -> &Device;

    fn matmul_naive(
        &self,
        a: &[f32],
        b: &[f32],
        m: usize,
        k: usize,
        n: usize,
    ) -> Result<Vec<f32>>;

    fn matmul_tiled(
        &self,
        a: &[f32],
        b: &[f32],
        m: usize,
        k: usize,
        n: usize,
        tile_size: usize,
    ) -> Result<Vec<f32>>;

    fn relu(&self, input: &[f32], output: &mut [f32]) -> Result<()>;
    fn gelu(&self, input: &[f32], output: &mut [f32]) -> Result<()>;
    fn silu(&self, input: &[f32], output: &mut [f32]) -> Result<()>;

    fn softmax(&self, input: &[f32], output: &mut [f32], rows: usize, cols: usize) -> Result<()>;

    fn layer_norm(
        &self,
        input: &[f32],
        gamma: &[f32],
        beta: &[f32],
        output: &mut [f32],
        rows: usize,
        cols: usize,
        eps: f32,
    ) -> Result<()>;
}

// ---------------------------------------------------------------------------
// CPU fallback implementations
// ---------------------------------------------------------------------------

/// CPU reference implementation of all kernels.
pub struct CpuBackend {
    device: Device,
    thread_count: usize,
}

impl CpuBackend {
    pub fn new() -> Self {
        let device = Device::cpu();
        let thread_count = device.compute_units;
        Self { device, thread_count }
    }

    pub fn with_threads(thread_count: usize) -> Self {
        let mut device = Device::cpu();
        device.compute_units = thread_count;
        Self { device, thread_count }
    }
}

impl Default for CpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelBackend for CpuBackend {
    fn device(&self) -> &Device {
        &self.device
    }

    fn matmul_naive(
        &self,
        a: &[f32],
        b: &[f32],
        m: usize,
        k: usize,
        n: usize,
    ) -> Result<Vec<f32>> {
        if a.len() != m * k {
            return Err(KernelError::ShapeMismatch(format!(
                "a has {} elements, expected {}",
                a.len(),
                m * k
            )));
        }
        if b.len() != k * n {
            return Err(KernelError::ShapeMismatch(format!(
                "b has {} elements, expected {}",
                b.len(),
                k * n
            )));
        }

        let mut c = vec![0.0f32; m * n];

        for i in 0..m {
            for p in 0..k {
                let a_val = a[i * k + p];
                for j in 0..n {
                    c[i * n + j] += a_val * b[p * n + j];
                }
            }
        }

        Ok(c)
    }

    fn matmul_tiled(
        &self,
        a: &[f32],
        b: &[f32],
        m: usize,
        k: usize,
        n: usize,
        tile_size: usize,
    ) -> Result<Vec<f32>> {
        if a.len() != m * k || b.len() != k * n {
            return Err(KernelError::ShapeMismatch("matmul dimension mismatch".to_string()));
        }

        let mut c = vec![0.0f32; m * n];

        // Tiled matrix multiplication — improves cache locality
        for ii in (0..m).step_by(tile_size) {
            for jj in (0..n).step_by(tile_size) {
                for pp in (0..k).step_by(tile_size) {
                    let i_end = (ii + tile_size).min(m);
                    let j_end = (jj + tile_size).min(n);
                    let p_end = (pp + tile_size).min(k);

                    for i in ii..i_end {
                        for p in pp..p_end {
                            let a_val = a[i * k + p];
                            for j in jj..j_end {
                                c[i * n + j] += a_val * b[p * n + j];
                            }
                        }
                    }
                }
            }
        }

        Ok(c)
    }

    fn relu(&self, input: &[f32], output: &mut [f32]) -> Result<()> {
        if input.len() != output.len() {
            return Err(KernelError::ShapeMismatch(format!(
                "input len {} != output len {}",
                input.len(),
                output.len()
            )));
        }
        for (o, &x) in output.iter_mut().zip(input.iter()) {
            *o = x.max(0.0);
        }
        Ok(())
    }

    fn gelu(&self, input: &[f32], output: &mut [f32]) -> Result<()> {
        if input.len() != output.len() {
            return Err(KernelError::ShapeMismatch(format!(
                "input len {} != output len {}",
                input.len(),
                output.len()
            )));
        }
        for (o, &x) in output.iter_mut().zip(input.iter()) {
            // Approximate GELU: x * σ(1.702x)
            let sigma = 1.0 / (1.0 + (-1.702 * x).exp());
            *o = x * sigma;
        }
        Ok(())
    }

    fn silu(&self, input: &[f32], output: &mut [f32]) -> Result<()> {
        if input.len() != output.len() {
            return Err(KernelError::ShapeMismatch(format!(
                "input len {} != output len {}",
                input.len(),
                output.len()
            )));
        }
        for (o, &x) in output.iter_mut().zip(input.iter()) {
            let sigma = 1.0 / (1.0 + (-x).exp());
            *o = x * sigma;
        }
        Ok(())
    }

    fn softmax(&self, input: &[f32], output: &mut [f32], rows: usize, cols: usize) -> Result<()> {
        if input.len() != rows * cols || output.len() != rows * cols {
            return Err(KernelError::ShapeMismatch(format!(
                "softmax: input {} or output {} doesn't match rows×cols={}",
                input.len(),
                output.len(),
                rows * cols
            )));
        }
        for r in 0..rows {
            let offset = r * cols;
            let row = &input[offset..offset + cols];

            // Numerically stable: subtract max
            let max_val = row.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let exp_sum: f32 = row.iter().map(|v| (v - max_val).exp()).sum();

            for (j, &v) in row.iter().enumerate() {
                output[offset + j] = (v - max_val).exp() / exp_sum;
            }
        }
        Ok(())
    }

    fn layer_norm(
        &self,
        input: &[f32],
        gamma: &[f32],
        beta: &[f32],
        output: &mut [f32],
        rows: usize,
        cols: usize,
        eps: f32,
    ) -> Result<()> {
        if input.len() != rows * cols || output.len() != rows * cols {
            return Err(KernelError::ShapeMismatch(format!(
                "layer_norm: input {} or output {} doesn't match rows×cols={}",
                input.len(),
                output.len(),
                rows * cols
            )));
        }
        if gamma.len() != cols || beta.len() != cols {
            return Err(KernelError::ShapeMismatch(format!(
                "gamma len {} or beta len {} doesn't match cols {}",
                gamma.len(),
                beta.len(),
                cols
            )));
        }

        for r in 0..rows {
            let offset = r * cols;
            let row = &input[offset..offset + cols];

            let mean: f32 = row.iter().sum::<f32>() / cols as f32;
            let variance: f32 = row.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / cols as f32;
            let inv_std = 1.0 / (variance + eps).sqrt();

            for j in 0..cols {
                let normalized = (row[j] - mean) * inv_std;
                output[offset + j] = normalized * gamma[j] + beta[j];
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CUDA kernel dispatch (stub)
// ---------------------------------------------------------------------------

/// CUDA kernel backend — dispatches to GPU kernels via FFI.
///
/// In production, this would use cuda-sys or a CUDA runtime wrapper to launch
/// kernels. The actual kernel implementations live in `.cu` files and are
/// compiled with nvcc. This struct provides the Rust-side dispatch logic.
pub struct CudaBackend {
    device: Device,
    stream: u64, // CUDA stream handle
}

impl CudaBackend {
    /// Create a new CUDA backend on the given device.
    ///
    /// # Safety
    /// Caller must ensure CUDA is initialized and the device is valid.
    pub fn new(device_id: usize) -> Result<Self> {
        // In production: cudaSetDevice(device_id), cudaStreamCreate
        let device = Device::cuda(device_id, 8 * 1024 * 1024 * 1024, 128);
        Ok(Self {
            device,
            stream: 0, // default stream
        })
    }
}

impl KernelBackend for CudaBackend {
    fn device(&self) -> &Device {
        &self.device
    }

    fn matmul_naive(
        &self,
        _a: &[f32],
        _b: &[f32],
        _m: usize,
        _k: usize,
        _n: usize,
    ) -> Result<Vec<f32>> {
        // In production: launch CUDA matmul kernel
        // For now, delegate to CPU
        Err(KernelError::Unsupported(
            "CUDA matmul kernel not yet compiled — add nvcc build step and FFI bindings".to_string(),
        ))
    }

    fn matmul_tiled(
        &self,
        _a: &[f32],
        _b: &[f32],
        _m: usize,
        _k: usize,
        _n: usize,
        _tile_size: usize,
    ) -> Result<Vec<f32>> {
        Err(KernelError::Unsupported(
            "CUDA tiled matmul kernel not yet compiled".to_string(),
        ))
    }

    fn relu(&self, _input: &[f32], _output: &mut [f32]) -> Result<()> {
        Err(KernelError::Unsupported("CUDA ReLU kernel not yet compiled".to_string()))
    }

    fn gelu(&self, _input: &[f32], _output: &mut [f32]) -> Result<()> {
        Err(KernelError::Unsupported("CUDA GELU kernel not yet compiled".to_string()))
    }

    fn silu(&self, _input: &[f32], _output: &mut [f32]) -> Result<()> {
        Err(KernelError::Unsupported("CUDA SiLU kernel not yet compiled".to_string()))
    }

    fn softmax(&self, _input: &[f32], _output: &mut [f32], _rows: usize, _cols: usize) -> Result<()> {
        Err(KernelError::Unsupported("CUDA softmax kernel not yet compiled".to_string()))
    }

    fn layer_norm(
        &self,
        _input: &[f32],
        _gamma: &[f32],
        _beta: &[f32],
        _output: &mut [f32],
        _rows: usize,
        _cols: usize,
        _eps: f32,
    ) -> Result<()> {
        Err(KernelError::Unsupported("CUDA layer_norm kernel not yet compiled".to_string()))
    }
}

// ---------------------------------------------------------------------------
// Kernel dispatcher — picks the best backend for the device
// ---------------------------------------------------------------------------

/// Top-level kernel dispatcher that selects the appropriate backend.
pub struct KernelDispatcher {
    cpu: CpuBackend,
}

impl KernelDispatcher {
    pub fn new() -> Self {
        Self {
            cpu: CpuBackend::new(),
        }
    }

    pub fn with_cpu_threads(threads: usize) -> Self {
        Self {
            cpu: CpuBackend::with_threads(threads),
        }
    }

    /// Get the CPU backend directly.
    pub fn cpu(&self) -> &CpuBackend {
        &self.cpu
    }

    // Convenience methods that delegate to CPU backend

    pub fn matmul_naive(&self, a: &[f32], b: &[f32], m: usize, k: usize, n: usize) -> Result<Vec<f32>> {
        self.cpu.matmul_naive(a, b, m, k, n)
    }

    pub fn matmul_tiled(&self, a: &[f32], b: &[f32], m: usize, k: usize, n: usize, tile: usize) -> Result<Vec<f32>> {
        self.cpu.matmul_tiled(a, b, m, k, n, tile)
    }

    pub fn relu(&self, input: &[f32], output: &mut [f32]) -> Result<()> {
        self.cpu.relu(input, output)
    }

    pub fn gelu(&self, input: &[f32], output: &mut [f32]) -> Result<()> {
        self.cpu.gelu(input, output)
    }

    pub fn silu(&self, input: &[f32], output: &mut [f32]) -> Result<()> {
        self.cpu.silu(input, output)
    }

    pub fn softmax(&self, input: &[f32], output: &mut [f32], rows: usize, cols: usize) -> Result<()> {
        self.cpu.softmax(input, output, rows, cols)
    }

    pub fn layer_norm(
        &self,
        input: &[f32],
        gamma: &[f32],
        beta: &[f32],
        output: &mut [f32],
        rows: usize,
        cols: usize,
        eps: f32,
    ) -> Result<()> {
        self.cpu.layer_norm(input, gamma, beta, output, rows, cols, eps)
    }
}

impl Default for KernelDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Device ----

    #[test]
    fn test_device_cpu() {
        let d = Device::cpu();
        assert_eq!(d.device_type, DeviceType::Cpu);
        assert!(!d.is_gpu());
        assert!(d.compute_units > 0);
    }

    #[test]
    fn test_device_cuda() {
        let d = Device::cuda(0, 8_000_000_000, 128);
        assert_eq!(d.device_type, DeviceType::Cuda);
        assert!(d.is_gpu());
    }

    // ---- Matmul naive ----

    #[test]
    fn test_matmul_naive_identity() {
        let cpu = CpuBackend::new();
        // 2×2 identity matrix
        let a = vec![1.0, 0.0, 0.0, 1.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let c = cpu.matmul_naive(&a, &b, 2, 2, 2).unwrap();
        assert!((c[0] - 5.0).abs() < 1e-6);
        assert!((c[1] - 6.0).abs() < 1e-6);
        assert!((c[2] - 7.0).abs() < 1e-6);
        assert!((c[3] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_matmul_naive_3x3() {
        let cpu = CpuBackend::new();
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let b = vec![9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let c = cpu.matmul_naive(&a, &b, 3, 3, 3).unwrap();
        // 1*9+2*6+3*3 = 9+12+9 = 30
        assert!((c[0] - 30.0).abs() < 1e-5);
        // 1*8+2*5+3*2 = 8+10+6 = 24
        assert!((c[1] - 24.0).abs() < 1e-5);
    }

    #[test]
    fn test_matmul_naive_shape_error() {
        let cpu = CpuBackend::new();
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0];
        assert!(cpu.matmul_naive(&a, &b, 2, 2, 1).is_err());
    }

    // ---- Matmul tiled ----

    #[test]
    fn test_matmul_tiled_matches_naive() {
        let cpu = CpuBackend::new();
        let a: Vec<f32> = (0..20).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..20).map(|i| (i + 1) as f32).collect();
        // 4×5 × 5×4 = 4×4
        let c_naive = cpu.matmul_naive(&a, &b, 4, 5, 4).unwrap();
        let c_tiled = cpu.matmul_tiled(&a, &b, 4, 5, 4, 2).unwrap();
        for i in 0..16 {
            assert!(
                (c_naive[i] - c_tiled[i]).abs() < 1e-4,
                "mismatch at [{i}]: naive={} tiled={}",
                c_naive[i],
                c_tiled[i]
            );
        }
    }

    #[test]
    fn test_matmul_tiled_different_tile_sizes() {
        let cpu = CpuBackend::new();
        let a: Vec<f32> = (0..36).map(|i| i as f32).collect();
        let b: Vec<f32> = (1..37).map(|i| i as f32).collect();
        // 6×6 × 6×6 = 6×6
        let c1 = cpu.matmul_tiled(&a, &b, 6, 6, 6, 2).unwrap();
        let c2 = cpu.matmul_tiled(&a, &b, 6, 6, 6, 3).unwrap();
        let c3 = cpu.matmul_tiled(&a, &b, 6, 6, 6, 6).unwrap();
        for i in 0..36 {
            assert!((c1[i] - c2[i]).abs() < 1e-4);
            assert!((c2[i] - c3[i]).abs() < 1e-4);
        }
    }

    // ---- ReLU ----

    #[test]
    fn test_relu() {
        let cpu = CpuBackend::new();
        let input = vec![-3.0, -1.0, 0.0, 1.0, 3.0];
        let mut output = vec![0.0; 5];
        cpu.relu(&input, &mut output).unwrap();
        assert_eq!(output, vec![0.0, 0.0, 0.0, 1.0, 3.0]);
    }

    #[test]
    fn test_relu_shape_error() {
        let cpu = CpuBackend::new();
        let input = vec![1.0, 2.0];
        let mut output = vec![0.0; 3];
        assert!(cpu.relu(&input, &mut output).is_err());
    }

    // ---- GELU ----

    #[test]
    fn test_gelu_zero() {
        let cpu = CpuBackend::new();
        let input = vec![0.0];
        let mut output = vec![0.0; 1];
        cpu.gelu(&input, &mut output).unwrap();
        assert!(output[0].abs() < 0.01, "GELU(0) should be ~0, got {}", output[0]);
    }

    #[test]
    fn test_gelu_positive() {
        let cpu = CpuBackend::new();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        cpu.gelu(&input, &mut output).unwrap();
        // GELU should be close to identity for positive values
        assert!(output[0] > 0.8, "GELU(1) should be > 0.8, got {}", output[0]);
        assert!(output[1] > 1.8, "GELU(2) should be > 1.8, got {}", output[1]);
    }

    #[test]
    fn test_gelu_negative() {
        let cpu = CpuBackend::new();
        let input = vec![-1.0, -2.0];
        let mut output = vec![0.0; 2];
        cpu.gelu(&input, &mut output).unwrap();
        // GELU suppresses negative values
        assert!(output[0].abs() < 0.5, "GELU(-1) should be small, got {}", output[0]);
        assert!(output[1].abs() < 0.1, "GELU(-2) should be very small, got {}", output[1]);
    }

    // ---- SiLU ----

    #[test]
    fn test_silu_zero() {
        let cpu = CpuBackend::new();
        let input = vec![0.0];
        let mut output = vec![0.0; 1];
        cpu.silu(&input, &mut output).unwrap();
        assert!(output[0].abs() < 1e-6, "SiLU(0) should be 0, got {}", output[0]);
    }

    #[test]
    fn test_silu_positive() {
        let cpu = CpuBackend::new();
        let input = vec![2.0];
        let mut output = vec![0.0; 1];
        cpu.silu(&input, &mut output).unwrap();
        let expected = 2.0 / (1.0 + (-2.0f32).exp());
        assert!((output[0] - expected).abs() < 1e-5);
    }

    #[test]
    fn test_silu_monotonic() {
        let cpu = CpuBackend::new();
        // SiLU is monotonically increasing for x > -1.28 (its minimum)
        let input: Vec<f32> = (-1..=5).map(|i| i as f32).collect();
        let mut output = vec![0.0; input.len()];
        cpu.silu(&input, &mut output).unwrap();
        for i in 1..output.len() {
            assert!(output[i] >= output[i - 1], "SiLU should be monotonic for x >= -1");
        }
    }

    // ---- Softmax ----

    #[test]
    fn test_softmax_sums_to_one() {
        let cpu = CpuBackend::new();
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let mut output = vec![0.0; 6];
        cpu.softmax(&input, &mut output, 2, 3).unwrap();
        let sum1: f32 = output[0..3].iter().sum();
        let sum2: f32 = output[3..6].iter().sum();
        assert!((sum1 - 1.0).abs() < 1e-5, "row 1 should sum to 1, got {sum1}");
        assert!((sum2 - 1.0).abs() < 1e-5, "row 2 should sum to 1, got {sum2}");
    }

    #[test]
    fn test_softmax_peak() {
        let cpu = CpuBackend::new();
        let input = vec![0.0, 0.0, 100.0];
        let mut output = vec![0.0; 3];
        cpu.softmax(&input, &mut output, 1, 3).unwrap();
        assert!(output[2] > 0.99, "softmax should peak at 100.0, got {}", output[2]);
    }

    #[test]
    fn test_softmax_uniform() {
        let cpu = CpuBackend::new();
        let input = vec![5.0, 5.0, 5.0];
        let mut output = vec![0.0; 3];
        cpu.softmax(&input, &mut output, 1, 3).unwrap();
        for &v in &output {
            assert!((v - 1.0 / 3.0).abs() < 1e-5, "uniform input should give uniform output");
        }
    }

    // ---- Layer norm ----

    #[test]
    fn test_layer_norm_zero_mean() {
        let cpu = CpuBackend::new();
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let gamma = vec![1.0, 1.0, 1.0];
        let beta = vec![0.0, 0.0, 0.0];
        let mut output = vec![0.0; 6];
        cpu.layer_norm(&input, &gamma, &beta, &mut output, 2, 3, 1e-5).unwrap();

        // Each row should have zero mean
        for r in 0..2 {
            let mean: f32 = output[r * 3..r * 3 + 3].iter().sum::<f32>() / 3.0;
            assert!(mean.abs() < 1e-4, "row {r} mean should be ~0, got {mean}");
        }
    }

    #[test]
    fn test_layer_norm_with_beta() {
        let cpu = CpuBackend::new();
        let input = vec![1.0, 1.0, 1.0];
        let gamma = vec![1.0, 1.0, 1.0];
        let beta = vec![5.0, 5.0, 5.0];
        let mut output = vec![0.0; 3];
        cpu.layer_norm(&input, &gamma, &beta, &mut output, 1, 3, 1e-5).unwrap();
        // Input is constant → normalized = 0 → output = beta
        for &v in &output {
            assert!((v - 5.0).abs() < 1e-4, "constant input + beta=5 should give 5, got {v}");
        }
    }

    #[test]
    fn test_layer_norm_shape_error() {
        let cpu = CpuBackend::new();
        let input = vec![1.0, 2.0];
        let gamma = vec![1.0];
        let beta = vec![0.0];
        let mut output = vec![0.0; 2];
        assert!(cpu.layer_norm(&input, &gamma, &beta, &mut output, 2, 2, 1e-5).is_err());
    }

    // ---- Dispatcher ----

    #[test]
    fn test_dispatcher_matmul_naive() {
        let d = KernelDispatcher::new();
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let c = d.matmul_naive(&a, &b, 2, 2, 2).unwrap();
        // 1*5+2*7=19, 1*6+2*8=22, 3*5+4*7=43, 3*6+4*8=50
        assert!((c[0] - 19.0).abs() < 1e-5);
        assert!((c[1] - 22.0).abs() < 1e-5);
        assert!((c[2] - 43.0).abs() < 1e-5);
        assert!((c[3] - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_dispatcher_relu() {
        let d = KernelDispatcher::new();
        let input = vec![-1.0, 0.0, 1.0];
        let mut output = vec![0.0; 3];
        d.relu(&input, &mut output).unwrap();
        assert_eq!(output, vec![0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_dispatcher_softmax() {
        let d = KernelDispatcher::new();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        d.softmax(&input, &mut output, 1, 3).unwrap();
        let sum: f32 = output.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    // ---- KernelDispatcher default ----

    #[test]
    fn test_kernel_dispatcher_default() {
        let d = KernelDispatcher::default();
        let input = vec![-2.0, 3.0];
        let mut output = vec![0.0; 2];
        d.relu(&input, &mut output).unwrap();
        assert_eq!(output, vec![0.0, 3.0]);
    }

    // ---- Large matmul consistency ----

    #[test]
    fn test_matmul_large() {
        let cpu = CpuBackend::new();
        let m = 16;
        let k = 32;
        let n = 16;
        let a: Vec<f32> = (0..(m * k)).map(|i| (i as f32) * 0.01).collect();
        let b: Vec<f32> = (0..(k * n)).map(|i| (i as f32) * 0.01).collect();
        let c_naive = cpu.matmul_naive(&a, &b, m, k, n).unwrap();
        let c_tiled = cpu.matmul_tiled(&a, &b, m, k, n, 4).unwrap();
        assert_eq!(c_naive.len(), m * n);
        for i in 0..(m * n) {
            assert!(
                (c_naive[i] - c_tiled[i]).abs() < 1e-3,
                "mismatch at [{i}]: naive={} tiled={}",
                c_naive[i],
                c_tiled[i]
            );
        }
    }
}
