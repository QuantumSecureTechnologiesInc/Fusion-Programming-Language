//! Stream-oriented codec layer for gRPC streaming RPCs.

use crate::codec::{encode_message, decode_message, Codec};
use bytes::{Bytes, BytesMut};
use crate::GrpcError;

/// A framed stream encoder for gRPC messages.
pub struct StreamEncoder<C: Codec> {
    codec: C,
}

impl<C: Codec> StreamEncoder<C> {
    pub fn new(codec: C) -> Self {
        Self { codec }
    }

    pub fn encode(&self, msg: &C::Encode) -> Result<Bytes, GrpcError> {
        let serialized = self.codec.encode(msg)?;
        Ok(encode_message(&serialized, false))
    }
}

/// A framed stream decoder for gRPC messages.
pub struct StreamDecoder<C: Codec> {
    codec: C,
    buffer: BytesMut,
}

impl<C: Codec> StreamDecoder<C> {
    pub fn new(codec: C) -> Self {
        Self {
            codec,
            buffer: BytesMut::new(),
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn decode_next(&mut self) -> Result<Option<C::Decode>, GrpcError> {
        match decode_message(&mut self.buffer)? {
            Some(data) => {
                let msg = self.codec.decode(data)?;
                Ok(Some(msg))
            }
            None => Ok(None),
        }
    }

    pub fn has_pending(&self) -> bool {
        !self.buffer.is_empty()
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

/// Re-export the raw codec for stream usage.
pub use crate::codec::RawCodec;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::JsonCodec;

    #[test]
    fn test_stream_encoder_decoder() {
        let encoder = StreamEncoder::new(RawCodec);
        let original = Bytes::from("test message");
        let encoded = encoder.encode(&original).unwrap();

        let mut decoder = StreamDecoder::new(RawCodec);
        decoder.feed(&encoded);
        let decoded = decoder.decode_next().unwrap().unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_stream_decoder_partial() {
        let mut decoder = StreamDecoder::new(RawCodec);

        // Feed only the header
        let encoded = encode_message(b"hello", false);
        decoder.feed(&encoded[..3]);
        assert!(decoder.decode_next().unwrap().is_none());
        assert!(decoder.has_pending());

        // Feed the rest
        decoder.feed(&encoded[3..]);
        let decoded = decoder.decode_next().unwrap().unwrap();
        assert_eq!(&decoded[..], b"hello");
    }

    #[test]
    fn test_json_stream_codec() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct Msg {
            text: String,
        }

        let codec = JsonCodec::<Msg>::new();
        let encoder = StreamEncoder::new(codec);
        let msg = Msg {
            text: "hello".to_string(),
        };
        let encoded = encoder.encode(&msg).unwrap();

        let mut decoder = StreamDecoder::new(JsonCodec::<Msg>::new());
        decoder.feed(&encoded);
        let decoded: Msg = decoder.decode_next().unwrap().unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_multiple_messages_in_buffer() {
        let encoder = StreamEncoder::new(RawCodec);
        let mut combined = Vec::new();
        combined.extend_from_slice(&encoder.encode(&Bytes::from("msg1")).unwrap());
        combined.extend_from_slice(&encoder.encode(&Bytes::from("msg2")).unwrap());

        let mut decoder = StreamDecoder::new(RawCodec);
        decoder.feed(&combined);

        let m1 = decoder.decode_next().unwrap().unwrap();
        assert_eq!(&m1[..], b"msg1");

        let m2 = decoder.decode_next().unwrap().unwrap();
        assert_eq!(&m2[..], b"msg2");

        assert!(decoder.decode_next().unwrap().is_none());
    }
}
