//! Codec definitions for gRPC message serialization/deserialization.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::GrpcError;

/// Length-prefixed message framing for gRPC wire format.
pub const HEADER_SIZE: usize = 5; // 1 byte compressed flag + 4 bytes length

/// Encode a message with gRPC length-prefix framing.
pub fn encode_message(data: &[u8], compressed: bool) -> Bytes {
    let mut buf = BytesMut::with_capacity(HEADER_SIZE + data.len());
    buf.put_u8(if compressed { 1 } else { 0 });
    buf.put_u32(data.len() as u32);
    buf.extend_from_slice(data);
    buf.freeze()
}

/// Decode a gRPC length-prefixed message from a buffer.
/// Returns `None` if there are not enough bytes in the buffer.
pub fn decode_message(buf: &mut BytesMut) -> Result<Option<Bytes>, GrpcError> {
    if buf.len() < HEADER_SIZE {
        return Ok(None);
    }

    let compressed = buf[0] != 0;
    let length = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;

    if compressed {
        return Err(GrpcError::Codec(
            "Compressed messages not supported".to_string(),
        ));
    }

    if buf.len() < HEADER_SIZE + length {
        return Ok(None);
    }

    buf.advance(HEADER_SIZE);
    let data = buf.split_to(length).freeze();
    Ok(Some(data))
}

/// Codec trait for serializing/deserializing gRPC messages.
pub trait Codec: Send + Sync + 'static {
    type Encode: Send + Sync;
    type Decode: Send + Sync;

    fn encode(&self, msg: &Self::Encode) -> Result<Bytes, GrpcError>;
    fn decode(&self, buf: Bytes) -> Result<Self::Decode, GrpcError>;
    fn name(&self) -> &str;
}

/// JSON codec for simple message types that implement Serialize/Deserialize.
pub struct JsonCodec<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> JsonCodec<T> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Default for JsonCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Codec for JsonCodec<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    type Encode = T;
    type Decode = T;

    fn encode(&self, msg: &T) -> Result<Bytes, GrpcError> {
        let data = serde_json::to_vec(msg)
            .map_err(|e| GrpcError::Codec(format!("JSON encode error: {}", e)))?;
        Ok(Bytes::from(data))
    }

    fn decode(&self, buf: Bytes) -> Result<T, GrpcError> {
        serde_json::from_slice(&buf)
            .map_err(|e| GrpcError::Codec(format!("JSON decode error: {}", e)))
    }

    fn name(&self) -> &str {
        "json"
    }
}

/// Raw bytes codec — no serialization, just passes through.
pub struct RawCodec;

impl Codec for RawCodec {
    type Encode = Bytes;
    type Decode = Bytes;

    fn encode(&self, msg: &Bytes) -> Result<Bytes, GrpcError> {
        Ok(msg.clone())
    }

    fn decode(&self, buf: Bytes) -> Result<Bytes, GrpcError> {
        Ok(buf)
    }

    fn name(&self) -> &str {
        "raw"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = b"hello gRPC";
        let encoded = encode_message(original, false);
        assert_eq!(encoded[0], 0); // not compressed

        let mut buf = BytesMut::from(&encoded[..]);
        let decoded = decode_message(&mut buf).unwrap().unwrap();
        assert_eq!(&decoded[..], original);
    }

    #[test]
    fn test_decode_incomplete() {
        let mut buf = BytesMut::from(&[0u8; 3][..]);
        assert!(decode_message(&mut buf).unwrap().is_none());
    }

    #[test]
    fn test_json_codec() {
        let codec = JsonCodec::<serde_json::Value>::new();
        let msg = serde_json::json!({"key": "value"});
        let encoded = codec.encode(&msg).unwrap();
        let decoded: serde_json::Value = codec.decode(encoded).unwrap();
        assert_eq!(decoded["key"], "value");
    }
}
