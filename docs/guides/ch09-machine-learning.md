# Chapter 9: Machine Learning

> Tensor operations, automatic differentiation, neural networks, and GPU support

---

## Tensor Operations

Tensors are the fundamental data structure for machine learning. Fusion provides native tensor support.

### Creating Tensors

```fusion
use std::ml;

fn main() -> int {
    // Scalar tensor
    let scalar: ml::Tensor = ml::tensor(5.0);
    println("Scalar shape: %s", scalar.shape().to_string());

    // 1D tensor (vector)
    let vector: ml::Tensor = ml::tensor([1.0, 2.0, 3.0, 4.0]);
    println("Vector shape: %s", vector.shape().to_string());

    // 2D tensor (matrix)
    let matrix: ml::Tensor = ml::tensor([
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
    ]);
    println("Matrix shape: %s", matrix.shape().to_string());

    // 3D tensor (e.g., batch of images: batch x height x width)
    let tensor3d: ml::Tensor = ml::zeros([2, 3, 4]);
    println("3D tensor shape: %s", tensor3d.shape().to_string());

    // 4D tensor (e.g., batch of color images: batch x channels x height x width)
    let tensor4d: ml::Tensor = ml::zeros([32, 3, 224, 224]);
    println("4D tensor shape: %s", tensor4d.shape().to_string());

    // Random tensors
    let rand_normal: ml::Tensor = ml::randn([3, 4]);   // Normal distribution
    let rand_uniform: ml::Tensor = ml::rand([3, 4]);    // Uniform [0, 1)
    let rand_int: ml::Tensor = ml::randint(0, 10, [5]); // Random integers

    return 0;
}
```

### Tensor Operations

```fusion
use std::ml;

fn main() -> int {
    let a: ml::Tensor = ml::tensor([1.0, 2.0, 3.0]);
    let b: ml::Tensor = ml::tensor([4.0, 5.0, 6.0]);

    // Element-wise operations
    let sum: ml::Tensor = a + b;
    let diff: ml::Tensor = a - b;
    let prod: ml::Tensor = a * b;
    let div: ml::Tensor = a / b;

    println("Sum: %s", sum.to_string());
    println("Diff: %s", diff.to_string());
    println("Product: %s", prod.to_string());

    // Matrix multiplication
    let m1: ml::Tensor = ml::tensor([[1.0, 2.0], [3.0, 4.0]]);
    let m2: ml::Tensor = ml::tensor([[5.0, 6.0], [7.0, 8.0]]);
    let matmul: ml::Tensor = m1 @ m2;  // Matrix multiply
    println("Matmul: %s", matmul.to_string());

    // Batch matrix multiplication
    let batch1: ml::Tensor = ml::randn([8, 4, 3]);  // batch of 8, 4x3 matrices
    let batch2: ml::Tensor = ml::randn([8, 3, 5]);  // batch of 8, 3x5 matrices
    let batch_matmul: ml::Tensor = ml::bmm(batch1, batch2);  // Result: [8, 4, 5]
    println("Batch matmul shape: %s", batch_matmul.shape().to_string());

    // Reduction operations
    let total: float = a.sum();
    let mean: float = a.mean();
    let max_val: float = a.max();
    let min_val: float = a.min();

    println("Sum=%f, Mean=%f, Max=%f, Min=%f", total, mean, max_val, min_val);

    // Dimension-wise reductions
    let m: ml::Tensor = ml::tensor([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    let row_sum: ml::Tensor = m.sum(dim=1);   // Sum along rows
    let col_mean: ml::Tensor = m.mean(dim=0); // Mean along columns
    println("Row sum: %s", row_sum.to_string());
    println("Col mean: %s", col_mean.to_string());

    return 0;
}
```

### Tensor Manipulation

```fusion
use std::ml;

fn main() -> int {
    let t: ml::Tensor = ml::tensor([
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
    ]);

    // Reshape
    let reshaped: ml::Tensor = t.reshape([3, 2]);
    println("Reshaped: %s", reshaped.to_string());

    // Transpose
    let transposed: ml::Tensor = t.transpose();
    println("Transposed: %s", transposed.to_string());

    // Slice
    let sliced: ml::Tensor = t[0..1, 1..3];
    println("Sliced: %s", sliced.to_string());

    // Concatenate
    let a: ml::Tensor = ml::tensor([1.0, 2.0]);
    let b: ml::Tensor = ml::tensor([3.0, 4.0]);
    let concatenated: ml::Tensor = ml::cat([a, b], 0);
    println("Concatenated: %s", concatenated.to_string());

    // Stack
    let stacked: ml::Tensor = ml::stack([a, b], dim=0);
    println("Stacked shape: %s", stacked.shape().to_string());

    // Permute (for reordering dimensions, e.g., in image processing)
    let img: ml::Tensor = ml::randn([3, 224, 224]);  // C, H, W
    let img_nchw: ml::Tensor = img.permute([2, 0, 1]); // H, W, C
    println("Permuted shape: %s", img_nchw.shape().to_string());

    return 0;
}
```

---

## Automatic Differentiation

Fusion provides automatic differentiation for computing gradients.

### Basic Autograd

```fusion
use std::ml;

fn main() -> int {
    // Create a tensor with gradient tracking
    let x: ml::Tensor = ml::tensor(3.0).requires_grad(true);

    // Compute a function
    let y: ml::Tensor = x * x + 2 * x + 1;

    // Backpropagate
    y.backward();

    // Get gradient
    let grad: float = x.grad();
    println("dy/dx at x=3: %f", grad);  // Should be 8.0 (2*3 + 2)

    return 0;
}
```

### Gradient Computation

```fusion
use std::ml;

fn main() -> int {
    let x: ml::Tensor = ml::tensor(2.0).requires_grad(true);

    // Compute f(x) = x^3 + 2x^2 - 5x + 3
    let y: ml::Tensor = x.pow(3) + 2 * x.pow(2) - 5 * x + 3;

    // Compute gradient
    y.backward();

    // f'(x) = 3x^2 + 4x - 5
    // f'(2) = 3*4 + 4*2 - 5 = 12 + 8 - 5 = 15
    println("f'(2) = %f", x.grad());  // Should be 15.0

    return 0;
}
```

### Multi-Variable Gradients

```fusion
use std::ml;

fn main() -> int {
    let x: ml::Tensor = ml::tensor(1.0).requires_grad(true);
    let y: ml::Tensor = ml::tensor(2.0).requires_grad(true);

    // f(x, y) = x^2 + y^2 + 2xy
    let z: ml::Tensor = x.pow(2) + y.pow(2) + 2 * x * y;

    z.backward();

    // ∂f/∂x = 2x + 2y = 2(1) + 2(2) = 6
    // ∂f/∂y = 2y + 2x = 2(2) + 2(1) = 6
    println("∂f/∂x = %f", x.grad());  // 6.0
    println("∂f/∂y = %f", y.grad());  // 6.0

    return 0;
}
```

---

## Neural Network Layers

### Linear Layer

```fusion
use std::ml;

struct Linear {
    weight: ml::Tensor,
    bias: ml::Tensor,
}

impl Linear {
    fn new(in_features: int, out_features: int) -> Linear {
        // Kaiming uniform initialization
        let weight: ml::Tensor = ml::kaiming_uniform([out_features, in_features]);
        let bias: ml::Tensor = ml::zeros([out_features]);
        return Linear { weight, bias };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        return x @ self.weight + self.bias;
    }
}

fn main() -> int {
    let layer: Linear = Linear::new(3, 2);
    let input: ml::Tensor = ml::tensor([1.0, 2.0, 3.0]);
    let output: ml::Tensor = layer.forward(input);
    println("Linear output: %s", output.to_string());
    return 0;
}
```

### Convolutional Layer (Conv2d)

```fusion
use std::ml;

struct Conv2d {
    weight: ml::Tensor,
    bias: ml::Tensor,
    stride: int,
    padding: int,
}

impl Conv2d {
    fn new(in_channels: int, out_channels: int, kernel_size: int, stride: int, padding: int) -> Conv2d {
        let fan_in: int = in_channels * kernel_size * kernel_size;
        let weight: ml::Tensor = ml::kaiming_normal([out_channels, in_channels, kernel_size, kernel_size])
                                  / (fan_in as float).sqrt();
        let bias: ml::Tensor = ml::zeros([out_channels]);
        return Conv2d { weight, bias, stride, padding };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        return ml::conv2d(x, self.weight, self.bias, self.stride, self.padding);
    }
}

fn main() -> int {
    // Input: batch=1, channels=3, height=32, width=32
    let input: ml::Tensor = ml::randn([1, 3, 32, 32]);

    // Conv layer: 3 input channels, 16 output channels, 3x3 kernel
    let conv: Conv2d = Conv2d::new(3, 16, 3, 1, 1);
    let output: ml::Tensor = conv.forward(input);
    println("Conv2d output shape: %s", output.shape().to_string());
    // Output: [1, 16, 32, 32]

    return 0;
}
```

### MaxPool2d Layer

```fusion
use std::ml;

struct MaxPool2d {
    kernel_size: int,
    stride: int,
}

impl MaxPool2d {
    fn new(kernel_size: int, stride: int) -> MaxPool2d {
        return MaxPool2d { kernel_size, stride };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        return ml::max_pool2d(x, self.kernel_size, self.stride);
    }
}

fn main() -> int {
    let input: ml::Tensor = ml::randn([1, 16, 32, 32]);
    let pool: MaxPool2d = MaxPool2d::new(2, 2);
    let output: ml::Tensor = pool.forward(input);
    println("MaxPool2d output shape: %s", output.shape().to_string());
    // Output: [1, 16, 16, 16]

    return 0;
}
```

### LSTM Layer

```fusion
use std::ml;

struct LSTM {
    input_size: int,
    hidden_size: int,
    weight_ih: ml::Tensor,  // Input-to-hidden weights
    weight_hh: ml::Tensor,  // Hidden-to-hidden weights
    bias_ih: ml::Tensor,
    bias_hh: ml::Tensor,
}

impl LSTM {
    fn new(input_size: int, hidden_size: int) -> LSTM {
        return LSTM {
            input_size,
            hidden_size,
            weight_ih: ml::xavier_uniform([4 * hidden_size, input_size]),
            weight_hh: ml::xavier_uniform([4 * hidden_size, hidden_size]),
            bias_ih: ml::zeros([4 * hidden_size]),
            bias_hh: ml::zeros([4 * hidden_size]),
        };
    }

    fn forward(self, x: ml::Tensor, state: Option<(ml::Tensor, ml::Tensor)>) -> (ml::Tensor, ml::Tensor, ml::Tensor) {
        // x shape: [batch, seq_len, input_size]
        let batch_size: int = x.shape()[0];
        let seq_len: int = x.shape()[1];

        let mut h: ml::Tensor;
        let mut c: ml::Tensor;

        match state {
            Some((h_prev, c_prev)) => {
                h = h_prev;
                c = c_prev;
            }
            None => {
                h = ml::zeros([batch_size, self.hidden_size]);
                c = ml::zeros([batch_size, self.hidden_size]);
            }
        }

        let mut outputs: [ml::Tensor] = [];

        for t in 0..seq_len {
            let x_t: ml::Tensor = x[:, t, :];  // [batch, input_size]

            // Combined gates: i, f, g, o
            let gates: ml::Tensor = (x_t @ self.weight_ih + self.bias_ih)
                                   + (h @ self.weight_hh + self.bias_hh);

            // Split into four gates
            let i: ml::Tensor = ml::sigmoid(gates[:, 0..self.hidden_size]);              // Input gate
            let f: ml::Tensor = ml::sigmoid(gates[:, self.hidden_size..2*self.hidden_size]); // Forget gate
            let g: ml::Tensor = ml::tanh(gates[:, 2*self.hidden_size..3*self.hidden_size]);  // Cell gate
            let o: ml::Tensor = ml::sigmoid(gates[:, 3*self.hidden_size..]);              // Output gate

            // Update cell state and hidden state
            c = f * c + i * g;
            h = o * ml::tanh(c);

            outputs.push(h);
        }

        let output: ml::Tensor = ml::stack(outputs, dim=1);
        return (output, h, c);
    }
}

fn main() -> int {
    let lstm: LSTM = LSTM::new(10, 20);  // input_size=10, hidden_size=20
    let input: ml::Tensor = ml::randn([4, 5, 10]);  // batch=4, seq_len=5, features=10

    let (output, h_n, c_n): (ml::Tensor, ml::Tensor, ml::Tensor) = lstm.forward(input, None);
    println("LSTM output shape: %s", output.shape().to_string());   // [4, 5, 20]
    println("Hidden state shape: %s", h_n.shape().to_string());     // [4, 20]
    println("Cell state shape: %s", c_n.shape().to_string());       // [4, 20]

    return 0;
}
```

### Transformer Layer

```fusion
use std::ml;

struct MultiHeadAttention {
    num_heads: int,
    head_dim: int,
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    out_proj: Linear,
}

impl MultiHeadAttention {
    fn new(embed_dim: int, num_heads: int) -> MultiHeadAttention {
        let head_dim: int = embed_dim / num_heads;
        return MultiHeadAttention {
            num_heads,
            head_dim,
            q_proj: Linear::new(embed_dim, embed_dim),
            k_proj: Linear::new(embed_dim, embed_dim),
            v_proj: Linear::new(embed_dim, embed_dim),
            out_proj: Linear::new(embed_dim, embed_dim),
        };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        let batch: int = x.shape()[0];
        let seq_len: int = x.shape()[1];
        let embed_dim: int = x.shape()[2];

        // Project to Q, K, V
        let q: ml::Tensor = self.q_proj.forward(x);  // [batch, seq, embed]
        let k: ml::Tensor = self.k_proj.forward(x);
        let v: ml::Tensor = self.v_proj.forward(x);

        // Reshape to [batch, num_heads, seq_len, head_dim]
        let q: ml::Tensor = q.reshape([batch, seq_len, self.num_heads, self.head_dim]).transpose(1, 2);
        let k: ml::Tensor = k.reshape([batch, seq_len, self.num_heads, self.head_dim]).transpose(1, 2);
        let v: ml::Tensor = v.reshape([batch, seq_len, self.num_heads, self.head_dim]).transpose(1, 2);

        // Scaled dot-product attention
        let scale: float = (self.head_dim as float).sqrt();
        let scores: ml::Tensor = (q @ k.transpose(-2, -1)) / scale;
        let attn_weights: ml::Tensor = ml::softmax(scores, dim=-1);
        let attn_output: ml::Tensor = attn_weights @ v;  // [batch, heads, seq, head_dim]

        // Reshape and project
        let output: ml::Tensor = attn_output.transpose(1, 2)
                                         .reshape([batch, seq_len, embed_dim]);
        return self.out_proj.forward(output);
    }
}

struct TransformerBlock {
    attention: MultiHeadAttention,
    norm1: LayerNorm,
    ff1: Linear,
    ff2: Linear,
    norm2: LayerNorm,
}

impl TransformerBlock {
    fn new(embed_dim: int, num_heads: int, ff_dim: int) -> TransformerBlock {
        return TransformerBlock {
            attention: MultiHeadAttention::new(embed_dim, num_heads),
            norm1: LayerNorm::new(embed_dim),
            ff1: Linear::new(embed_dim, ff_dim),
            ff2: Linear::new(ff_dim, embed_dim),
            norm2: LayerNorm::new(embed_dim),
        };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        // Self-attention with residual connection
        let attn_out: ml::Tensor = self.attention.forward(self.norm1.forward(x));
        let x: ml::Tensor = x + attn_out;

        // Feed-forward with residual connection
        let ff_out: ml::Tensor = self.ff2.forward(ml::relu(self.ff1.forward(self.norm2.forward(x))));
        return x + ff_out;
    }
}

fn main() -> int {
    let embed_dim: int = 512;
    let num_heads: int = 8;
    let ff_dim: int = 2048;
    let batch: int = 2;
    let seq_len: int = 10;

    let block: TransformerBlock = TransformerBlock::new(embed_dim, num_heads, ff_dim);
    let input: ml::Tensor = ml::randn([batch, seq_len, embed_dim]);
    let output: ml::Tensor = block.forward(input);
    println("Transformer output shape: %s", output.shape().to_string());

    return 0;
}
```

---

## Activation Functions

```fusion
use std::ml;

fn main() -> int {
    let x: ml::Tensor = ml::tensor([-2.0, -1.0, 0.0, 1.0, 2.0]);

    // ReLU
    let relu_out: ml::Tensor = ml::relu(x);
    println("ReLU: %s", relu_out.to_string());

    // Leaky ReLU
    let leaky_out: ml::Tensor = ml::leaky_relu(x, 0.01);
    println("Leaky ReLU: %s", leaky_out.to_string());

    // GELU (used in Transformers)
    let gelu_out: ml::Tensor = ml::gelu(x);
    println("GELU: %s", gelu_out.to_string());

    // Sigmoid
    let sigmoid_out: ml::Tensor = ml::sigmoid(x);
    println("Sigmoid: %s", sigmoid_out.to_string());

    // Tanh
    let tanh_out: ml::Tensor = ml::tanh(x);
    println("Tanh: %s", tanh_out.to_string());

    // Softmax
    let logits: ml::Tensor = ml::tensor([1.0, 2.0, 3.0]);
    let softmax_out: ml::Tensor = ml::softmax(logits);
    println("Softmax: %s", softmax_out.to_string());

    // SiLU (Swish)
    let silu_out: ml::Tensor = ml::silu(x);
    println("SiLU: %s", silu_out.to_string());

    return 0;
}
```

---

## Training Loops

### Complete Training Example

```fusion
use std::ml;

fn main() -> int {
    // Create model
    let model: Network = Network::new();

    // Create optimizer
    let optimizer: ml::Adam = ml::Adam::new(model.parameters(), 0.001);

    // Create loss function
    let loss_fn: ml::CrossEntropyLoss = ml::CrossEntropyLoss::new();

    // Training loop
    for epoch in 0..100 {
        let mut total_loss: float = 0.0;

        for batch in train_data.batches(32) {
            // Forward pass
            let predictions: ml::Tensor = model.forward(batch.inputs);
            let loss: ml::Tensor = loss_fn.forward(predictions, batch.labels);

            // Backward pass
            optimizer.zero_grad();
            loss.backward();
            optimizer.step();

            total_loss = total_loss + loss.item();
        }

        if epoch %% 10 == 0 {
            println("Epoch %d: loss = %f", epoch, total_loss / train_data.num_batches());
        }
    }

    return 0;
}
```

### Loss Functions

```fusion
use std::ml;

fn main() -> int {
    let predictions: ml::Tensor = ml::tensor([0.9, 0.1, 0.0]);
    let targets: ml::Tensor = ml::tensor([1.0, 0.0, 0.0]);

    // Cross-entropy loss (for classification)
    let ce_loss: ml::Tensor = ml::cross_entropy(predictions, targets);
    println("Cross-entropy loss: %f", ce_loss.item());

    // Mean squared error (for regression)
    let mse_loss: ml::Tensor = ml::mse(predictions, targets);
    println("MSE loss: %f", mse_loss.item());

    // Binary cross-entropy (for binary classification)
    let bce_loss: ml::Tensor = ml::binary_cross_entropy(predictions, targets);
    println("BCE loss: %f", bce_loss.item());

    // L1 loss (MAE)
    let l1_loss: ml::Tensor = ml::l1_loss(predictions, targets);
    println("L1 loss: %f", l1_loss.item());

    // Huber loss (smooth L1)
    let huber_loss: ml::Tensor = ml::huber_loss(predictions, targets, 1.0);
    println("Huber loss: %f", huber_loss.item());

    return 0;
}
```

### Learning Rate Scheduling

```fusion
use std::ml;

fn main() -> int {
    let model: Network = Network::new();
    let optimizer: ml::Adam = ml::Adam::new(model.parameters(), 0.001);

    // Step decay scheduler
    let scheduler: ml::StepLR = ml::StepLR::new(optimizer, step_size=10, gamma=0.5);

    // Cosine annealing scheduler
    let scheduler2: ml::CosineAnnealingLR = ml::CosineAnnealingLR::new(
        optimizer, T_max=100, eta_min=0.0001,
    );

    // One cycle policy
    let scheduler3: ml::OneCycleLR = ml::OneCycleLR::new(
        optimizer, max_lr=0.01, total_steps=1000,
    );

    for epoch in 0..100 {
        // Training step
        let loss: ml::Tensor = train_one_epoch(model, optimizer);
        scheduler.step();

        println("Epoch %d: lr=%f, loss=%f", epoch, scheduler.get_lr(), loss.item());
    }

    return 0;
}
```

### Gradient Clipping

```fusion
use std::ml;

fn main() -> int {
    let model: Network = Network::new();
    let optimizer: ml::Adam = ml::Adam::new(model.parameters(), 0.001);

    for batch in train_data.batches(32) {
        let loss: ml::Tensor = model.forward(batch.inputs);
        loss.backward();

        // Clip gradients by norm
        ml::clip_grad_norm(model.parameters(), max_norm=1.0);

        // Or clip by value
        // ml::clip_grad_value(model.parameters(), clip_value=0.5);

        optimizer.step();
        optimizer.zero_grad();
    }

    return 0;
}
```

---

## Model Zoo

### LeNet-5 (Classic CNN)

```fusion
use std::ml;

struct LeNet5 {
    conv1: Conv2d,
    conv2: Conv2d,
    fc1: Linear,
    fc2: Linear,
    fc3: Linear,
}

impl LeNet5 {
    fn new(num_classes: int) -> LeNet5 {
        return LeNet5 {
            conv1: Conv2d::new(1, 6, 5, 1, 2),     // 1→6 channels, 5x5 kernel
            conv2: Conv2d::new(6, 16, 5, 1, 0),    // 6→16 channels, 5x5 kernel
            fc1: Linear::new(16 * 5 * 5, 120),      // 16*5*5 → 120
            fc2: Linear::new(120, 84),               // 120 → 84
            fc3: Linear::new(84, num_classes),       // 84 → num_classes
        };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        // Conv1 → ReLU → MaxPool
        let x: ml::Tensor = ml::max_pool2d(ml::relu(self.conv1.forward(x)), 2, 2);
        // Conv2 → ReLU → MaxPool
        let x: ml::Tensor = ml::max_pool2d(ml::relu(self.conv2.forward(x)), 2, 2);

        // Flatten
        let x: ml::Tensor = x.flatten(1);

        // FC layers
        let x: ml::Tensor = ml::relu(self.fc1.forward(x));
        let x: ml::Tensor = ml::relu(self.fc2.forward(x));
        return self.fc3.forward(x);
    }
}

fn main() -> int {
    let model: LeNet5 = LeNet5::new(10);  // 10 classes for MNIST
    let input: ml::Tensor = ml::randn([1, 1, 28, 28]);  // MNIST image
    let output: ml::Tensor = model.forward(input);
    println("LeNet5 output shape: %s", output.shape().to_string());  // [1, 10]
    return 0;
}
```

### ResNet Block and ResNet-18

```fusion
use std::ml;

struct ResidualBlock {
    conv1: Conv2d,
    bn1: BatchNorm2d,
    conv2: Conv2d,
    bn2: BatchNorm2d,
    shortcut: Option<(Conv2d, BatchNorm2d)>,
}

impl ResidualBlock {
    fn new(in_channels: int, out_channels: int, stride: int) -> ResidualBlock {
        let shortcut: Option<(Conv2d, BatchNorm2d)> = if stride != 1 || in_channels != out_channels {
            Some((
                Conv2d::new(in_channels, out_channels, 1, stride, 0),
                BatchNorm2d::new(out_channels),
            ))
        } else {
            None
        };

        return ResidualBlock {
            conv1: Conv2d::new(in_channels, out_channels, 3, stride, 1),
            bn1: BatchNorm2d::new(out_channels),
            conv2: Conv2d::new(out_channels, out_channels, 3, 1, 1),
            bn2: BatchNorm2d::new(out_channels),
            shortcut,
        };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        let identity: ml::Tensor = x.clone();

        let out: ml::Tensor = ml::relu(self.bn1.forward(self.conv1.forward(x)));
        let out: ml::Tensor = self.bn2.forward(self.conv2.forward(out));

        let out: ml::Tensor = match self.shortcut {
            Some((conv, bn)) => out + bn.forward(conv.forward(identity)),
            None => out + identity,
        };

        return ml::relu(out);
    }
}

fn make_resnet_layer(in_channels: int, out_channels: int, num_blocks: int, stride: int) -> Vec<ResidualBlock> {
    let mut layers: Vec<ResidualBlock> = [];
    layers.push(ResidualBlock::new(in_channels, out_channels, stride));

    for _ in 1..num_blocks {
        layers.push(ResidualBlock::new(out_channels, out_channels, 1));
    }

    return layers;
}

fn main() -> int {
    // ResNet-18 structure
    let block1: Vec<ResidualBlock> = make_resnet_layer(64, 64, 2, 1);
    let block2: Vec<ResidualBlock> = make_resnet_layer(64, 128, 2, 2);
    let block3: Vec<ResidualBlock> = make_resnet_layer(128, 256, 2, 2);
    let block4: Vec<ResidualBlock> = make_resnet_layer(256, 512, 2, 2);

    println("ResNet-18 with %d residual blocks",
        block1.len() + block2.len() + block3.len() + block4.len());

    return 0;
}
```

### VGG Block

```fusion
use std::ml;

struct VGGBlock {
    layers: Vec<Conv2d>,
    pool: MaxPool2d,
}

impl VGGBlock {
    fn new(in_channels: int, out_channels: int, num_convs: int) -> VGGBlock {
        let mut layers: Vec<Conv2d> = [];
        let mut current_channels: int = in_channels;

        for _ in 0..num_convs {
            layers.push(Conv2d::new(current_channels, out_channels, 3, 1, 1));
            current_channels = out_channels;
        }

        return VGGBlock {
            layers,
            pool: MaxPool2d::new(2, 2),
        };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        let mut x: ml::Tensor = x;
        for conv in self.layers {
            x = ml::relu(conv.forward(x));
        }
        return self.pool.forward(x);
    }
}

fn main() -> int {
    // VGG-style blocks
    let block1: VGGBlock = VGGBlock::new(3, 64, 2);     // 2 conv layers
    let block2: VGGBlock = VGGBlock::new(64, 128, 2);   // 2 conv layers
    let block3: VGGBlock = VGGBlock::new(128, 256, 3);  // 3 conv layers
    let block4: VGGBlock = VGGBlock::new(256, 512, 3);  // 3 conv layers
    let block5: VGGBlock = VGGBlock::new(512, 512, 3);  // 3 conv layers

    let input: ml::Tensor = ml::randn([1, 3, 224, 224]);
    let x: ml::Tensor = block1.forward(input);
    let x: ml::Tensor = block2.forward(x);
    let x: ml::Tensor = block3.forward(x);
    let x: ml::Tensor = block4.forward(x);
    let x: ml::Tensor = block5.forward(x);
    println("VGG output shape: %s", x.shape().to_string());

    return 0;
}
```

---

## AI Provider Integration

Fusion integrates with local and cloud AI providers for inference and fine-tuning.

### Ollama (Local Models)

```fusion
use std::ai;

fn main() -> int {
    // Connect to local Ollama instance
    let client: ai::OllamaClient = ai::OllamaClient::new("http://localhost:11434");

    // List available models
    let models: Vec<string> = client.list_models();
    println("Available models: %s", models.join(", "));

    // Generate text
    let response: ai::ChatResponse = client.chat(ai::ChatRequest {
        model: "llama3.2",
        messages: [
            ai::Message { role: "system", content: "You are a helpful assistant." },
            ai::Message { role: "user", content: "Explain quantum computing in one sentence." },
        ],
        temperature: 0.7,
        max_tokens: 256,
    });

    println("Response: %s", response.content);

    return 0;
}
```

### Mistral (Cloud API)

```fusion
use std::ai;

fn main() -> int {
    let client: ai::MistralClient = ai::MistralClient::new(
        std::env::var("MISTRAL_API_KEY"),
    );

    // Text generation
    let response: ai::ChatResponse = client.chat(ai::ChatRequest {
        model: "mistral-large-latest",
        messages: [
            ai::Message { role: "user", content: "Write a haiku about machine learning." },
        ],
        temperature: 0.8,
        max_tokens: 100,
    });

    println("Generated: %s", response.content);

    // Embeddings
    let embedding: ml::Tensor = client.embed(
        "mistral-embed",
        "Machine learning is a branch of artificial intelligence.",
    );
    println("Embedding dimensions: %d", embedding.shape()[0]);

    return 0;
}
```

### DeepSeek (Cloud API)

```fusion
use std::ai;

fn main() -> int {
    let client: ai::DeepSeekClient = ai::DeepSeekClient::new(
        std::env::var("DEEPSEEK_API_KEY"),
    );

    let response: ai::ChatResponse = client.chat(ai::ChatRequest {
        model: "deepseek-chat",
        messages: [
            ai::Message { role: "system", content: "You are an expert programmer." },
            ai::Message { role: "user", content: "Write a function to compute fibonacci numbers." },
        ],
        temperature: 0.3,
        max_tokens: 512,
    });

    println("Code:\n%s", response.content);

    return 0;
}
```

### OpenAI-Compatible Provider

```fusion
use std::ai;

fn main() -> int {
    // Works with any OpenAI-compatible API (OpenAI, Together, Groq, etc.)
    let client: ai::OpenAICompatClient = ai::OpenAICompatClient::new(
        "https://api.openai.com/v1",
        std::env::var("OPENAI_API_KEY"),
    );

    let response: ai::ChatResponse = client.chat(ai::ChatRequest {
        model: "gpt-4o",
        messages: [
            ai::Message { role: "user", content: "What is the capital of France?" },
        ],
        temperature: 0.0,
        max_tokens: 50,
    });

    println("Answer: %s", response.content);

    return 0;
}
```

### Anthropic-Compatible Provider

```fusion
use std::ai;

fn main() -> int {
    let client: ai::AnthropicClient = ai::AnthropicClient::new(
        std::env::var("ANTHROPIC_API_KEY"),
    );

    let response: ai::ChatResponse = client.chat(ai::ChatRequest {
        model: "claude-sonnet-4-20250514",
        messages: [
            ai::Message { role: "user", content: "Explain the difference between ML and AI." },
        ],
        max_tokens: 512,
    });

    println("Response: %s", response.content);

    return 0;
}
```

---

## Configuration in Fusion.toml

```toml
[ml]
# Default device for tensor operations
device = "cpu"  # Options: "cpu", "cuda", "mps"

# Default data type
dtype = "float32"  # Options: "float32", "float16", "bfloat16"

# Enable automatic mixed precision
mixed_precision = false

# Random seed for reproducibility
seed = 42

[ml.gpu]
# CUDA device settings
cuda_device = 0
memory_fraction = 0.8
allow_growth = true

# Enable TF32 for Ampere+ GPUs
tf32 = true

[ml.training]
# Default batch size
batch_size = 32

# Default learning rate
learning_rate = 0.001

# Default optimizer
optimizer = "adam"  # Options: "sgd", "adam", "adamw", "rmsprop"

# Gradient clipping
clip_grad_norm = 1.0

# Enable gradient accumulation
gradient_accumulation_steps = 1

# Checkpoint settings
checkpoint_dir = "./checkpoints"
save_every_n_epochs = 10

[ml.distributed]
# Distributed training settings
backend = "nccl"  # Options: "nccl", "gloo", "mpi"
world_size = 1
rank = 0

[ai.providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama3.2"
timeout = 120

[ai.providers.mistral]
api_key_env = "MISTRAL_API_KEY"
default_model = "mistral-large-latest"
base_url = "https://api.mistral.ai/v1"

[ai.providers.deepseek]
api_key_env = "DEEPSEEK_API_KEY"
default_model = "deepseek-chat"
base_url = "https://api.deepseek.com/v1"

[ai.providers.openai]
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4o"
base_url = "https://api.openai.com/v1"

[ai.providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
default_model = "claude-sonnet-4-20250514"
base_url = "https://api.anthropic.com/v1"
```

---

## GPU Support

### Using GPU Tensors

```fusion
use std::ml;

fn main() -> int {
    // Check if GPU is available
    let gpu_available: bool = ml::cuda::is_available();
    println("GPU available: %d", gpu_available);

    if gpu_available {
        // Create tensor on GPU
        let gpu_tensor: ml::Tensor = ml::tensor([1.0, 2.0, 3.0]).cuda();
        println("Tensor on GPU: %s", gpu_tensor.device());

        // Move back to CPU
        let cpu_tensor: ml::Tensor = gpu_tensor.cpu();
        println("Tensor on CPU: %s", cpu_tensor.device());

        // Specify CUDA device
        let device: ml::Device = ml::cuda::device(1);  // Use second GPU
        let gpu_tensor2: ml::Tensor = ml::tensor([4.0, 5.0, 6.0]).to(device);
        println("Tensor on GPU 1: %s", gpu_tensor2.device());
    }

    return 0;
}
```

### GPU Training

```fusion
use std::ml;

fn main() -> int {
    let device: ml::Device = ml::cuda::device(0);

    // Move model to GPU
    let model: Network = Network::new().to(device);

    // Move data to GPU
    let inputs: ml::Tensor = ml::randn([32, 784]).to(device);
    let labels: ml::Tensor = ml::randint(0, 10, [32]).to(device);

    // Forward pass on GPU
    let outputs: ml::Tensor = model.forward(inputs);
    let loss: ml::Tensor = ml::cross_entropy(outputs, labels);

    println("Loss computed on GPU: %f", loss.item());

    return 0;
}
```

---

## Hybrid Quantum ML

### Variational Quantum Eigensolver (VQE)

```fusion
use std::quantum;
use std::ml;

fn quantum_ansatz(params: ml::Tensor, qubits: [quantum::SimQubit]) {
    // Parameterized quantum circuit
    for i in 0..qubits.len() {
        qubits[i].rx(params[i * 2].item());
        qubits[i].rz(params[i * 2 + 1].item());
    }

    for i in 0..qubits.len() - 1 {
        quantum::cnot(qubits[i], qubits[i + 1]);
    }
}

fn main() -> int {
    let num_qubits: int = 2;
    let num_params: int = num_qubits * 2;

    // Initialize parameters
    let params: ml::Tensor = ml::randn([num_params]).requires_grad(true);

    // Optimization loop
    let optimizer: ml::Adam = ml::Adam::new(0.01);

    for iteration in 0..100 {
        // Create simulator
        let sim: quantum::Simulator = quantum::Simulator::new();
        let qubits: [quantum::SimQubit] = [
            sim.allocate_qubit(),
            sim.allocate_qubit(),
        ];

        // Apply parameterized circuit
        quantum_ansatz(params, qubits);

        // Measure energy expectation
        let energy: float = sim.expectation_z(qubits[0]);

        // Compute gradient and update
        // (Simplified: in practice, use parameter shift rule)
        optimizer.zero_grad();
        // energy.backward()  // Quantum gradient estimation
        optimizer.step();

        if iteration %% 10 == 0 {
            println("Iteration %d: energy = %f", iteration, energy);
        }
    }

    return 0;
}
```

### Quantum Neural Network (QNN)

```fusion
use std::quantum;
use std::ml;

struct QuantumLayer {
    num_qubits: int,
    num_params: int,
}

impl QuantumLayer {
    fn new(num_qubits: int) -> QuantumLayer {
        return QuantumLayer {
            num_qubits,
            num_params: num_qubits * 3,  // RX, RY, RZ per qubit
        };
    }

    fn forward(self, params: ml::Tensor, qubits: [quantum::SimQubit]) {
        // Parameterized rotation on each qubit
        for i in 0..self.num_qubits {
            qubits[i].rx(params[i * 3].item());
            qubits[i].ry(params[i * 3 + 1].item());
            qubits[i].rz(params[i * 3 + 2].item());
        }

        // Entangling layer
        for i in 0..self.num_qubits - 1 {
            quantum::cnot(qubits[i], qubits[i + 1]);
        }
    }
}

fn main() -> int {
    let num_qubits: int = 3;
    let num_layers: int = 2;

    let layers: [QuantumLayer] = [
        QuantumLayer::new(num_qubits),
        QuantumLayer::new(num_qubits),
    ];

    // Total parameters
    let total_params: int = num_layers * layers[0].num_params;
    let params: ml::Tensor = ml::randn([total_params]).requires_grad(true);

    // Forward pass through QNN
    let sim: quantum::Simulator = quantum::Simulator::new();
    let qubits: [quantum::SimQubit] = [
        sim.allocate_qubit(),
        sim.allocate_qubit(),
        sim.allocate_qubit(),
    ];

    // Apply layers
    for layer_idx in 0..num_layers {
        let start: int = layer_idx * layers[layer_idx].num_params;
        let end: int = start + layers[layer_idx].num_params;
        let layer_params: ml::Tensor = params[start..end];
        layers[layer_idx].forward(layer_params, qubits);
    }

    // Measure
    let result: quantum::Measurement = sim.measure_all();
    println("QNN output: %s", result.to_string());

    return 0;
}
```

---

## Complete Examples

### Image Classification (MNIST)

```fusion
use std::ml;

struct MnistNet {
    conv1: Conv2d,
    conv2: Conv2d,
    fc1: Linear,
    fc2: Linear,
    dropout: Dropout,
}

impl MnistNet {
    fn new() -> MnistNet {
        return MnistNet {
            conv1: Conv2d::new(1, 32, 3, 1, 1),
            conv2: Conv2d::new(32, 64, 3, 1, 1),
            fc1: Linear::new(64 * 7 * 7, 128),
            fc2: Linear::new(128, 10),
            dropout: Dropout::new(0.25),
        };
    }

    fn forward(self, x: ml::Tensor) -> ml::Tensor {
        // Conv block 1
        let x: ml::Tensor = ml::relu(self.conv1.forward(x));
        let x: ml::Tensor = ml::max_pool2d(x, 2, 2);

        // Conv block 2
        let x: ml::Tensor = ml::relu(self.conv2.forward(x));
        let x: ml::Tensor = ml::max_pool2d(x, 2, 2);

        // Flatten and classify
        let x: ml::Tensor = x.flatten(1);
        let x: ml::Tensor = ml::relu(self.fc1.forward(x));
        let x: ml::Tensor = self.dropout.forward(x);
        return self.fc2.forward(x);
    }
}

fn main() -> int {
    let model: MnistNet = MnistNet::new();
    let optimizer: ml::Adam = ml::Adam::new(model.parameters(), 0.001);
    let loss_fn: ml::CrossEntropyLoss = ml::CrossEntropyLoss::new();

    // Load MNIST dataset
    let train_data: ml::Dataset = ml::datasets::mnist::load_train();
    let test_data: ml::Dataset = ml::datasets::mnist::load_test();

    // Training
    for epoch in 0..10 {
        let mut total_loss: float = 0.0;
        let mut correct: int = 0;
        let mut total: int = 0;

        for batch in train_data.batches(64) {
            let images: ml::Tensor = batch.images.reshape([-1, 1, 28, 28]);
            let labels: ml::Tensor = batch.labels;

            // Forward pass
            let logits: ml::Tensor = model.forward(images);
            let loss: ml::Tensor = loss_fn.forward(logits, labels);

            // Backward pass
            optimizer.zero_grad();
            loss.backward();
            optimizer.step();

            // Track metrics
            total_loss = total_loss + loss.item();
            let predictions: ml::Tensor = logits.argmax(dim=1);
            correct = correct + predictions.eq(labels).sum().item() as int;
            total = total + labels.shape()[0];
        }

        // Evaluate on test set
        let mut test_correct: int = 0;
        let mut test_total: int = 0;

        for batch in test_data.batches(256) {
            let images: ml::Tensor = batch.images.reshape([-1, 1, 28, 28]);
            let logits: ml::Tensor = model.forward(images);
            let predictions: ml::Tensor = logits.argmax(dim=1);
            test_correct = test_correct + predictions.eq(batch.labels).sum().item() as int;
            test_total = test_total + batch.labels.shape()[0];
        }

        println("Epoch %d: loss=%.4f, train_acc=%.2f%%, test_acc=%.2f%%",
            epoch,
            total_loss / train_data.num_batches(),
            (correct as float) / (total as float) * 100.0,
            (test_correct as float) / (test_total as float) * 100.0,
        );
    }

    return 0;
}
```

### Text Generation (Transformer)

```fusion
use std::ml;

struct TextGenerator {
    embedding: Embedding,
    transformer_layers: Vec<TransformerBlock>,
    output_proj: Linear,
    pos_encoding: ml::Tensor,
}

impl TextGenerator {
    fn new(vocab_size: int, embed_dim: int, num_heads: int, num_layers: int, max_seq_len: int) -> TextGenerator {
        let mut layers: Vec<TransformerBlock> = [];
        for _ in 0..num_layers {
            layers.push(TransformerBlock::new(embed_dim, num_heads, embed_dim * 4));
        }

        return TextGenerator {
            embedding: Embedding::new(vocab_size, embed_dim),
            transformer_layers: layers,
            output_proj: Linear::new(embed_dim, vocab_size),
            pos_encoding: ml::sinusoidal_encoding(max_seq_len, embed_dim),
        };
    }

    fn forward(self, input_ids: ml::Tensor) -> ml::Tensor {
        let seq_len: int = input_ids.shape()[1];

        // Token + positional embeddings
        let x: ml::Tensor = self.embedding.forward(input_ids) + self.pos_encoding[0..seq_len, :];

        // Transformer layers
        let mut x: ml::Tensor = x;
        for layer in self.transformer_layers {
            x = layer.forward(x);
        }

        // Project to vocabulary
        return self.output_proj.forward(x);
    }

    fn generate(self, prompt_ids: ml::Tensor, max_new_tokens: int, temperature: float) -> ml::Tensor {
        let mut tokens: ml::Tensor = prompt_ids;

        for _ in 0..max_new_tokens {
            let logits: ml::Tensor = self.forward(tokens);
            let next_logits: ml::Tensor = logits[:, -1, :] / temperature;
            let next_token: ml::Tensor = ml::multinomial(ml::softmax(next_logits), 1);
            tokens = ml::cat([tokens, next_token], dim=1);
        }

        return tokens;
    }
}

fn main() -> int {
    let vocab_size: int = 32000;
    let embed_dim: int = 512;
    let num_heads: int = 8;
    let num_layers: int = 6;

    let model: TextGenerator = TextGenerator::new(vocab_size, embed_dim, num_heads, num_layers, 1024);

    // Generate text from a prompt
    let prompt: ml::Tensor = ml::tensor([[101, 2023, 2003, 1037]]);  // Tokenized prompt
    let generated: ml::Tensor = model.generate(prompt, max_new_tokens=50, temperature=0.8);

    println("Generated token IDs: %s", generated.to_string());

    return 0;
}
```

---

## Tips and Best Practices

1. **Start simple**: Begin with small models and increase complexity.
2. **Use GPU when available**: GPU acceleration significantly speeds up training.
3. **Monitor gradients**: Watch for vanishing/exploding gradients.
4. **Use learning rate scheduling**: Adjust learning rate during training.
5. **Validate regularly**: Check model performance on validation data.
6. **Use mixed precision**: fp16/bfloat16 training saves memory and speeds up training.
7. **Use gradient clipping**: Prevent gradient explosion in RNNs and Transformers.
8. **Leverage transfer learning**: Use pre-trained models and fine-tune for your task.
9. **Use model checkpoints**: Save model state periodically for recovery.

---

## Cross-References

- **Chapter 7**: Post-Quantum Cryptography for secure model distribution
- **Chapter 8**: Quantum Computing for quantum-specific operations
- **Chapter 10**: Concurrency for distributed training
- **Chapter 14**: Examples for complete ML examples
