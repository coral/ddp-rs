//! Allocation-free DDP frame construction.
//!
//! [`FrameBuilder`] contains the chunking and sequence-numbering logic for sending pixel
//! data, decoupled from any transport. It writes each ready-to-send frame into a
//! caller-provided scratch buffer and hands it to a closure, so it works identically on
//! `std` (backing [`crate::connection::DDPConnection`]) and on bare-metal `no_std` targets
//! where you supply your own UDP stack.

use super::{Header, PixelConfig, ID};

/// Maximum pixel data size per DDP packet (480 pixels × 3 bytes RGB = 1440 bytes).
///
/// A frame's scratch buffer must hold a header plus this many payload bytes; 1500 (one MTU)
/// is always sufficient.
pub const MAX_DATA_LENGTH: usize = 480 * 3;

/// Builds DDP frames from a pixel buffer, splitting large buffers across multiple packets
/// and managing the rolling sequence number.
///
/// # Examples
///
/// ```
/// use ddp_rs::protocol::{FrameBuilder, PixelConfig, ID};
///
/// let mut builder = FrameBuilder::new(PixelConfig::default(), ID::Default);
/// let mut scratch = [0u8; 1500];
///
/// // 2 RGB pixels. The closure receives each fully-assembled frame ready to send.
/// builder
///     .for_each_frame(&[255, 0, 0, 0, 0, 255], 0, &mut scratch, |frame| {
///         // e.g. socket.send(frame) on your platform
///         assert_eq!(frame.len(), 10 + 6);
///         Ok::<(), ()>(())
///     })
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct FrameBuilder {
    /// Pixel format configuration written into each frame header.
    pub pixel_config: PixelConfig,
    /// Protocol ID written into each frame header.
    pub id: ID,
    sequence_number: u8,
}

impl FrameBuilder {
    /// Creates a new frame builder. The sequence number starts at 1.
    pub fn new(pixel_config: PixelConfig, id: ID) -> Self {
        Self {
            pixel_config,
            id,
            sequence_number: 1,
        }
    }

    /// The sequence number that will be used for the next frame.
    pub fn sequence_number(&self) -> u8 {
        self.sequence_number
    }

    /// Splits `data` into DDP frames and invokes `f` with each fully-assembled frame.
    ///
    /// Each frame is written into `scratch` (which must be at least
    /// `header_len + MAX_DATA_LENGTH`, i.e. ≥ 1500 bytes) and the resulting slice is passed
    /// to `f`, which performs the actual transmission. `offset` is the starting byte offset
    /// into the display buffer (not a pixel index); subsequent chunks advance it
    /// automatically. The push flag is set on the final frame.
    ///
    /// Uses this builder's configured [`pixel_config`](Self::pixel_config) and
    /// [`id`](Self::id). For control messages with a custom id, use [`Self::frames_with`].
    pub fn for_each_frame<F, E>(
        &mut self,
        data: &[u8],
        offset: u32,
        scratch: &mut [u8],
        f: F,
    ) -> Result<(), E>
    where
        F: FnMut(&[u8]) -> Result<(), E>,
    {
        let header = Header {
            pixel_config: self.pixel_config,
            id: self.id,
            ..Default::default()
        };
        self.frames_with(header, data, offset, scratch, f)
    }

    /// Like [`Self::for_each_frame`] but using a caller-supplied header template.
    ///
    /// The template's `packet_type`, `pixel_config`, `id` and `time_code` are used as-is;
    /// the `offset`, `length`, `sequence_number` and push flag are managed per chunk.
    pub fn frames_with<F, E>(
        &mut self,
        mut header: Header,
        data: &[u8],
        offset: u32,
        scratch: &mut [u8],
        mut f: F,
    ) -> Result<(), E>
    where
        F: FnMut(&[u8]) -> Result<(), E>,
    {
        header.packet_type.push(false);

        let total = data.len();
        let num_iterations = total.div_ceil(MAX_DATA_LENGTH);
        let mut chunk_index = 0usize;
        let mut data_offset = 0usize;

        while data_offset < total {
            chunk_index += 1;

            // Mark the final chunk with the push flag.
            if chunk_index == num_iterations {
                header.packet_type.push(true);
            }

            header.sequence_number = self.sequence_number;

            let chunk_end = core::cmp::min(data_offset + MAX_DATA_LENGTH, total);
            let chunk = &data[data_offset..chunk_end];
            header.length = chunk.len() as u16;
            header.offset = offset + data_offset as u32;

            let header_len = header.write_into(scratch);
            scratch[header_len..header_len + chunk.len()].copy_from_slice(chunk);
            f(&scratch[..header_len + chunk.len()])?;

            // Sequence number wraps around back to 1.
            if self.sequence_number > 15 {
                self.sequence_number = 1;
            } else {
                self.sequence_number += 1;
            }

            data_offset += MAX_DATA_LENGTH;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::PixelConfig;

    #[test]
    fn single_frame_matches_expected_bytes() {
        let mut builder = FrameBuilder::new(PixelConfig::default(), ID::Default);
        let mut scratch = [0u8; 1500];
        let mut frames: Vec<Vec<u8>> = Vec::new();

        builder
            .for_each_frame(&[255, 0, 0, 255, 0, 0, 255, 0, 0], 0, &mut scratch, |frame| {
                frames.push(frame.to_vec());
                Ok::<(), ()>(())
            })
            .unwrap();

        assert_eq!(frames.len(), 1);
        // Byte-for-byte identical to what DDPConnection emits (see connection.rs test_conn).
        assert_eq!(
            frames[0],
            vec![
                0x41, 0x01, 0x0D, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0xFF, 0x00, 0x00,
                0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn chunks_large_data_and_sets_push_on_last() {
        let mut builder = FrameBuilder::new(PixelConfig::default(), ID::Default);
        let mut scratch = [0u8; 1500];
        let data = vec![7u8; MAX_DATA_LENGTH * 2 + 30];
        let mut pushes: Vec<bool> = Vec::new();
        let mut seqs: Vec<u8> = Vec::new();

        builder
            .for_each_frame(&data, 0, &mut scratch, |frame| {
                let h = Header::from(frame);
                pushes.push(h.packet_type.push);
                seqs.push(h.sequence_number);
                Ok::<(), ()>(())
            })
            .unwrap();

        assert_eq!(pushes, vec![false, false, true]);
        assert_eq!(seqs, vec![1, 2, 3]);
    }

    #[test]
    fn frame_roundtrips_through_packet_ref() {
        use crate::packet::PacketRef;

        let mut builder = FrameBuilder::new(PixelConfig::default(), ID::Custom(42));
        let mut scratch = [0u8; 1500];
        let payload: Vec<u8> = (0..90u8).collect();

        builder
            .for_each_frame(&payload, 30, &mut scratch, |frame| {
                let parsed = PacketRef::from_bytes(frame).unwrap();
                assert_eq!(parsed.header.offset, 30);
                assert_eq!(parsed.header.id, ID::Custom(42));
                assert_eq!(parsed.header.length as usize, payload.len());
                assert_eq!(parsed.data, &payload[..]);
                Ok::<(), ()>(())
            })
            .unwrap();
    }

    #[test]
    fn empty_data_produces_no_frames() {
        let mut builder = FrameBuilder::new(PixelConfig::default(), ID::Default);
        let mut scratch = [0u8; 1500];
        let mut count = 0;
        builder
            .for_each_frame(&[], 0, &mut scratch, |_| {
                count += 1;
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(count, 0);
    }
}
