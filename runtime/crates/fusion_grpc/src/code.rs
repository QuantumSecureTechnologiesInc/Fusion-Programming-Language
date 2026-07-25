//! gRPC status codes per the official specification.

use std::fmt;

/// gRPC status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GrpcCode {
    Ok = 0,
    Cancelled = 1,
    Unknown = 2,
    InvalidArgument = 3,
    DeadlineExceeded = 4,
    NotFound = 5,
    AlreadyExists = 6,
    PermissionDenied = 7,
    ResourceExhausted = 8,
    FailedPrecondition = 9,
    Aborted = 10,
    OutOfRange = 11,
    Unimplemented = 12,
    Internal = 13,
    Unavailable = 14,
    DataLoss = 15,
    Unauthenticated = 16,
}

impl GrpcCode {
    pub fn from_u32(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Ok),
            1 => Some(Self::Cancelled),
            2 => Some(Self::Unknown),
            3 => Some(Self::InvalidArgument),
            4 => Some(Self::DeadlineExceeded),
            5 => Some(Self::NotFound),
            6 => Some(Self::AlreadyExists),
            7 => Some(Self::PermissionDenied),
            8 => Some(Self::ResourceExhausted),
            9 => Some(Self::FailedPrecondition),
            10 => Some(Self::Aborted),
            11 => Some(Self::OutOfRange),
            12 => Some(Self::Unimplemented),
            13 => Some(Self::Internal),
            14 => Some(Self::Unavailable),
            15 => Some(Self::DataLoss),
            16 => Some(Self::Unauthenticated),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Ok => "The operation completed successfully",
            Self::Cancelled => "The operation was cancelled",
            Self::Unknown => "Unknown error",
            Self::InvalidArgument => "Invalid argument",
            Self::DeadlineExceeded => "Deadline expired before operation could complete",
            Self::NotFound => "Some requested entity was not found",
            Self::AlreadyExists => "Some entity that we attempted to create already exists",
            Self::PermissionDenied => "The caller does not have permission to execute the specified operation",
            Self::ResourceExhausted => "Some resource has been exhausted",
            Self::FailedPrecondition => "The system is not in a state required for the operation's execution",
            Self::Aborted => "The operation was aborted",
            Self::OutOfRange => "Operation was attempted past the valid range",
            Self::Unimplemented => "Operation is not implemented or not supported",
            Self::Internal => "Internal error",
            Self::Unavailable => "The service is currently unavailable",
            Self::DataLoss => "Unrecoverable data loss or corruption",
            Self::Unauthenticated => "The request does not have valid authentication credentials",
        }
    }

    pub fn is_ok(&self) -> bool {
        *self == Self::Ok
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Unavailable | Self::ResourceExhausted | Self::Aborted | Self::Internal
        )
    }
}

impl fmt::Display for GrpcCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Ok => "OK",
            Self::Cancelled => "CANCELLED",
            Self::Unknown => "UNKNOWN",
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::DeadlineExceeded => "DEADLINE_EXCEEDED",
            Self::NotFound => "NOT_FOUND",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::FailedPrecondition => "FAILED_PRECONDITION",
            Self::Aborted => "ABORTED",
            Self::OutOfRange => "OUT_OF_RANGE",
            Self::Unimplemented => "UNIMPLEMENTED",
            Self::Internal => "INTERNAL",
            Self::Unavailable => "UNAVAILABLE",
            Self::DataLoss => "DATA_LOSS",
            Self::Unauthenticated => "UNAUTHENTICATED",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_code_roundtrip() {
        for code_val in 0u32..=16 {
            let code = GrpcCode::from_u32(code_val).unwrap();
            assert_eq!(code as u32, code_val);
        }
    }

    #[test]
    fn test_grpc_code_retryable() {
        assert!(GrpcCode::Unavailable.is_retryable());
        assert!(GrpcCode::ResourceExhausted.is_retryable());
        assert!(!GrpcCode::InvalidArgument.is_retryable());
        assert!(!GrpcCode::Ok.is_retryable());
    }

    #[test]
    fn test_grpc_code_display() {
        assert_eq!(format!("{}", GrpcCode::Ok), "OK");
        assert_eq!(format!("{}", GrpcCode::NotFound), "NOT_FOUND");
    }
}
