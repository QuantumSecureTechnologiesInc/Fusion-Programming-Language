//! Tensor operations (matrix multiply, transpose, etc.)
//! Integrated from fusion_core Tensor Operations.rs

use crate::error::{TensorError, TensorResult};
use crate::tensor::Matrix;
use fusion_traits::Numeric;

/// Tensor operations trait
pub trait TensorOps<T: Numeric> {
    /// Matrix multiplication (for 2D tensors)
    fn matmul(&self, other: &Self) -> TensorResult<Self>
    where
        Self: Sized;

    /// Transpose (for 2D tensors)
    fn transpose(&self) -> Self;

    /// Element-wise addition
    fn add(&self, other: &Self) -> TensorResult<Self>
    where
        Self: Sized;

    /// Element-wise multiplication
    fn mul(&self, other: &Self) -> TensorResult<Self>
    where
        Self: Sized;

    /// Element-wise division
    fn div(&self, other: &Self) -> TensorResult<Self>
    where
        Self: Sized;

    /// Element-wise subtraction
    fn sub(&self, other: &Self) -> TensorResult<Self>
    where
        Self: Sized;

    /// Hadamard (element-wise) product
    fn hadamard(&self, other: &Self) -> TensorResult<Self>
    where
        Self: Sized;

    /// Scale by a scalar
    fn scale(&self, scalar: T) -> Self;

    /// Negate all elements
    fn negate(&self) -> Self;
}

impl<T: Numeric> TensorOps<T> for Matrix<T> {
    fn matmul(&self, other: &Self) -> TensorResult<Self> {
        let (m, k) = self.dims();
        let (k2, n) = other.dims();

        if k != k2 {
            return Err(TensorError::MatrixDimensionMismatch {
                op: "matmul".into(),
                lhs_shape: (m, k),
                rhs_shape: (k2, n),
            });
        }

        let mut result = Matrix::zeros([m, n]);

        for i in 0..m {
            for j in 0..n {
                let mut sum = T::zero();
                for p in 0..k {
                    let a_val = self.at(i, p)?;
                    let b_val = other.at(p, j)?;
                    // Convert to f64, compute, and convert back
                    let prod = a_val.to_f64() * b_val.to_f64();
                    sum = T::from_f64(sum.to_f64() + prod);
                }
                result.set_at(i, j, sum)?;
            }
        }

        Ok(result)
    }

    fn transpose(&self) -> Self {
        let (m, n) = self.dims();
        let mut result = Matrix::zeros([n, m]);

        for i in 0..m {
            for j in 0..n {
                if let Ok(val) = self.at(i, j) {
                    let _ = result.set_at(j, i, val);
                }
            }
        }

        result
    }

    fn add(&self, other: &Self) -> TensorResult<Self> {
        if self.shape() != other.shape() {
            return Err(TensorError::ShapeMismatch {
                op: "add".into(),
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
                let sum = T::from_f64(a.to_f64() + b.to_f64());
                result.set_at(i, j, sum)?;
            }
        }

        Ok(result)
    }

    fn mul(&self, other: &Self) -> TensorResult<Self> {
        if self.shape() != other.shape() {
            return Err(TensorError::ShapeMismatch {
                op: "mul".into(),
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
                let prod = T::from_f64(a.to_f64() * b.to_f64());
                result.set_at(i, j, prod)?;
            }
        }

        Ok(result)
    }

    fn div(&self, other: &Self) -> TensorResult<Self> {
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

    fn sub(&self, other: &Self) -> TensorResult<Self> {
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

    fn hadamard(&self, other: &Self) -> TensorResult<Self> {
        self.mul(other)
    }

    fn scale(&self, scalar: T) -> Self {
        let (m, n) = self.dims();
        let mut result = Matrix::zeros([m, n]);
        for i in 0..m {
            for j in 0..n {
                if let Ok(val) = self.at(i, j) {
                    let _ = result.set_at(i, j, T::from_f64(val.to_f64() * scalar.to_f64()));
                }
            }
        }
        result
    }

    fn negate(&self) -> Self {
        let (m, n) = self.dims();
        let mut result = Matrix::zeros([m, n]);
        for i in 0..m {
            for j in 0..n {
                if let Ok(val) = self.at(i, j) {
                    let _ = result.set_at(i, j, T::from_f64(-val.to_f64()));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matmul() {
        let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let b = Matrix::from_vec(vec![5.0, 6.0, 7.0, 8.0], [2, 2]).unwrap();

        let c = a.matmul(&b).unwrap();
        // [1,2] [5,6]   [19, 22]
        // [3,4] [7,8] = [43, 50]
        assert_eq!(c.at(0, 0).unwrap(), 19.0);
        assert_eq!(c.at(1, 1).unwrap(), 50.0);
    }

    #[test]
    fn test_transpose() {
        let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], [2, 3]).unwrap();
        let b = a.transpose();
        assert_eq!(b.dims(), (3, 2));
        assert_eq!(b.at(0, 0).unwrap(), 1.0);
        assert_eq!(b.at(2, 1).unwrap(), 6.0);
    }

    #[test]
    fn test_add() {
        let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let b = Matrix::from_vec(vec![5.0, 6.0, 7.0, 8.0], [2, 2]).unwrap();
        let c = a.add(&b).unwrap();
        assert_eq!(c.at(0, 0).unwrap(), 6.0);
        assert_eq!(c.at(1, 1).unwrap(), 12.0);
    }

    #[test]
    fn test_sub() {
        let a = Matrix::from_vec(vec![10.0, 20.0, 30.0, 40.0], [2, 2]).unwrap();
        let b = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let c = a.sub(&b).unwrap();
        assert_eq!(c.at(0, 0).unwrap(), 9.0);
        assert_eq!(c.at(1, 1).unwrap(), 36.0);
    }

    #[test]
    fn test_div() {
        let a = Matrix::from_vec(vec![10.0, 20.0, 30.0, 40.0], [2, 2]).unwrap();
        let b = Matrix::from_vec(vec![2.0, 4.0, 5.0, 8.0], [2, 2]).unwrap();
        let c = a.div(&b).unwrap();
        assert_eq!(c.at(0, 0).unwrap(), 5.0);
        assert_eq!(c.at(1, 1).unwrap(), 5.0);
    }

    #[test]
    fn test_div_by_zero() {
        let a = Matrix::from_vec(vec![10.0, 20.0], [1, 2]).unwrap();
        let b = Matrix::from_vec(vec![0.0, 5.0], [1, 2]).unwrap();
        let c = a.div(&b).unwrap();
        assert_eq!(c.at(0, 0).unwrap(), 0.0); // safe: 0/0 -> 0
        assert_eq!(c.at(0, 1).unwrap(), 4.0);
    }

    #[test]
    fn test_scale() {
        let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let b = a.scale(3.0);
        assert_eq!(b.at(0, 0).unwrap(), 3.0);
        assert_eq!(b.at(1, 1).unwrap(), 12.0);
    }

    #[test]
    fn test_negate() {
        let a = Matrix::from_vec(vec![1.0, -2.0, 3.0, -4.0], [2, 2]).unwrap();
        let b = a.negate();
        assert_eq!(b.at(0, 0).unwrap(), -1.0);
        assert_eq!(b.at(0, 1).unwrap(), 2.0);
    }

    #[test]
    fn test_hadamard() {
        let a = Matrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], [2, 2]).unwrap();
        let b = Matrix::from_vec(vec![2.0, 3.0, 4.0, 5.0], [2, 2]).unwrap();
        let c = a.hadamard(&b).unwrap();
        assert_eq!(c.at(0, 0).unwrap(), 2.0);
        assert_eq!(c.at(1, 1).unwrap(), 20.0);
    }

    #[test]
    fn test_dimension_mismatch() {
        let a = Matrix::from_vec(vec![1.0, 2.0], [1, 2]).unwrap();
        let b = Matrix::from_vec(vec![1.0, 2.0, 3.0], [1, 3]).unwrap();
        assert!(a.matmul(&b).is_err());
    }
}
