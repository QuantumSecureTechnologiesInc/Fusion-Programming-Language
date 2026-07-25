> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 16: Tensor Types and AI/ML

This chapter introduces Fusion's native artificial intelligence and machine learning capabilities—the second pillar of Tri-brid computing. Unlike languages that require external libraries for numerical computing (NumPy in Python, ndarray in Rust), Fusion integrates **tensors** and **automatic differentiation** as first-class language features.

Whether you're training a large language model, deploying a computer vision system, or building a recommendation engine, Fusion's AI infrastructure provides everything you need: hardware acceleration, distributed training, and type-safe neural networks—all without leaving the language.

This chapter covers:

- The `Tensor<T>` type and tensor operations
- GPU acceleration and device management
- Automatic differentiation for gradient computation
- Building neural networks with the `nn` module
- Training loops and optimisation strategies
- Best practices for production ML systems

---

## 16.1 Understanding Tensors

A **tensor** is a multi-dimensional array—the fundamental data structure of machine learning. Scalars, vectors, and matrices are all special cases of tensors.

### 16.1.1 What Is a Tensor?

In mathematical terms:
- A **scalar** (0-dimensional tensor) is a single number: `42`
- A **vector** (1-dimensional tensor) is an array: `[1, 2, 3, 4]`
- A **matrix** (2-dimensional tensor) is a 2D grid: `[[1, 2], [3, 4]]`
- A **tensor** (n-dimensional) extends this to arbitrary dimensions

In deep learning, we work with tensors constantly:
- An image is a 3D tensor: `[height, width, channels]`
- A batch of images is 4D: `[batch_size, height, width, channels]`
- A sequence of embeddings is 3D: `[batch_size, sequence_length, embedding_dim]`

Fusion's `Tensor<T>` type handles all these cases efficiently.

### 16.1.2 Creating Tensors

The `Tensor` type is parameterised by its element type:

```fusion
use fusion_ai_core::Tensor

fn main() {
    // Tensor from literal values
    let v = Tensor::from_vec(vec![1.0, 2.0, 3.0, 4.0])
    println("Vector: {:?}", v)
    println("Shape: {:?}", v.shape())  // [4]

    // Tensor with explicit shape
    let m = Tensor::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
        .reshape([2, 3])
    println("Matrix shape: {:?}", m.shape())  // [2, 3]

    // Convenience constructors
    let zeros = Tensor::<f32>::zeros([3, 4])       // All zeros
    let ones = Tensor::<f32>::ones([2, 2])         // All ones
    let random = Tensor::<f32>::rand([100, 100])   // Uniform [0, 1)
    let randn = Tensor::<f32>::randn([64, 128])    // Normal distribution

    // Identity matrix
    let eye = Tensor::<f32>::eye(4)  // 4×4 identity

    // Range tensor
    let range = Tensor::arange(0.0, 10.0, 0.5)  // [0.0, 0.5, 1.0, ..., 9.5]
}
```text

### 16.1.3 Tensor Properties

Every tensor has several key properties:

```fusion
let t = Tensor::<f32>::rand([2, 3, 4])

// Shape: dimensions of the tensor
let shape: Vec<usize> = t.shape()    // [2, 3, 4]

// Rank/Dimensions: number of dimensions
let rank: usize = t.ndim()           // 3

// Total elements
let numel: usize = t.numel()         // 2 * 3 * 4 = 24

// Data type
let dtype: DataType = t.dtype()      // Float32

// Device location
let device: Device = t.device()      // CPU or CUDA(0)

// Memory layout (row-major by default)
let strides: Vec<usize> = t.strides()  // [12, 4, 1]

// Does this tensor track gradients?
let requires_grad: bool = t.requires_grad()  // false
```text

### 16.1.4 Data Types

Fusion supports multiple numerical precisions:

```fusion
// Floating point
let f16_tensor = Tensor::<f16>::zeros([10, 10])   // Half precision
let f32_tensor = Tensor::<f32>::zeros([10, 10])   // Single precision
let f64_tensor = Tensor::<f64>::zeros([10, 10])   // Double precision
let bf16_tensor = Tensor::<bf16>::zeros([10, 10]) // Brain float

// Integer types
let i32_tensor = Tensor::<i32>::zeros([10, 10])
let i64_tensor = Tensor::<i64>::zeros([10, 10])

// Boolean
let bool_tensor = Tensor::<bool>::zeros([10, 10])

// Complex (for quantum computing integration)
let complex_tensor = Tensor::<Complex<f64>>::zeros([10, 10])
```text

**Choosing a data type:**
- Use `f32` for most training (good balance of precision and speed)
- Use `f16` or `bf16` for memory-constrained scenarios or mixed-precision training
- Use `f64` when numerical stability is critical (gradients, loss computation)
- Use `i64` for indices and discrete values

---

## 16.2 Tensor Operations

Fusion provides a comprehensive library of tensor operations, from basic arithmetic to advanced linear algebra.

### 16.2.1 Element-wise Operations

Operations that apply independently to each element:

```fusion
let a = Tensor::from_vec(vec![1.0, 2.0, 3.0, 4.0])
let b = Tensor::from_vec(vec![2.0, 2.0, 2.0, 2.0])

// Basic arithmetic
let sum = &a + &b        // [3.0, 4.0, 5.0, 6.0]
let diff = &a - &b       // [-1.0, 0.0, 1.0, 2.0]
let prod = &a * &b       // [2.0, 4.0, 6.0, 8.0]
let quot = &a / &b       // [0.5, 1.0, 1.5, 2.0]

// Scalar operations
let scaled = &a * 2.0    // [2.0, 4.0, 6.0, 8.0]
let offset = &a + 10.0   // [11.0, 12.0, 13.0, 14.0]

// Mathematical functions
let squared = a.pow(2.0)  // [1.0, 4.0, 9.0, 16.0]
let sqrt = a.sqrt()       // [1.0, 1.414, 1.732, 2.0]
let exp = a.exp()         // [2.718, 7.389, 20.086, 54.598]
let log = a.log()         // [0.0, 0.693, 1.099, 1.386]
let sin = a.sin()
let cos = a.cos()
let tanh = a.tanh()
let abs = a.abs()
let neg = a.neg()         // Negate all elements
```text

### 16.2.2 Matrix Operations

Operations for 2D tensors:

```fusion
let a = Tensor::<f32>::rand([3, 4])  // 3×4 matrix
let b = Tensor::<f32>::rand([4, 5])  // 4×5 matrix

// Matrix multiplication
let c = a.matmul(&b)  // Result: 3×5 matrix
println("Result shape: {:?}", c.shape())  // [3, 5]

// Batch matrix multiplication (for batched data)
let batch_a = Tensor::<f32>::rand([32, 3, 4])  // 32 matrices, each 3×4
let batch_b = Tensor::<f32>::rand([32, 4, 5])  // 32 matrices, each 4×5
let batch_c = batch_a.bmm(&batch_b)            // 32 matrices, each 3×5

// Transpose
let transposed = a.t()           // 4×3 matrix
let transposed = a.transpose(0, 1)  // Explicit dimension swap

// Matrix properties
let det = a.det()                // Determinant (square matrices)
let inv = a.inverse()            // Matrix inverse
let trace = a.trace()            // Sum of diagonal elements

// Decompositions
let (u, s, v) = a.svd()          // Singular Value Decomposition
let (q, r) = a.qr()              // QR decomposition
let eigenvalues = a.eigenvalues() // Eigenvalue decomposition
```text

### 16.2.3 Reduction Operations

Reduce tensor dimensions by aggregating values:

```fusion
let t = Tensor::<f32>::rand([3, 4, 5])

// Global reductions (reduce to scalar)
let sum = t.sum()               // Sum of all elements
let mean = t.mean()             // Mean of all elements
let max = t.max()               // Maximum element
let min = t.min()               // Minimum element
let prod = t.prod()             // Product of all elements
let std = t.std()               // Standard deviation
let var = t.variance()          // Variance

// Dimension-specific reductions
let row_sum = t.sum_dim(1)      // Sum along dimension 1
let col_mean = t.mean_dim(0)    // Mean along dimension 0
let max_vals, max_indices = t.max_dim(2)  // Max with indices

// Keep dimensions (useful for broadcasting)
let row_sum = t.sum_dim(1, keep_dim: true)  // Shape: [3, 1, 5]
```text

### 16.2.4 Shape Manipulation

```fusion
let t = Tensor::<f32>::rand([2, 3, 4])

// Reshape (same total elements, different dimensions)
let reshaped = t.reshape([6, 4])      // [2*3, 4] = [6, 4]
let reshaped = t.reshape([24])        // Flatten to 1D
let reshaped = t.reshape([-1, 4])     // -1 infers dimension: [6, 4]

// Flatten
let flat = t.flatten()                // [24]
let flat = t.flatten(start_dim: 0, end_dim: 1)  // [6, 4]

// Add/remove dimensions
let expanded = t.unsqueeze(0)         // Add dim at position 0: [1, 2, 3, 4]
let squeezed = expanded.squeeze(0)    // Remove dim of size 1: [2, 3, 4]

// Concatenate tensors
let a = Tensor::<f32>::rand([2, 3])
let b = Tensor::<f32>::rand([2, 3])
let cat = Tensor::cat([&a, &b], dim: 0)  // [4, 3]
let cat = Tensor::cat([&a, &b], dim: 1)  // [2, 6]

// Stack tensors (adds new dimension)
let stacked = Tensor::stack([&a, &b], dim: 0)  // [2, 2, 3]

// Split tensors
let chunks = t.chunk(3, dim: 0)       // Split into 3 parts along dim 0
let parts = t.split([1, 1, 1], dim: 0) // Explicit sizes

// Permute dimensions
let permuted = t.permute([2, 0, 1])   // Reorder dimensions
```text

### 16.2.5 Broadcasting

Broadcasting automatically expands tensors to compatible shapes:

```fusion
let a = Tensor::<f32>::rand([3, 4])    // 3×4
let b = Tensor::<f32>::rand([4])       // 4

// b is broadcast to [3, 4] by repeating rows
let c = &a + &b  // Result: [3, 4]

// Broadcasting rules:
// 1. Align shapes from the right
// 2. Dimensions must be equal or one must be 1
// 3. Dimensions of size 1 are broadcast

let x = Tensor::<f32>::rand([1, 3, 1])
let y = Tensor::<f32>::rand([2, 1, 4])
let z = &x + &y  // Result: [2, 3, 4]
```text

---

## 16.3 GPU Acceleration

For serious machine learning, GPUs are essential. Fusion makes GPU computing seamless.

### 16.3.1 Device Management

```fusion
use fusion_ai_core::Device

fn main() {
    // Check CUDA availability
    if Device::cuda_is_available() {
        println("CUDA available!")
        println("GPU count: {}", Device::cuda_device_count())

        // Get device properties
        let props = Device::cuda_get_device_properties(0)
        println("GPU 0: {}", props.name)
        println("Memory: {} GB", props.total_memory / 1_000_000_000)
    } else {
        println("Running on CPU")
    }

    // Device types
    let cpu = Device::CPU
    let gpu0 = Device::CUDA(0)
    let gpu1 = Device::CUDA(1)
}
```text

### 16.3.2 Moving Tensors to GPU

```fusion
// Create tensor on CPU
let cpu_tensor = Tensor::<f32>::rand([1000, 1000])
println("Device: {:?}", cpu_tensor.device())  // CPU

// Move to GPU
let gpu_tensor = cpu_tensor.to(Device::CUDA(0))
println("Device: {:?}", gpu_tensor.device())  // CUDA(0)

// Create tensor directly on GPU
let gpu_tensor = Tensor::<f32>::rand([1000, 1000])
    .to(Device::CUDA(0))

// Operations on GPU tensors stay on GPU
let a = Tensor::<f32>::rand([1000, 1000]).to(Device::CUDA(0))
let b = Tensor::<f32>::rand([1000, 1000]).to(Device::CUDA(0))
let c = a.matmul(&b)  // Computed on GPU; result stays on GPU

// Move back to CPU (for saving, printing, etc.)
let cpu_result = c.to(Device::CPU)
```text

### 16.3.3 Performance Considerations

**Memory Management:**

```fusion
// GPU memory is limited; be mindful of allocations
{
    let large_tensor = Tensor::<f32>::rand([10000, 10000]).to(Device::CUDA(0))
    // Use large_tensor...
}  // Tensor dropped; GPU memory freed

// Explicit memory clearing
Device::cuda_empty_cache()
```text

**Asynchronous Execution:**

```fusion
// GPU operations are asynchronous by default
let a = Tensor::<f32>::rand([1000, 1000]).to(Device::CUDA(0))
let b = a.matmul(&a)  // Queued, may not be complete yet

// Synchronise when needed
Device::cuda_synchronize()  // Wait for all GPU operations
```text

**Best Practices:**
1. **Batch operations**: GPU efficiency increases with larger batch sizes
2. **Minimise transfers**: CPU↔GPU transfers are expensive; keep data on GPU
3. **Use appropriate precision**: `f16` or `bf16` can double throughput
4. **Profile first**: Use `fusion profile --gpu` to identify bottlenecks

---

## 16.4 Automatic Differentiation

Training neural networks requires computing gradients. Fusion's **autograd** engine automatically differentiates tensor operations.

### 16.4.1 Enabling Gradient Tracking

```fusion
// By default, tensors don't track gradients
let x = Tensor::from_vec(vec![1.0, 2.0, 3.0])
println("Requires grad: {}", x.requires_grad())  // false

// Enable gradient tracking
let x = Tensor::from_vec(vec![1.0, 2.0, 3.0])
    .requires_grad_(true)
println("Requires grad: {}", x.requires_grad())  // true

// Alternative: set during creation
let x = Tensor::from_vec_with_grad(vec![1.0, 2.0, 3.0], true)
```text

### 16.4.2 Computing Gradients

```fusion
// Forward computation
let x = Tensor::from_vec(vec![2.0, 3.0])
    .requires_grad_(true)

let y = &x * &x         // y = x²
let z = y.sum()         // z = Σ(x²)

// Backward pass
z.backward()

// Access gradients
println("x.grad: {:?}", x.grad())  // [4.0, 6.0] = 2x
```text

### 16.4.3 The Computation Graph

Fusion builds a **computation graph** as you perform operations:

```text
   x (requires_grad=true)
   │
   ├─────┐
   │     │
   ▼     ▼
  mul (x * x)
   │
   ▼
  sum
   │
   ▼
   z
```text

When you call `z.backward()`, Fusion traverses this graph in reverse, applying the chain rule to compute gradients.

### 16.4.4 Gradient Control

```fusion
// Temporarily disable gradient tracking (for inference)
with no_grad() {
    let output = model.forward(input)
    // No gradients computed here
}

// Detach from graph (create tensor without gradient history)
let detached = tensor.detach()

// Clear gradients before next backward pass
x.grad().zero_()

// In-place gradient zeroing on optimizer
optimizer.zero_grad()
```text

### 16.4.5 Higher-Order Gradients

For some applications (meta-learning, physics-informed neural networks), you need second derivatives:

```fusion
let x = Tensor::from_vec(vec![2.0])
    .requires_grad_(true)

let y = x.pow(3.0)  // y = x³
let dy_dx = grad(y, x, create_graph: true)  // dy/dx = 3x²

let d2y_dx2 = grad(dy_dx.sum(), x)  // d²y/dx² = 6x
println("Second derivative: {:?}", d2y_dx2)  // [12.0]
```text

---

## 16.5 Building Neural Networks

The `nn` module provides layers, activations, and utilities for building neural networks.

### 16.5.1 The Module Trait

All neural network components implement the `Module` trait:

```fusion
trait Module {
    fn forward(&self, input: &Tensor) -> Tensor
    fn parameters(&self) -> Vec<&Tensor>
    fn train(&mut self)
    fn eval(&mut self)
}
```text

### 16.5.2 Common Layers

**Dense (Fully Connected):**

```fusion
use fusion_ai_core::nn::Dense

let layer = Dense::new(
    in_features: 784,
    out_features: 256,
    bias: true  // Include bias term (default: true)
)

let input = Tensor::<f32>::rand([32, 784])   // Batch of 32
let output = layer.forward(&input)            // Shape: [32, 256]
```text

**Convolutional:**

```fusion
use fusion_ai_core::nn::Conv2d

let conv = Conv2d::new(
    in_channels: 3,
    out_channels: 64,
    kernel_size: (3, 3),
    stride: (1, 1),
    padding: (1, 1)
)

let images = Tensor::<f32>::rand([32, 3, 224, 224])  // Batch of 32 images
let features = conv.forward(&images)  // Shape: [32, 64, 224, 224]
```text

**Recurrent (LSTM):**

```fusion
use fusion_ai_core::nn::LSTM

let lstm = LSTM::new(
    input_size: 512,
    hidden_size: 256,
    num_layers: 2,
    dropout: 0.1,
    bidirectional: false
)

let sequence = Tensor::<f32>::rand([32, 100, 512])  // [batch, seq_len, features]
let (output, (h_n, c_n)) = lstm.forward(&sequence)
// output: [32, 100, 256]
// h_n: [2, 32, 256] (hidden state for each layer)
```text

**Transformer:**

```fusion
use fusion_ai_core::nn::TransformerEncoder

let encoder = TransformerEncoder::new(
    d_model: 512,
    nhead: 8,
    num_layers: 6,
    dim_feedforward: 2048,
    dropout: 0.1
)

let tokens = Tensor::<f32>::rand([32, 100, 512])  // [batch, seq_len, d_model]
let encoded = encoder.forward(&tokens)  // [32, 100, 512]
```text

### 16.5.3 Activation Functions

```fusion
use fusion_ai_core::nn::functional as F

// ReLU: max(0, x)
let activated = F::relu(&x)

// Sigmoid: 1 / (1 + e^(-x))
let activated = F::sigmoid(&x)

// Tanh
let activated = F::tanh(&x)

// Softmax (along dimension)
let probs = F::softmax(&logits, dim: -1)

// GELU (used in transformers)
let activated = F::gelu(&x)

// Leaky ReLU
let activated = F::leaky_relu(&x, negative_slope: 0.01)
```text

### 16.5.4 Building a Custom Model

```fusion
use fusion_ai_core::{Tensor, Device}
use fusion_ai_core::nn::{Module, Dense, Conv2d, BatchNorm2d, Dropout}
use fusion_ai_core::nn::functional as F

class ImageClassifier {
    conv1: Conv2d
    bn1: BatchNorm2d
    conv2: Conv2d
    bn2: BatchNorm2d
    fc1: Dense
    fc2: Dense
    dropout: Dropout
}

impl ImageClassifier {
    fn new(num_classes: int) -> ImageClassifier {
        ImageClassifier {
            conv1: Conv2d::new(3, 32, (3, 3), padding: (1, 1)),
            bn1: BatchNorm2d::new(32),
            conv2: Conv2d::new(32, 64, (3, 3), padding: (1, 1)),
            bn2: BatchNorm2d::new(64),
            fc1: Dense::new(64 * 8 * 8, 256),
            fc2: Dense::new(256, num_classes),
            dropout: Dropout::new(0.5),
        }
    }
}

impl Module for ImageClassifier {
    fn forward(&self, x: &Tensor) -> Tensor {
        // x: [batch, 3, 32, 32]

        // Conv block 1
        let x = self.conv1.forward(x)
        let x = self.bn1.forward(&x)
        let x = F::relu(&x)
        let x = F::max_pool2d(&x, (2, 2))  // [batch, 32, 16, 16]

        // Conv block 2
        let x = self.conv2.forward(&x)
        let x = self.bn2.forward(&x)
        let x = F::relu(&x)
        let x = F::max_pool2d(&x, (2, 2))  // [batch, 64, 8, 8]

        // Flatten
        let x = x.flatten(start_dim: 1)  // [batch, 64*8*8]

        // Fully connected
        let x = self.fc1.forward(&x)
        let x = F::relu(&x)
        let x = self.dropout.forward(&x)
        let x = self.fc2.forward(&x)

        x  // Logits (apply softmax during loss or inference)
    }

    fn parameters(&self) -> Vec<&Tensor> {
        vec![
            self.conv1.weight(), self.conv1.bias(),
            self.bn1.weight(), self.bn1.bias(),
            self.conv2.weight(), self.conv2.bias(),
            self.bn2.weight(), self.bn2.bias(),
            self.fc1.weight(), self.fc1.bias(),
            self.fc2.weight(), self.fc2.bias(),
        ].into_iter().flatten().collect()
    }
}
```text

---

## 16.6 Training Neural Networks

### 16.6.1 Loss Functions

```fusion
use fusion_ai_core::nn::loss

// Cross-entropy (for classification)
let loss = loss::cross_entropy(&predictions, &targets)

// Mean squared error (for regression)
let loss = loss::mse_loss(&predictions, &targets)

// Binary cross-entropy (for binary classification)
let loss = loss::binary_cross_entropy(&predictions, &targets)

// Custom loss (just compute with tensors)
fn custom_loss(pred: &Tensor, target: &Tensor) -> Tensor {
    let diff = pred - target
    (&diff * &diff).mean()  // MSE manually
}
```text

### 16.6.2 Optimisers

```fusion
use fusion_ai_core::optim::{SGD, Adam, AdamW}

// Stochastic Gradient Descent
let sgd = SGD::new(model.parameters(), lr: 0.01, momentum: 0.9)

// Adam (recommended default)
let adam = Adam::new(
    model.parameters(),
    lr: 0.001,
    betas: (0.9, 0.999),
    eps: 1e-8
)

// AdamW with weight decay (for transformers)
let adamw = AdamW::new(
    model.parameters(),
    lr: 0.001,
    weight_decay: 0.01
)
```text

### 16.6.3 Complete Training Loop

```fusion
fn train(
    model: &mut ImageClassifier,
    train_loader: DataLoader,
    epochs: int,
    device: Device,
) {
    model.train()  // Set to training mode (enables dropout, etc.)
    let mut optimizer = Adam::new(model.parameters(), lr: 0.001)

    for epoch in 0..epochs {
        let mut total_loss = 0.0
        let mut correct = 0
        let mut total = 0

        for (images, labels) in train_loader.iter() {
            // Move to device
            let images = images.to(device)
            let labels = labels.to(device)

            // Forward pass
            let outputs = model.forward(&images)
            let loss = F::cross_entropy(&outputs, &labels)

            // Backward pass
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()

            // Track metrics
            total_loss += loss.item()
            let predicted = outputs.argmax(dim: 1)
            correct += predicted.eq(&labels).sum().item() as int
            total += labels.numel()
        }

        let avg_loss = total_loss / train_loader.len() as f64
        let accuracy = 100.0 * (correct as f64) / (total as f64)
        println("Epoch {}: Loss = {:.4}, Accuracy = {:.2}%", epoch, avg_loss, accuracy)
    }
}
```text

### 16.6.4 Validation and Testing

```fusion
fn evaluate(
    model: &ImageClassifier,
    test_loader: DataLoader,
    device: Device,
) -> f64 {
    model.eval()  // Disable dropout, use running stats for batchnorm

    let mut correct = 0
    let mut total = 0

    with no_grad() {  // No gradient computation during evaluation
        for (images, labels) in test_loader.iter() {
            let images = images.to(device)
            let labels = labels.to(device)

            let outputs = model.forward(&images)
            let predicted = outputs.argmax(dim: 1)

            correct += predicted.eq(&labels).sum().item() as int
            total += labels.numel()
        }
    }

    100.0 * (correct as f64) / (total as f64)
}
```text

---

## 16.7 Advanced Topics

### 16.7.1 Learning Rate Scheduling

```fusion
use fusion_ai_core::optim::lr_scheduler::*

// Step decay
let scheduler = StepLR::new(&optimizer, step_size: 30, gamma: 0.1)

// Cosine annealing
let scheduler = CosineAnnealingLR::new(&optimizer, T_max: 100)

// Learning rate warmup
let scheduler = WarmupLR::new(&optimizer, warmup_steps: 1000)

// Usage in training loop
for epoch in 0..epochs {
    // ... training ...
    scheduler.step()  // Update learning rate
}
```text

### 16.7.2 Gradient Clipping

Prevent exploding gradients:

```fusion
// Clip by norm
nn::utils::clip_grad_norm_(model.parameters(), max_norm: 1.0)

// Clip by value
nn::utils::clip_grad_value_(model.parameters(), clip_value: 0.5)
```text

### 16.7.3 Model Checkpointing

```fusion
// Save model
let state_dict = model.state_dict()
fusion_ai_core::save(&state_dict, "model.pt")

// Load model
let state_dict = fusion_ai_core::load("model.pt")
model.load_state_dict(&state_dict)
```text

---

## 16.8 Best Practices for Production ML

### 16.8.1 Reproducibility

```fusion
// Set seeds for reproducibility
fusion_ai_core::manual_seed(42)
if Device::cuda_is_available() {
    fusion_ai_core::cuda::manual_seed_all(42)
}
```text

### 16.8.2 Memory Efficiency

```fusion
// Gradient checkpointing for large models
use fusion_ai_core::utils::checkpoint

let output = checkpoint::checkpoint(expensive_layer, input)
// Trades compute for memory by recomputing in backward pass

// Mixed precision training
use fusion_ai_core::cuda::amp

let scaler = amp::GradScaler::new()
with amp::autocast() {
    let output = model.forward(&input)
    let loss = loss_fn(&output, &target)
}
scaler.scale(loss).backward()
scaler.step(&optimizer)
scaler.update()
```text

### 16.8.3 Profiling

```fusion
// Profile GPU operations
with fusion_ai_core::profiler::profile() as prof {
    model.forward(&input)
}
println("{}", prof.key_averages().table())
```text

---

## 16.9 Summary

This chapter covered Fusion's AI/ML capabilities:

| Feature                       | Description                                         |
| :---------------------------- | :-------------------------------------------------- |
| **Tensor**                    | Multi-dimensional array with GPU support            |
| **Operations**                | Element-wise, matrix, reduction, shape manipulation |
| **Automatic Differentiation** | Gradient computation via computation graphs         |
| **Layers**                    | Dense, Conv, LSTM, Transformer                      |
| **Training**                  | Optimisers, loss functions, training loops          |
| **GPU Acceleration**          | Seamless CPU ↔ GPU transfers                        |

Key takeaways:

1. Tensors are first-class types in Fusion, not library additions
2. GPU acceleration is transparent—same code runs on CPU or GPU
3. Automatic differentiation eliminates manual gradient derivation
4. The module system provides composable, type-safe neural networks
5. Training utilities (optimisers, schedulers, checkpointing) are built-in

Fusion's ML capabilities integrate naturally with the rest of the language—quantum circuits can use tensor outputs, classical code can orchestrate ML inference, and everything benefits from Fusion's memory safety.

---

## 16.10 Exercises

1. **Matrix Operations**: Create two random matrices and compute their product, sum, and element-wise product.

2. **Simple Autograd**: Implement the gradient computation for `z = 3x² + 2x + 1` at `x = 2` using automatic differentiation.

3. **MLP from Scratch**: Build a multi-layer perceptron with two hidden layers and train it on a simple classification task.

4. **GPU Benchmark**: Compare the time to multiply two 4096×4096 matrices on CPU vs GPU.

5. **Custom Layer**: Implement a custom attention layer and verify gradients flow through it correctly.

---

[Next: Chapter 17 - Quantum Computing →](./chapter-17-quantum.md)