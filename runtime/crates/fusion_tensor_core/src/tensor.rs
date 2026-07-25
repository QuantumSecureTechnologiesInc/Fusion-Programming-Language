//! N-dimensional tensor with compile-time rank enforcement
//! Integrated from fusion_core Tensor Types.rs

use crate::error::{TensorError, TensorResult};
use fusion_traits::{DataType, Numeric};
use std::marker::PhantomData;

/// N-dimensional array with compile-time rank enforcement
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor<T: Numeric, const RANK: usize> {
    pub(crate) data: Vec<T>,
    pub(crate) shape: [usize; RANK],
    pub(crate) strides: [usize; RANK],
    pub(crate) dtype: DataType,
    pub(crate) _phantom: PhantomData<T>,
}

// Type aliases for common ranks
pub type Scalar<T> = Tensor<T, 0>;
pub type Vector<T> = Tensor<T, 1>;
pub type Matrix<T> = Tensor<T, 2>;

impl<T: Numeric, const RANK: usize> Tensor<T, RANK> {
    /// Create tensor filled with zeros
    pub fn zeros(shape: [usize; RANK]) -> Self {
        let size: usize = shape.iter().product();
        let data = vec![T::zero(); size];
        let strides = Self::compute_strides(&shape);

        Tensor {
            data,
            shape,
            strides,
            dtype: T::data_type(),
            _phantom: PhantomData,
        }
    }

    /// Create tensor filled with ones
    pub fn ones(shape: [usize; RANK]) -> Self {
        let size: usize = shape.iter().product();
        let data = vec![T::one(); size];
        let strides = Self::compute_strides(&shape);

        Tensor {
            data,
            shape,
            strides,
            dtype: T::data_type(),
            _phantom: PhantomData,
        }
    }

    /// Create tensor from vector with shape validation
    pub fn from_vec(data: Vec<T>, shape: [usize; RANK]) -> TensorResult<Self> {
        let size: usize = shape.iter().product();
        if data.len() != size {
            return Err(TensorError::ShapeMismatch {
                op: "Tensor::from_vec".into(),
                lhs: vec![data.len()],
                rhs: vec![size],
            });
        }

        let strides = Self::compute_strides(&shape);
        Ok(Tensor {
            data,
            shape,
            strides,
            dtype: T::data_type(),
            _phantom: PhantomData,
        })
    }

    /// Compute row-major strides from shape
    fn compute_strides(shape: &[usize; RANK]) -> [usize; RANK] {
        let mut strides = [1; RANK];
        if RANK > 0 {
            for i in (0..RANK - 1).rev() {
                strides[i] = strides[i + 1] * shape[i + 1];
            }
        }
        strides
    }

    /// Get element with bounds checking
    pub fn get(&self, indices: [usize; RANK]) -> TensorResult<T> {
        let index = self.compute_flat_index(&indices)?;
        Ok(self.data[index])
    }

    /// Set element with bounds checking
    pub fn set(&mut self, indices: [usize; RANK], value: T) -> TensorResult<()> {
        let index = self.compute_flat_index(&indices)?;
        self.data[index] = value;
        Ok(())
    }

    /// Compute flat index from multi-dimensional indices
    fn compute_flat_index(&self, indices: &[usize; RANK]) -> TensorResult<usize> {
        let mut index = 0;
        for i in 0..RANK {
            if indices[i] >= self.shape[i] {
                return Err(TensorError::IndexOutOfBounds {
                    indices: indices.to_vec(),
                    shape: self.shape.to_vec(),
                });
            }
            index += indices[i] * self.strides[i];
        }
        Ok(index)
    }

    /// Get tensor shape
    pub fn shape(&self) -> &[usize; RANK] {
        &self.shape
    }

    /// Get total number of elements
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get data type
    pub fn dtype(&self) -> DataType {
        self.dtype
    }

    /// Get raw data slice
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Get raw data as mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Get a reference to the raw data vector
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    /// Reshape into a tensor of a different rank.
    /// The total number of elements must remain the same.
    pub fn reshape<const NEW_RANK: usize>(
        self,
        new_shape: [usize; NEW_RANK],
    ) -> TensorResult<Tensor<T, NEW_RANK>> {
        let new_size: usize = new_shape.iter().product();
        if new_size != self.data.len() {
            return Err(TensorError::ShapeMismatch {
                op: "reshape".into(),
                lhs: vec![self.data.len()],
                rhs: vec![new_size],
            });
        }
        let new_strides = Tensor::<T, NEW_RANK>::compute_strides(&new_shape);
        Ok(Tensor {
            data: self.data,
            shape: new_shape,
            strides: new_strides,
            dtype: self.dtype,
            _phantom: PhantomData,
        })
    }

    /// Element-wise addition with broadcasting support.
    /// `other` may have fewer dimensions; leading dimensions are broadcast.
    pub fn add_broadcast(&self, other: &Self) -> TensorResult<Self> {
        if self.shape == other.shape {
            return self.broadcast_binary_op(other, |a, b| T::from_f64(a.to_f64() + b.to_f64()));
        }
        self.broadcast_binary_op(other, |a, b| T::from_f64(a.to_f64() + b.to_f64()))
    }

    /// Element-wise multiplication with broadcasting support.
    pub fn mul_broadcast(&self, other: &Self) -> TensorResult<Self> {
        if self.shape == other.shape {
            return self.broadcast_binary_op(other, |a, b| T::from_f64(a.to_f64() * b.to_f64()));
        }
        self.broadcast_binary_op(other, |a, b| T::from_f64(a.to_f64() * b.to_f64()))
    }

    /// Sum all elements
    pub fn sum_all(&self) -> T {
        self.data
            .iter()
            .fold(T::zero(), |acc, &x| T::from_f64(acc.to_f64() + x.to_f64()))
    }

    /// Mean of all elements
    pub fn mean_all(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.data.iter().map(|x| x.to_f64()).sum();
        sum / self.data.len() as f64
    }

    /// Max element
    pub fn max_all(&self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }
        let mut max = self.data[0];
        for &v in &self.data[1..] {
            if v.to_f64() > max.to_f64() {
                max = v;
            }
        }
        Some(max)
    }

    /// Min element
    pub fn min_all(&self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }
        let mut min = self.data[0];
        for &v in &self.data[1..] {
            if v.to_f64() < min.to_f64() {
                min = v;
            }
        }
        Some(min)
    }

    /// Internal: perform a broadcast binary op between self and other.
    /// other may have fewer dims; we right-align shapes and broadcast.
    fn broadcast_binary_op<F>(&self, other: &Self, op: F) -> TensorResult<Self>
    where
        F: Fn(T, T) -> T,
    {
        // Right-align the shapes: pad other's shape with 1s on the left
        let mut rhs_shape = [0usize; RANK];
        let offset = RANK - other.shape.len();
        for i in 0..RANK {
            if i < offset {
                rhs_shape[i] = 1; // broadcast dimension
            } else {
                rhs_shape[i] = other.shape[i - offset];
            }
        }

        // Validate broadcast compatibility
        for i in 0..RANK {
            if self.shape[i] != rhs_shape[i] && self.shape[i] != 1 && rhs_shape[i] != 1 {
                return Err(TensorError::ShapeMismatch {
                    op: "broadcast".into(),
                    lhs: self.shape.to_vec(),
                    rhs: other.shape.to_vec(),
                });
            }
        }

        let mut result = Self::zeros(self.shape);
        for flat in 0..result.data.len() {
            // Convert flat index in result to multi-dim indices
            let mut indices = [0usize; RANK];
            let mut tmp = flat;
            for i in 0..RANK {
                indices[i] = tmp / result.strides[i];
                tmp %= result.strides[i];
            }
            // Map indices to other tensor (broadcast)
            let mut other_indices = [0usize; RANK];
            for i in 0..RANK {
                other_indices[i] = if other.shape.len() > 0 && i >= offset {
                    let oi = i - offset;
                    if other.shape[oi] == 1 { 0 } else { indices[i] }
                } else {
                    0
                };
            }
            let a = self.data[flat];
            let b = other.data[other.compute_flat_index(&other_indices)?];
            result.data[flat] = op(a, b);
        }
        Ok(result)
    }
}

// Special implementations for Matrix (2D tensors)
impl<T: Numeric> Matrix<T> {
    /// Get matrix dimensions (rows, cols)
    pub fn dims(&self) -> (usize, usize) {
        (self.shape[0], self.shape[1])
    }

    /// Get element at (row, col)
    pub fn at(&self, row: usize, col: usize) -> TensorResult<T> {
        self.get([row, col])
    }

    /// Set element at (row, col)
    pub fn set_at(&mut self, row: usize, col: usize, value: T) -> TensorResult<()> {
        self.set([row, col], value)
    }

    /// Sum along an axis (0 = collapse rows, 1 = collapse cols).
    /// axis=0: [m, n] -> [n]; axis=1: [m, n] -> [m]
    pub fn sum_axis(&self, axis: usize) -> TensorResult<Vector<T>> {
        if axis >= 2 {
            return Err(TensorError::IndexOutOfBounds {
                indices: vec![axis],
                shape: vec![2],
            });
        }
        let (m, n) = self.dims();
        if axis == 0 {
            let mut result = Vector::<T>::zeros([n]);
            for j in 0..n {
                let mut sum = T::zero();
                for i in 0..m {
                    let val = self.at(i, j)?;
                    sum = T::from_f64(sum.to_f64() + val.to_f64());
                }
                result.set([j], sum)?;
            }
            Ok(result)
        } else {
            let mut result = Vector::<T>::zeros([m]);
            for i in 0..m {
                let mut sum = T::zero();
                for j in 0..n {
                    let val = self.at(i, j)?;
                    sum = T::from_f64(sum.to_f64() + val.to_f64());
                }
                result.set([i], sum)?;
            }
            Ok(result)
        }
    }

    /// Mean along an axis
    pub fn mean_axis(&self, axis: usize) -> TensorResult<Vector<f64>> {
        if axis >= 2 {
            return Err(TensorError::IndexOutOfBounds {
                indices: vec![axis],
                shape: vec![2],
            });
        }
        let (m, n) = self.dims();
        if axis == 0 {
            let mut result = Vector::<f64>::zeros([n]);
            for j in 0..n {
                let mut sum = 0.0;
                for i in 0..m {
                    sum += self.at(i, j)?.to_f64();
                }
                result.set([j], sum / m as f64)?;
            }
            Ok(result)
        } else {
            let mut result = Vector::<f64>::zeros([m]);
            for i in 0..m {
                let mut sum = 0.0;
                for j in 0..n {
                    sum += self.at(i, j)?.to_f64();
                }
                result.set([i], sum / n as f64)?;
            }
            Ok(result)
        }
    }

    /// Max along an axis, returning a Vector of the same numeric type.
    pub fn max_axis(&self, axis: usize) -> TensorResult<Vector<T>> {
        if axis >= 2 {
            return Err(TensorError::IndexOutOfBounds {
                indices: vec![axis],
                shape: vec![2],
            });
        }
        let (m, n) = self.dims();
        if axis == 0 {
            let mut result = Vector::<T>::zeros([n]);
            for j in 0..n {
                let mut max = self.at(0, j)?;
                for i in 1..m {
                    let v = self.at(i, j)?;
                    if v.to_f64() > max.to_f64() {
                        max = v;
                    }
                }
                result.set([j], max)?;
            }
            Ok(result)
        } else {
            let mut result = Vector::<T>::zeros([m]);
            for i in 0..m {
                let mut max = self.at(i, 0)?;
                for j in 1..n {
                    let v = self.at(i, j)?;
                    if v.to_f64() > max.to_f64() {
                        max = v;
                    }
                }
                result.set([i], max)?;
            }
            Ok(result)
        }
    }

    /// Element-wise division with broadcasting support
    pub fn div_broadcast(&self, other: &Self) -> TensorResult<Self> {
        if self.shape == other.shape {
            return self.div_op(other);
        }
        self.broadcast_binary_op(other, |a, b| {
            if b.to_f64() == 0.0 {
                T::zero() // safe division: 0/0 = 0
            } else {
                T::from_f64(a.to_f64() / b.to_f64())
            }
        })
    }

    /// Element-wise subtraction
    pub fn sub_op(&self, other: &Self) -> TensorResult<Self> {
        if self.shape() != other.shape() {
            return Err(TensorError::ShapeMismatch {
                op: "sub".into(),
                lhs: self.shape().to_vec(),
                rhs: other.shape().to_vec(),
            });
        }
        let (m, n) = self.dims();
        let mut result = Matrix::zeros([m, n]);
        for i in 0..m {
            for j in 0..n {
                let a = self.at(i, j)?;
                let b = other.at(i, j)?;
                result.set_at(i, j, T::from_f64(a.to_f64() - b.to_f64()))?;
            }
        }
        Ok(result)
    }

    /// Element-wise division
    pub fn div_op(&self, other: &Self) -> TensorResult<Self> {
        if self.shape() != other.shape() {
            return Err(TensorError::ShapeMismatch {
                op: "div".into(),
                lhs: self.shape().to_vec(),
                rhs: other.shape().to_vec(),
            });
        }
        let (m, n) = self.dims();
        let mut result = Matrix::zeros([m, n]);
        for i in 0..m {
            for j in 0..n {
                let a = self.at(i, j)?;
                let b = other.at(i, j)?;
                if b.to_f64() == 0.0 {
                    result.set_at(i, j, T::zero())?;
                } else {
                    result.set_at(i, j, T::from_f64(a.to_f64() / b.to_f64()))?;
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_creation() {
        let t: Tensor<f64, 2> = Tensor::zeros([3, 4]);
        assert_eq!(t.shape(), &[3, 4]);
        assert_eq!(t.size(), 12);
    }

    #[test]
    fn test_tensor_from_vec() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let t = Tensor::from_vec(data, [2, 2]).unwrap();
        assert_eq!(t.get([0, 0]).unwrap(), 1.0);
        assert_eq!(t.get([1, 1]).unwrap(), 4.0);
    }

    #[test]
    fn test_tensor_get_set() {
        let mut t: Tensor<i32, 2> = Tensor::zeros([2, 2]);
        t.set([0, 1], 42).unwrap();
        assert_eq!(t.get([0, 1]).unwrap(), 42);
    }

    #[test]
    fn test_bounds_checking() {
        let t: Tensor<f64, 2> = Tensor::zeros([2, 2]);
        assert!(t.get([3, 0]).is_err());
    }

    #[test]
    fn test_matrix_ops() {
        let m: Matrix<f64> = Matrix::ones([3, 3]);
        assert_eq!(m.dims(), (3, 3));
        assert_eq!(m.at(1, 1).unwrap(), 1.0);
    }

    #[test]
    fn test_reshape() {
        let t = Tensor::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], [2, 3]).unwrap();
        let reshaped: Tensor<f64, 3> = t.reshape([1, 3, 2]).unwrap();
        assert_eq!(reshaped.shape(), &[1, 3, 2]);
        assert_eq!(reshaped.get([0, 0, 0]).unwrap(), 1.0);
        assert_eq!(reshaped.get([0, 1, 0]).unwrap(), 3.0);
    }

    #[test]
    fn test_reshape_same_rank() {
        let t = Tensor::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let reshaped: Tensor<f64, 2> = t.reshape([4, 1]).unwrap();
        assert_eq!(reshaped.shape(), &[4, 1]);
        assert_eq!(reshaped.get([0, 0]).unwrap(), 1.0);
        assert_eq!(reshaped.get([2, 0]).unwrap(), 3.0);
    }

    #[test]
    fn test_reshape_size_mismatch() {
        let t = Tensor::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let result: std::result::Result<Tensor<f64, 3>, _> = t.reshape([1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_sum_all() {
        let t = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        assert_eq!(t.sum_all(), 10.0);
    }

    #[test]
    fn test_mean_all() {
        let t = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        assert_eq!(t.mean_all(), 2.5);
    }

    #[test]
    fn test_max_min_all() {
        let t = Matrix::from_vec(vec![3.0, 1.0, 4.0, 1.5], [2, 2]).unwrap();
        assert_eq!(t.max_all(), Some(4.0));
        assert_eq!(t.min_all(), Some(1.0));
    }

    #[test]
    fn test_sum_axis() {
        // [[1,2],[3,4]]
        let m = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        // sum along axis 0 (collapse rows) -> [4, 6]
        let v = m.sum_axis(0).unwrap();
        assert_eq!(v.get([0]).unwrap(), 4.0);
        assert_eq!(v.get([1]).unwrap(), 6.0);
        // sum along axis 1 (collapse cols) -> [3, 7]
        let v = m.sum_axis(1).unwrap();
        assert_eq!(v.get([0]).unwrap(), 3.0);
        assert_eq!(v.get([1]).unwrap(), 7.0);
    }

    #[test]
    fn test_max_axis() {
        // [[1,5],[3,2]]
        let m = Matrix::from_vec(vec![1.0, 5.0, 3.0, 2.0], [2, 2]).unwrap();
        let v = m.max_axis(0).unwrap();
        assert_eq!(v.get([0]).unwrap(), 3.0);
        assert_eq!(v.get([1]).unwrap(), 5.0);
    }

    #[test]
    fn test_div_op() {
        let a = Matrix::from_vec(vec![10.0, 20.0, 30.0, 40.0], [2, 2]).unwrap();
        let b = Matrix::from_vec(vec![2.0, 4.0, 5.0, 8.0], [2, 2]).unwrap();
        let c = a.div_op(&b).unwrap();
        assert_eq!(c.at(0, 0).unwrap(), 5.0);
        assert_eq!(c.at(1, 1).unwrap(), 5.0);
    }

    #[test]
    fn test_sub_op() {
        let a = Matrix::from_vec(vec![10.0, 20.0, 30.0, 40.0], [2, 2]).unwrap();
        let b = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let c = a.sub_op(&b).unwrap();
        assert_eq!(c.at(0, 0).unwrap(), 9.0);
        assert_eq!(c.at(1, 1).unwrap(), 36.0);
    }

    #[test]
    fn test_broadcast_add() {
        let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let b = Matrix::from_vec(vec![10.0, 20.0], [1, 2]).unwrap();
        let c = a.add_broadcast(&b).unwrap();
        assert_eq!(c.at(0, 0).unwrap(), 11.0);
        assert_eq!(c.at(1, 0).unwrap(), 13.0);
    }
}
