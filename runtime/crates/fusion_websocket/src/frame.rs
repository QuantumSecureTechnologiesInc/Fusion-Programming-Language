//! WebSocket frame encoding and decoding per RFC 6455.

use crate::error::{Result, WsError};
use crate::opcodes::Opcode;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rand::RngCore;
use std::fmt;

/// WebSocket frame header.
#[derive(Debug, Clone, Copy)]
pub struct FrameHeader {
    pub fin: bool,
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub opcode: Opcode,
    pub masked: bool,
    pub payload_length: u64,
    pub mask_key: Option<[u8; 4]>,
}

impl FrameHeader {
    /// Minimum header size (2 bytes for the first two octets).
    pub const MIN_SIZE: usize = 2;

    /// Parse a frame header from a byte buffer.
    pub fn parse(buf: &mut BytesMut) -> Result<Option<Self>> {
        if buf.len() < Self::MIN_SIZE {
            return Ok(None);
        }

        let first = buf[0];
        let second = buf[1];

        let fin = (first & 0x80) != 0;
        let rsv1 = (first & 0x40) != 0;
        let rsv2 = (first & 0x20) != 0;
        let rsv3 = (first & 0x10) != 0;
        let opcode = Opcode::from_u8(first & 0x0F)
            .ok_or_else(|| WsError::InvalidOpcode(first & 0x0F))?;

        let masked = (second & 0x80) != 0;
        let mut payload_length = (second & 0x7F) as u64;

        let mut consumed = 2;

        // Extended payload length
        if payload_length == 126 {
            if buf.len() < 4 {
                return Ok(None);
            }
            payload_length = u16::from_be_bytes([buf[2], buf[3]]) as u64;
            consumed = 4;
        } else if payload_length == 127 {
            if buf.len() < 10 {
                return Ok(None);
            }
            payload_length = u64::from_be_bytes([
                buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
            ]);
            consumed = 10;
        }

        // Mask key
        let mask_key = if masked {
            let mask_start = consumed;
            if buf.len() < mask_start + 4 {
                return Ok(None);
            }
            let mut key = [0u8; 4];
            key.copy_from_slice(&buf[mask_start..mask_start + 4]);
            consumed += 4;
            Some(key)
        } else {
            None
        };

        Ok(Some(FrameHeader {
            fin,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            masked,
            payload_length,
            mask_key,
        }))
    }

    /// Total header size (fixed + extended length + mask key).
    pub fn header_size(&self) -> usize {
        let mut size = 2;
        if self.payload_length == 126 {
            size += 2;
        } else if self.payload_length == 127 {
            size += 8;
        }
        if self.masked {
            size += 4;
        }
        size
    }

    /// Serialize the header to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        let mut first = 0u8;
        if self.fin {
            first |= 0x80;
        }
        if self.rsv1 {
            first |= 0x40;
        }
        if self.rsv2 {
            first |= 0x20;
        }
        if self.rsv3 {
            first |= 0x10;
        }
        first |= self.opcode as u8;
        buf.push(first);

        let mut second = 0u8;
        if self.masked {
            second |= 0x80;
        }

        if self.payload_length < 126 {
            second |= self.payload_length as u8;
            buf.push(second);
        } else if self.payload_length <= 0xFFFF {
            second |= 126;
            buf.push(second);
            buf.extend_from_slice(&(self.payload_length as u16).to_be_bytes());
        } else {
            second |= 127;
            buf.push(second);
            buf.extend_from_slice(&self.payload_length.to_be_bytes());
        }

        if let Some(key) = self.mask_key {
            buf.extend_from_slice(&key);
        }

        buf
    }
}

/// A complete WebSocket frame.
#[derive(Debug, Clone)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Bytes,
}

impl Frame {
    /// Create a new frame.
    pub fn new(opcode: Opcode, payload: &[u8]) -> Self {
        Self {
            header: FrameHeader {
                fin: true,
                rsv1: false,
                rsv2: false,
                rsv3: false,
                opcode,
                masked: false,
                payload_length: payload.len() as u64,
                mask_key: None,
            },
            payload: Bytes::copy_from_slice(payload),
        }
    }

    /// Create a masked frame (client-to-server).
    pub fn masked(opcode: Opcode, payload: &[u8]) -> Self {
        let mut mask_key = [0u8; 4];
        rand::thread_rng().fill_bytes(&mut mask_key);

        let masked_payload = Self::apply_mask(payload, mask_key);

        Self {
            header: FrameHeader {
                fin: true,
                rsv1: false,
                rsv2: false,
                rsv3: false,
                opcode,
                masked: true,
                payload_length: payload.len() as u64,
                mask_key: Some(mask_key),
            },
            payload: Bytes::from(masked_payload),
        }
    }

    /// Parse a complete frame from a buffer.
    pub fn parse(buf: &mut BytesMut) -> Result<Option<Self>> {
        let header = match FrameHeader::parse(buf)? {
            Some(h) => h,
            None => return Ok(None),
        };

        let total_size = header.header_size() + header.payload_length as usize;
        if buf.len() < total_size {
            return Ok(None);
        }

        // Skip past the header
        let header_size = header.header_size();
        buf.advance(header_size);

        let payload = buf.split_to(header.payload_length as usize);

        let mut data = payload.to_vec();

        // Unmask if needed
        if let Some(mask_key) = header.mask_key {
            data = Self::apply_mask(&data, mask_key);
        }

        Ok(Some(Frame {
            header,
            payload: Bytes::from(data),
        }))
    }

    /// Create a close frame with status code and reason.
    pub fn close(code: u16, reason: &str) -> Self {
        let mut payload = Vec::new();
        payload.extend_from_slice(&code.to_be_bytes());
        payload.extend_from_slice(reason.as_bytes());
        Self::new(Opcode::Close, &payload)
    }

    /// Create a ping frame.
    pub fn ping(data: &[u8]) -> Self {
        Self::new(Opcode::Ping, data)
    }

    /// Create a pong frame.
    pub fn pong(data: &[u8]) -> Self {
        Self::new(Opcode::Pong, data)
    }

    /// Create a text frame.
    pub fn text(text: &str) -> Self {
        Self::new(Opcode::Text, text.as_bytes())
    }

    /// Create a binary frame.
    pub fn binary(data: &[u8]) -> Self {
        Self::new(Opcode::Binary, data)
    }

    /// Serialize the frame to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = self.header.serialize();
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Apply XOR masking to payload data.
    fn apply_mask(data: &[u8], mask: [u8; 4]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &b)| b ^ mask[i % 4])
            .collect()
    }

    pub fn is_final(&self) -> bool {
        self.header.fin
    }

    pub fn opcode(&self) -> Opcode {
        self.header.opcode
    }

    pub fn payload_bytes(&self) -> &[u8] {
        &self.payload
    }

    pub fn payload_text(&self) -> Result<&str> {
        std::str::from_utf8(&self.payload)
            .map_err(|e| WsError::Protocol(format!("Invalid UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_text_roundtrip() {
        let frame = Frame::text("hello");
        let bytes = frame.serialize();
        let mut buf = BytesMut::from(&bytes[..]);
        let parsed = Frame::parse(&mut buf).unwrap().unwrap();
        assert_eq!(parsed.opcode(), Opcode::Text);
        assert_eq!(parsed.payload_text().unwrap(), "hello");
    }

    #[test]
    fn test_frame_binary_roundtrip() {
        let frame = Frame::binary(&[1, 2, 3, 4]);
        let bytes = frame.serialize();
        let mut buf = BytesMut::from(&bytes[..]);
        let parsed = Frame::parse(&mut buf).unwrap().unwrap();
        assert_eq!(parsed.opcode(), Opcode::Binary);
        assert_eq!(parsed.payload_bytes(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_close_frame() {
        let frame = Frame::close(1000, "normal");
        assert_eq!(frame.opcode(), Opcode::Close);
        assert_eq!(frame.payload_bytes()[0..2], [0x03, 0xE8]); // 1000
    }

    #[test]
    fn test_ping_pong() {
        let ping = Frame::ping(b"test");
        assert_eq!(ping.opcode(), Opcode::Ping);

        let pong = Frame::pong(b"test");
        assert_eq!(pong.opcode(), Opcode::Pong);
    }

    #[test]
    fn test_masked_frame() {
        let frame = Frame::masked(Opcode::Text, b"hello");
        assert!(frame.header.masked);
        assert!(frame.header.mask_key.is_some());
    }

    #[test]
    fn test_large_payload() {
        let data = vec![0u8; 70000]; // > 65535
        let frame = Frame::binary(&data);
        let bytes = frame.serialize();
        let mut buf = BytesMut::from(&bytes[..]);
        let parsed = Frame::parse(&mut buf).unwrap().unwrap();
        assert_eq!(parsed.payload_bytes().len(), 70000);
    }
}
