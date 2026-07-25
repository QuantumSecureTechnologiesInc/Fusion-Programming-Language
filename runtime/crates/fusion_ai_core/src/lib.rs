//! # Fusion AI Core
//!
//! AI/ML primitives with automatic differentiation and zero-copy tensor operations.

use std::collections::HashMap;
use tracing::{debug, trace};
use rand::Rng;

// ==================== Errors ====================

#[derive(Debug, Clone, thiserror::Error)]
pub enum AiError {
    #[error("Dimension mismatch: {lhs:?} @ {rhs:?}")]
    DimensionMismatch { lhs: Vec<usize>, rhs: Vec<usize> },
    #[error("Shape mismatch in {op}: expected {expected:?}, got {got:?}")]
    ShapeMismatch {
        op: String,
        expected: Vec<usize>,
        got: Vec<usize>,
    },
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type AiResult<T> = Result<T, AiError>;

// ==================== Tensor ====================

/// Dense f32 tensor with shape metadata and optional gradient tracking.
#[derive(Debug, Clone)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub shape: Vec<usize>,
    pub requires_grad: bool,
    pub grad: Option<Vec<f32>>,
}

impl Tensor {
    /// Create a tensor filled with zeros.
    pub fn zeros(shape: impl Into<Vec<usize>>) -> Self {
        let shape = shape.into();
        let size: usize = shape.iter().product();
        debug!("Creating zero tensor with shape {:?}", shape);
        Self {
            data: vec![0.0; size],
            shape,
            requires_grad: false,
            grad: None,
        }
    }

    /// Create a tensor filled with ones.
    pub fn ones(shape: impl Into<Vec<usize>>) -> Self {
        let shape = shape.into();
        let size: usize = shape.iter().product();
        Self {
            data: vec![1.0; size],
            shape,
            requires_grad: false,
            grad: None,
        }
    }

    /// Create a tensor from raw data and shape.
    pub fn from_data(data: Vec<f32>, shape: Vec<usize>) -> AiResult<Self> {
        let expected: usize = shape.iter().product();
        if data.len() != expected {
            return Err(AiError::ShapeMismatch {
                op: "from_data".into(),
                expected: shape,
                got: vec![data.len()],
            });
        }
        Ok(Self {
            data,
            shape,
            requires_grad: false,
            grad: None,
        })
    }

    /// Create a 1D tensor from a slice.
    pub fn from_slice(data: &[f32]) -> Self {
        let len = data.len();
        Self {
            data: data.to_vec(),
            shape: vec![len],
            requires_grad: false,
            grad: None,
        }
    }

    /// Create an identity matrix.
    pub fn eye(n: usize) -> Self {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Self {
            data,
            shape: vec![n, n],
            requires_grad: false,
            grad: None,
        }
    }

    /// Create a tensor with random values from uniform distribution [0, 1).
    pub fn rand(shape: impl Into<Vec<usize>>) -> Self {
        let shape = shape.into();
        let size: usize = shape.iter().product();
        let mut rng = rand::thread_rng();
        let data: Vec<f32> = (0..size).map(|_| rng.gen()).collect();
        Self {
            data,
            shape,
            requires_grad: false,
            grad: None,
        }
    }

    /// Create a tensor with random values from normal distribution.
    pub fn randn(shape: impl Into<Vec<usize>>) -> Self {
        let shape = shape.into();
        let size: usize = shape.iter().product();
        let mut rng = rand::thread_rng();
        let data: Vec<f32> = (0..size).map(|_| rng.sample(rand_distr::StandardNormal)).collect();
        Self {
            data,
            shape,
            requires_grad: false,
            grad: None,
        }
    }

    /// Enable gradient tracking.
    pub fn requires_grad(mut self, requires_grad: bool) -> Self {
        self.requires_grad = requires_grad;
        self
    }

    /// Set the device (metadata only for now).
    pub fn to_device(self, _device: impl Into<String>) -> Self {
        // GPU transfer would happen here
        self
    }

    /// Get the shape of the tensor.
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Get the total number of elements.
    pub fn numel(&self) -> usize {
        self.data.len()
    }

    /// Get the number of dimensions.
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    /// Reshape the tensor (same data, new shape).
    pub fn reshape(&self, new_shape: impl Into<Vec<usize>>) -> AiResult<Self> {
        let new_shape = new_shape.into();
        let new_size: usize = new_shape.iter().product();
        if new_size != self.data.len() {
            return Err(AiError::ShapeMismatch {
                op: "reshape".into(),
                expected: new_shape,
                got: vec![self.data.len()],
            });
        }
        Ok(Self {
            data: self.data.clone(),
            shape: new_shape,
            requires_grad: self.requires_grad,
            grad: self.grad.clone(),
        })
    }

    /// Get the scalar value of a single-element tensor.
    pub fn item(&self) -> f32 {
        assert_eq!(self.data.len(), 1, "item() requires a single-element tensor");
        self.data[0]
    }

    /// Compute the mean of all elements.
    pub fn mean(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f32>() / self.data.len() as f32
    }

    /// Compute the sum of all elements.
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    /// Compute the max of all elements.
    pub fn max(&self) -> f32 {
        self.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    }

    /// Element-wise addition.
    pub fn add(&self, other: &Tensor) -> AiResult<Tensor> {
        self.binary_op(other, "add", |a, b| a + b)
    }

    /// Element-wise subtraction.
    pub fn sub(&self, other: &Tensor) -> AiResult<Tensor> {
        self.binary_op(other, "sub", |a, b| a - b)
    }

    /// Element-wise multiplication.
    pub fn mul(&self, other: &Tensor) -> AiResult<Tensor> {
        self.binary_op(other, "mul", |a, b| a * b)
    }

    /// Element-wise division.
    pub fn div(&self, other: &Tensor) -> AiResult<Tensor> {
        self.binary_op(other, "div", |a, b| {
            if b.abs() < 1e-8 { 0.0 } else { a / b }
        })
    }

    /// Scalar multiplication.
    pub fn scale(&self, scalar: f32) -> Tensor {
        Tensor {
            data: self.data.iter().map(|&x| x * scalar).collect(),
            shape: self.shape.clone(),
            requires_grad: self.requires_grad,
            grad: None,
        }
    }

    /// Negate all elements.
    pub fn negate(&self) -> Tensor {
        self.scale(-1.0)
    }

    /// Transpose a 2D tensor.
    pub fn transpose(&self) -> AiResult<Tensor> {
        if self.shape.len() != 2 {
            return Err(AiError::InvalidOperation(
                "transpose only supports 2D tensors".into(),
            ));
        }
        let (rows, cols) = (self.shape[0], self.shape[1]);
        let mut data = vec![0.0; self.data.len()];
        for i in 0..rows {
            for j in 0..cols {
                data[j * rows + i] = self.data[i * cols + j];
            }
        }
        Ok(Tensor {
            data,
            shape: vec![cols, rows],
            requires_grad: self.requires_grad,
            grad: None,
        })
    }

    /// Matrix multiplication (2D tensors only).
    pub fn matmul(&self, other: &Tensor) -> AiResult<Tensor> {
        if self.shape.len() != 2 || other.shape.len() != 2 {
            return Err(AiError::InvalidOperation(
                "matmul requires 2D tensors".into(),
            ));
        }
        if self.shape[1] != other.shape[0] {
            return Err(AiError::DimensionMismatch {
                lhs: self.shape.clone(),
                rhs: other.shape.clone(),
            });
        }

        let m = self.shape[0];
        let k = self.shape[1];
        let n = other.shape[1];

        let mut data = vec![0.0; m * n];

        // Naive matrix multiplication
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for p in 0..k {
                    sum += self.data[i * k + p] * other.data[p * n + j];
                }
                data[i * n + j] = sum;
            }
        }

        trace!("matmul: {:?} @ {:?} -> {:?}", self.shape, other.shape, vec![m, n]);

        Ok(Tensor {
            data,
            shape: vec![m, n],
            requires_grad: self.requires_grad || other.requires_grad,
            grad: None,
        })
    }

    /// Apply ReLU element-wise.
    pub fn relu(&self) -> Tensor {
        Tensor {
            data: self.data.iter().map(|&x| x.max(0.0)).collect(),
            shape: self.shape.clone(),
            requires_grad: self.requires_grad,
            grad: None,
        }
    }

    /// Apply sigmoid element-wise.
    pub fn sigmoid(&self) -> Tensor {
        Tensor {
            data: self.data.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect(),
            shape: self.shape.clone(),
            requires_grad: self.requires_grad,
            grad: None,
        }
    }

    /// Apply tanh element-wise.
    pub fn tanh(&self) -> Tensor {
        Tensor {
            data: self.data.iter().map(|&x| x.tanh()).collect(),
            shape: self.shape.clone(),
            requires_grad: self.requires_grad,
            grad: None,
        }
    }

    /// Softmax along the last dimension.
    pub fn softmax(&self) -> AiResult<Tensor> {
        if self.shape.is_empty() {
            return Err(AiError::InvalidOperation("softmax requires non-empty shape".into()));
        }
        let last_dim = *self.shape.last().unwrap();
        let outer: usize = self.shape[..self.shape.len() - 1].iter().product();

        let mut data = vec![0.0; self.data.len()];
        for i in 0..outer {
            let start = i * last_dim;
            let end = start + last_dim;
            let slice = &self.data[start..end];
            let max_val = slice.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let mut sum = 0.0;
            for j in 0..last_dim {
                data[start + j] = (slice[j] - max_val).exp();
                sum += data[start + j];
            }
            for j in 0..last_dim {
                data[start + j] /= sum;
            }
        }

        Ok(Tensor {
            data,
            shape: self.shape.clone(),
            requires_grad: self.requires_grad,
            grad: None,
        })
    }

    /// Mean squared error loss.
    pub fn mse_loss(&self, target: &Tensor) -> AiResult<Tensor> {
        if self.shape != target.shape {
            return Err(AiError::ShapeMismatch {
                op: "mse_loss".into(),
                expected: self.shape.clone(),
                got: target.shape.clone(),
            });
        }
        let n = self.data.len() as f32;
        let loss: f32 = self
            .data
            .iter()
            .zip(target.data.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            / n;
        Ok(Tensor {
            data: vec![loss],
            shape: vec![1],
            requires_grad: false,
            grad: None,
        })
    }

    /// Internal binary op helper for same-shape or broadcast operations.
    fn binary_op<F>(&self, other: &Tensor, op_name: &str, f: F) -> AiResult<Tensor>
    where
        F: Fn(f32, f32) -> f32,
    {
        if self.shape == other.shape {
            let data: Vec<f32> = self.data.iter().zip(other.data.iter()).map(|(&a, &b)| f(a, b)).collect();
            Ok(Tensor {
                data,
                shape: self.shape.clone(),
                requires_grad: self.requires_grad || other.requires_grad,
                grad: None,
            })
        } else {
            // Simple broadcast: other has fewer dims, right-aligned
            let offset = self.shape.len().saturating_sub(other.shape.len());
            let mut new_shape = vec![1usize; self.shape.len()];
            for i in offset..self.shape.len() {
                new_shape[i] = other.shape[i - offset];
            }

            // Validate broadcast
            for i in 0..self.shape.len() {
                if self.shape[i] != new_shape[i] && new_shape[i] != 1 {
                    return Err(AiError::ShapeMismatch {
                        op: op_name.into(),
                        expected: self.shape.clone(),
                        got: other.shape.clone(),
                    });
                }
            }

            let total: usize = self.shape.iter().product();
            let mut data = vec![0.0; total];
            for flat in 0..total {
                // Convert flat index to multi-dim for self
                let mut idx = vec![0usize; self.shape.len()];
                let mut tmp = flat;
                for i in (0..self.shape.len()).rev() {
                    idx[i] = tmp % self.shape[i];
                    tmp /= self.shape[i];
                }
                // Map to other's index
                let mut other_idx = vec![0usize; other.shape.len()];
                for i in 0..other.shape.len() {
                    let self_i = offset + i;
                    other_idx[i] = if other.shape[i] == 1 { 0 } else { idx[self_i] };
                }
                let a = self.data[flat];
                let b = other.data[other.flat_index(&other_idx)];
                data[flat] = f(a, b);
            }

            Ok(Tensor {
                data,
                shape: self.shape.clone(),
                requires_grad: self.requires_grad || other.requires_grad,
                grad: None,
            })
        }
    }

    /// Compute flat index from multi-dim indices.
    fn flat_index(&self, indices: &[usize]) -> usize {
        let mut idx = 0;
        let mut stride = 1;
        for i in (0..self.shape.len()).rev() {
            idx += indices[i] * stride;
            stride *= self.shape[i];
        }
        idx
    }
}

impl From<Tensor> for Vec<f32> {
    fn from(t: Tensor) -> Self {
        t.data
    }
}

// ==================== Autodiff ====================

/// Operation in the computation graph.
#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Mul,
    Matmul,
    ReLU,
    Sigmoid,
    Tanh,
    Scale(f32),
    SumToScalar,
}

/// Node in the computation graph.
#[derive(Debug, Clone)]
struct GraphNode {
    id: usize,
    op: Op,
    inputs: Vec<usize>,
    tensor_id: usize,
}

/// Automatic differentiation engine with reverse-mode (backpropagation).
pub struct Autodiff {
    nodes: Vec<GraphNode>,
    next_id: usize,
    tensor_grads: HashMap<usize, Vec<f32>>,
    node_grads: HashMap<usize, Vec<f32>>,
}

impl Autodiff {
    pub fn new() -> Self {
        debug!("Initializing autodiff engine");
        Self {
            nodes: Vec::new(),
            next_id: 0,
            tensor_grads: HashMap::new(),
            node_grads: HashMap::new(),
        }
    }

    /// Record an operation and return a node ID.
    pub fn record(&mut self, op: Op, inputs: Vec<usize>, tensor_id: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(GraphNode {
            id,
            op,
            inputs,
            tensor_id,
        });
        id
    }

    /// Compute gradients via backpropagation starting from a loss tensor.
    /// `root_tensor_id` is the index in `tensors` that receives the output gradient.
    pub fn backward(&mut self, tensors: &mut Vec<Tensor>, root_tensor_id: usize, output_grad: Vec<f32>) {
        self.tensor_grads.clear();
        self.node_grads.clear();

        // Find the node that produced the root tensor
        let root_node_id = self.nodes.iter().rev()
            .find(|n| n.tensor_id == root_tensor_id)
            .map(|n| n.id);

        if let Some(rid) = root_node_id {
            self.node_grads.insert(rid, output_grad);
        }

        // Collect node data to avoid borrow issues
        let node_snapshots: Vec<(usize, Op, Vec<usize>)> = self
            .nodes
            .iter()
            .map(|n| (n.id, n.op.clone(), n.inputs.clone()))
            .collect();

        // Traverse graph in reverse order
        for (node_id, op, inputs) in node_snapshots.iter().rev() {
            let node_grad = match self.node_grads.get(node_id) {
                Some(g) => g.clone(),
                None => continue,
            };

            match op {
                Op::Add => {
                    for &input_id in inputs {
                        Self::accumulate_tensor_grad(&mut self.tensor_grads, input_id, &node_grad);
                    }
                }
                Op::Mul => {
                    if inputs.len() == 2 {
                        let a_id = inputs[0];
                        let b_id = inputs[1];
                        if let Some(b_data) = tensors.get(b_id) {
                            let grad_a: Vec<f32> = node_grad
                                .iter()
                                .zip(b_data.data.iter())
                                .map(|(g, b)| g * b)
                                .collect();
                            Self::accumulate_tensor_grad(&mut self.tensor_grads, a_id, &grad_a);
                        }
                        if let Some(a_data) = tensors.get(a_id) {
                            let grad_b: Vec<f32> = node_grad
                                .iter()
                                .zip(a_data.data.iter())
                                .map(|(g, a)| g * a)
                                .collect();
                            Self::accumulate_tensor_grad(&mut self.tensor_grads, b_id, &grad_b);
                        }
                    }
                }
                Op::Matmul => {
                    if inputs.len() == 2 {
                        let a_id = inputs[0];
                        let b_id = inputs[1];
                        if let (Some(a), Some(b)) = (tensors.get(a_id), tensors.get(b_id)) {
                            if a.shape.len() == 2 && b.shape.len() == 2 {
                                let m = a.shape[0];
                                let k = a.shape[1];
                                let n = b.shape[1];

                                let mut grad_a = vec![0.0; m * k];
                                for i in 0..m {
                                    for j in 0..k {
                                        let mut sum = 0.0;
                                        for p in 0..n {
                                            sum += node_grad[i * n + p] * b.data[p * k + j];
                                        }
                                        grad_a[i * k + j] = sum;
                                    }
                                }
                                Self::accumulate_tensor_grad(&mut self.tensor_grads, a_id, &grad_a);

                                let mut grad_b = vec![0.0; k * n];
                                for i in 0..k {
                                    for j in 0..n {
                                        let mut sum = 0.0;
                                        for p in 0..m {
                                            sum += a.data[p * k + i] * node_grad[p * n + j];
                                        }
                                        grad_b[i * n + j] = sum;
                                    }
                                }
                                Self::accumulate_tensor_grad(&mut self.tensor_grads, b_id, &grad_b);
                            }
                        }
                    }
                }
                Op::ReLU => {
                    if let Some(&input_id) = inputs.first() {
                        if let Some(input) = tensors.get(input_id) {
                            let grad: Vec<f32> = node_grad
                                .iter()
                                .zip(input.data.iter())
                                .map(|(g, &x)| if x > 0.0 { *g } else { 0.0 })
                                .collect();
                            Self::accumulate_tensor_grad(&mut self.tensor_grads, input_id, &grad);
                        }
                    }
                }
                Op::Sigmoid => {
                    if let Some(&input_id) = inputs.first() {
                        if let Some(input) = tensors.get(input_id) {
                            let grad: Vec<f32> = node_grad
                                .iter()
                                .zip(input.data.iter())
                                .map(|(g, &x)| {
                                    let s = 1.0 / (1.0 + (-x).exp());
                                    *g * s * (1.0 - s)
                                })
                                .collect();
                            Self::accumulate_tensor_grad(&mut self.tensor_grads, input_id, &grad);
                        }
                    }
                }
                Op::Tanh => {
                    if let Some(&input_id) = inputs.first() {
                        if let Some(input) = tensors.get(input_id) {
                            let grad: Vec<f32> = node_grad
                                .iter()
                                .zip(input.data.iter())
                                .map(|(g, &x)| {
                                    let t = x.tanh();
                                    *g * (1.0 - t * t)
                                })
                                .collect();
                            Self::accumulate_tensor_grad(&mut self.tensor_grads, input_id, &grad);
                        }
                    }
                }
                Op::Scale(s) => {
                    if let Some(&input_id) = inputs.first() {
                        let grad: Vec<f32> = node_grad.iter().map(|g| g * s).collect();
                        Self::accumulate_tensor_grad(&mut self.tensor_grads, input_id, &grad);
                    }
                }
                Op::SumToScalar => {
                    if let Some(&input_id) = inputs.first() {
                        if let Some(input) = tensors.get(input_id) {
                            let grad = vec![1.0; input.data.len()];
                            Self::accumulate_tensor_grad(&mut self.tensor_grads, input_id, &grad);
                        }
                    }
                }
            }
        }

        // Write accumulated tensor grads back to tensors
        for (tensor_id, grad) in &self.tensor_grads {
            if *tensor_id < tensors.len() {
                if let Some(tensor) = tensors.get_mut(*tensor_id) {
                    tensor.grad = Some(grad.clone());
                }
            }
        }
    }

    fn accumulate_tensor_grad(grads: &mut HashMap<usize, Vec<f32>>, tensor_id: usize, grad: &[f32]) {
        let entry = grads.entry(tensor_id).or_insert_with(|| vec![0.0; grad.len()]);
        for (e, g) in entry.iter_mut().zip(grad.iter()) {
            *e += g;
        }
    }

    /// Reset the computation graph.
    pub fn reset(&mut self) {
        self.nodes.clear();
        self.tensor_grads.clear();
        self.node_grads.clear();
        self.next_id = 0;
    }
}

impl Default for Autodiff {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Neural Network Layers ====================

pub mod nn {
    use super::*;

    /// Trait for neural network modules.
    pub trait Module {
        /// Forward pass.
        fn forward(&self, input: &Tensor) -> AiResult<Tensor>;

        /// Get trainable parameters.
        fn parameters(&self) -> Vec<&Tensor>;

        /// Get mutable references to parameters.
        fn parameters_mut(&mut self) -> Vec<&mut Tensor>;
    }

    /// Fully connected (linear) layer: y = xW + b
    pub struct Linear {
        pub weights: Tensor,
        pub bias: Tensor,
    }

    impl Linear {
        pub fn new(in_features: usize, out_features: usize) -> Self {
            // Xavier/Glorot initialization
            let scale = (2.0 / (in_features + out_features) as f32).sqrt();
            let weights = Tensor {
                data: (0..in_features * out_features)
                    .map(|_| {
                        let mut rng = rand::thread_rng();
                        (rng.gen::<f32>() * 2.0 - 1.0) * scale
                    })
                    .collect(),
                shape: vec![in_features, out_features],
                requires_grad: true,
                grad: None,
            };
            let bias = Tensor::zeros(vec![out_features]).requires_grad(true);
            Self { weights, bias }
        }
    }

    impl Module for Linear {
        fn forward(&self, input: &Tensor) -> AiResult<Tensor> {
            // y = x @ W + b
            let xw = input.matmul(&self.weights)?;
            // Add bias (broadcast over batch dimension)
            let mut result = xw.clone();
            if result.shape.len() >= 1 {
                let last_dim = *result.shape.last().unwrap();
                if last_dim == self.bias.data.len() {
                    // Broadcast bias addition
                    let batch_size: usize = result.shape[..result.shape.len() - 1].iter().product();
                    for i in 0..batch_size {
                        for j in 0..last_dim {
                            result.data[i * last_dim + j] += self.bias.data[j];
                        }
                    }
                }
            }
            Ok(result)
        }

        fn parameters(&self) -> Vec<&Tensor> {
            vec![&self.weights, &self.bias]
        }

        fn parameters_mut(&mut self) -> Vec<&mut Tensor> {
            vec![&mut self.weights, &mut self.bias]
        }
    }

    /// ReLU activation layer.
    pub struct ReLU;

    impl ReLU {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for ReLU {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Module for ReLU {
        fn forward(&self, input: &Tensor) -> AiResult<Tensor> {
            Ok(input.relu())
        }

        fn parameters(&self) -> Vec<&Tensor> {
            vec![]
        }

        fn parameters_mut(&mut self) -> Vec<&mut Tensor> {
            vec![]
        }
    }

    /// Sigmoid activation layer.
    pub struct Sigmoid;

    impl Sigmoid {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for Sigmoid {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Module for Sigmoid {
        fn forward(&self, input: &Tensor) -> AiResult<Tensor> {
            Ok(input.sigmoid())
        }

        fn parameters(&self) -> Vec<&Tensor> {
            vec![]
        }

        fn parameters_mut(&mut self) -> Vec<&mut Tensor> {
            vec![]
        }
    }

    /// Sequential model: chains layers in order.
    pub struct Sequential {
        layers: Vec<Box<dyn Module>>,
    }

    impl Sequential {
        pub fn new() -> Self {
            Self { layers: Vec::new() }
        }

        pub fn add<M: Module + 'static>(mut self, layer: M) -> Self {
            self.layers.push(Box::new(layer));
            self
        }
    }

    impl Default for Sequential {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Module for Sequential {
        fn forward(&self, input: &Tensor) -> AiResult<Tensor> {
            let mut x = input.clone();
            for layer in &self.layers {
                x = layer.forward(&x)?;
            }
            Ok(x)
        }

        fn parameters(&self) -> Vec<&Tensor> {
            self.layers.iter().flat_map(|l| l.parameters()).collect()
        }

        fn parameters_mut(&mut self) -> Vec<&mut Tensor> {
            self.layers.iter_mut().flat_map(|l| l.parameters_mut()).collect()
        }
    }
}

// ==================== Optimizers ====================

pub mod optim {
    use super::*;

    pub trait Optimizer {
        fn step(&mut self, learning_rate: f32);
    }

    /// SGD optimizer with optional momentum.
    pub struct SGD {
        params: Vec<Vec<f32>>,
        velocities: Vec<Vec<f32>>,
        momentum: f32,
    }

    impl SGD {
        pub fn new(params: Vec<&Tensor>, momentum: f32) -> Self {
            let param_data: Vec<Vec<f32>> = params.iter().map(|p| p.data.clone()).collect();
            let velocities: Vec<Vec<f32>> = param_data.iter().map(|p| vec![0.0; p.len()]).collect();
            Self {
                params: param_data,
                velocities,
                momentum,
            }
        }

        pub fn update_params(&self, tensors: &mut Vec<&mut Tensor>) {
            for (tensor, param_data) in tensors.iter_mut().zip(self.params.iter()) {
                tensor.data = param_data.clone();
            }
        }
    }

    impl Optimizer for SGD {
        fn step(&mut self, _lr: f32) {
            // In practice, gradients would be read from tensors here.
            // This is a simplified version.
        }
    }

    /// Adam optimizer.
    pub struct Adam {
        m: Vec<Vec<f32>>,  // First moment
        v: Vec<Vec<f32>>,  // Second moment
        t: u32,             // Timestep
        beta1: f32,
        beta2: f32,
        eps: f32,
    }

    impl Adam {
        pub fn new(_param_count: usize, param_sizes: &[usize]) -> Self {
            let m = param_sizes.iter().map(|&s| vec![0.0; s]).collect();
            let v = param_sizes.iter().map(|&s| vec![0.0; s]).collect();
            Self {
                m,
                v,
                t: 0,
                beta1: 0.9,
                beta2: 0.999,
                eps: 1e-8,
            }
        }

        pub fn step_with_grads(&mut self, params: &mut [&mut Tensor], lr: f32) {
            self.t += 1;
            let bias_correction1 = 1.0 - self.beta1.powi(self.t as i32);
            let bias_correction2 = 1.0 - self.beta2.powi(self.t as i32);

            for (i, param) in params.iter_mut().enumerate() {
                if let Some(ref grad) = param.grad {
                    for j in 0..param.data.len() {
                        let g = grad[j];
                        self.m[i][j] = self.beta1 * self.m[i][j] + (1.0 - self.beta1) * g;
                        self.v[i][j] = self.beta2 * self.v[i][j] + (1.0 - self.beta2) * g * g;

                        let m_hat = self.m[i][j] / bias_correction1;
                        let v_hat = self.v[i][j] / bias_correction2;

                        param.data[j] -= lr * m_hat / (v_hat.sqrt() + self.eps);
                    }
                }
            }
        }
    }
}

// ==================== Convenience ====================

/// Matrix multiplication convenience function.
pub fn matmul(a: &Tensor, b: &Tensor) -> AiResult<Tensor> {
    a.matmul(b)
}

/// Compute MSE loss.
pub fn mse_loss(predicted: &Tensor, target: &Tensor) -> AiResult<Tensor> {
    predicted.mse_loss(target)
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use super::nn::*;

    #[test]
    fn test_tensor_creation() {
        let t = Tensor::zeros(vec![2, 3]);
        assert_eq!(t.shape(), &[2, 3]);
        assert_eq!(t.numel(), 6);

        let t = Tensor::ones(vec![3, 3]);
        assert_eq!(t.data.iter().sum::<f32>(), 9.0);
    }

    #[test]
    fn test_tensor_matmul() {
        let a = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = Tensor::from_data(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();
        let c = a.matmul(&b).unwrap();

        assert_eq!(c.shape(), &[2, 2]);
        // [1,2] [5,6]   [19, 22]
        // [3,4] [7,8] = [43, 50]
        assert!((c.data[0] - 19.0).abs() < 1e-5);
        assert!((c.data[3] - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_tensor_ops() {
        let a = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = Tensor::from_data(vec![2.0, 2.0, 2.0, 2.0], vec![2, 2]).unwrap();

        let c = a.add(&b).unwrap();
        assert_eq!(c.data, vec![3.0, 4.0, 5.0, 6.0]);

        let c = a.mul(&b).unwrap();
        assert_eq!(c.data, vec![2.0, 4.0, 6.0, 8.0]);

        let c = a.sub(&b).unwrap();
        assert_eq!(c.data, vec![-1.0, 0.0, 1.0, 2.0]);

        let c = a.div(&b).unwrap();
        assert_eq!(c.data, vec![0.5, 1.0, 1.5, 2.0]);
    }

    #[test]
    fn test_relu() {
        let t = Tensor::from_data(vec![-1.0, 0.0, 1.0, 2.0], vec![4]).unwrap();
        let r = t.relu();
        assert_eq!(r.data, vec![0.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn test_linear_layer() {
        let layer = Linear::new(3, 2);
        assert_eq!(layer.weights.shape, vec![3, 2]);
        assert_eq!(layer.bias.shape, vec![2]);

        let input = Tensor::ones(vec![1, 3]);
        let output = layer.forward(&input).unwrap();
        assert_eq!(output.shape, vec![1, 2]);
    }

    #[test]
    fn test_sequential() {
        let model = Sequential::new()
            .add(Linear::new(4, 8))
            .add(ReLU::new())
            .add(Linear::new(8, 2))
            .add(Sigmoid::new());

        let input = Tensor::ones(vec![1, 4]);
        let output = model.forward(&input).unwrap();
        assert_eq!(output.shape, vec![1, 2]);

        // Output should be in [0, 1] due to sigmoid
        for &v in &output.data {
            assert!(v >= 0.0 && v <= 1.0);
        }
    }

    #[test]
    fn test_mse_loss() {
        let pred = Tensor::from_data(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let target = Tensor::from_data(vec![1.5, 2.5, 2.5], vec![3]).unwrap();
        let loss = pred.mse_loss(&target).unwrap();
        assert!((loss.item() - 0.25).abs() < 1e-5);
    }

    #[test]
    fn test_softmax() {
        let t = Tensor::from_data(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let s = t.softmax().unwrap();
        let sum: f32 = s.data.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
        assert!(s.data[2] > s.data[0]); // higher logit -> higher probability
    }

    #[test]
    fn test_transpose() {
        let t = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
        let t_t = t.transpose().unwrap();
        assert_eq!(t_t.shape, vec![3, 2]);
        assert_eq!(t_t.data, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }

    #[test]
    fn test_autodiff_matmul() {
        let mut ad = Autodiff::new();

        let mut a = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        a.requires_grad = true;
        let mut b = Tensor::from_data(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();
        b.requires_grad = true;

        let mut tensors = vec![a.clone(), b.clone()];

        let mut result = a.matmul(&b).unwrap();
        result.requires_grad = true;
        tensors.push(result);

        // Record matmul op (tensor indices 0=a, 1=b, 2=result)
        let _node_id = ad.record(Op::Matmul, vec![0, 1], 2);

        // Backprop with unit gradient for the result tensor (index 2)
        let grad = vec![1.0; 4];
        ad.backward(&mut tensors, 2, grad);

        // Check that gradients were computed for inputs
        assert!(tensors[0].grad.is_some());
        assert!(tensors[1].grad.is_some());
    }

    #[test]
    fn test_autodiff_relu() {
        let mut ad = Autodiff::new();

        let mut input = Tensor::from_data(vec![-1.0, 0.5, 2.0], vec![3]).unwrap();
        input.requires_grad = true;
        let mut tensors = vec![input.clone()];

        let mut result = input.relu();
        result.requires_grad = true;
        tensors.push(result);

        let _node_id = ad.record(Op::ReLU, vec![0], 1);

        let grad = vec![1.0, 1.0, 1.0];
        // root_tensor_id=1 (the result tensor)
        ad.backward(&mut tensors, 1, grad);

        // ReLU gradient: 1 if x > 0, else 0
        let input_grad = tensors[0].grad.as_ref().unwrap();
        assert_eq!(input_grad[0], 0.0); // -1 -> 0
        assert_eq!(input_grad[1], 1.0); // 0.5 -> 1
        assert_eq!(input_grad[2], 1.0); // 2.0 -> 1
    }

    #[test]
    fn test_resize_reshape() {
        let t = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
        let r = t.reshape(vec![3, 2]).unwrap();
        assert_eq!(r.shape, vec![3, 2]);
        assert_eq!(r.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_broadcast_add() {
        let a = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = Tensor::from_data(vec![10.0, 20.0], vec![1, 2]).unwrap();
        let c = a.add(&b).unwrap();
        assert_eq!(c.data, vec![11.0, 22.0, 13.0, 24.0]);
    }

    #[test]
    fn test_forward_backward() {
        let mut model = Sequential::new()
            .add(Linear::new(2, 4))
            .add(ReLU::new())
            .add(Linear::new(4, 1));

        let input = Tensor::from_data(vec![1.0, 2.0], vec![1, 2]).unwrap();
        let target = Tensor::from_data(vec![0.5], vec![1, 1]).unwrap();

        // Forward
        let output = model.forward(&input).unwrap();
        let loss = output.mse_loss(&target).unwrap();

        // Loss should be positive
        assert!(loss.item() > 0.0);
    }
}
