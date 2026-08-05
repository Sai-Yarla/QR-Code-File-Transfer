use crate::protocol::{Frame, MetadataFrame};
use raptorq::{Decoder, EncodingPacket, ObjectTransmissionInformation};
use sha2::{Digest, Sha256};
use std::io::Read;

pub struct QrDecoder {
    metadata: Option<MetadataFrame>,
    decoder: Option<Decoder>,
    received_symbols: usize,
}

impl QrDecoder {
    pub fn new() -> Self {
        Self {
            metadata: None,
            decoder: None,
            received_symbols: 0,
        }
    }

    pub fn process_frame(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        let frame = Frame::decode(data)?;

        match frame {
            Frame::Metadata(m) => {
                if self.metadata.is_none() {
                    // Initialize Decoder
                    let oti = ObjectTransmissionInformation::new(
                        m.compressed_size,
                        m.symbol_size,
                        1, 1, 8 // basic configuration, should match encoder exactly in a real app
                    );
                    self.decoder = Some(Decoder::new(oti));
                    self.metadata = Some(m);
                }
                None
            }
            Frame::Data(d) => {
                if let Some(ref mut decoder) = self.decoder {
                    let packet = EncodingPacket::new(d.encoding_symbol_id, d.payload);
                    self.received_symbols += 1;
                    
                    if let Some(decoded_data) = decoder.decode(packet) {
                        // Decompress
                        let mut decompressed = Vec::new();
                        let mut zstd_decoder = zstd::stream::Decoder::new(&decoded_data[..])
                            .expect("Failed to initialize Zstd decoder");
                        zstd_decoder.read_to_end(&mut decompressed).ok()?;

                        // Verify Hash
                        let mut hasher = Sha256::new();
                        hasher.update(&decompressed);
                        let sha256_hash: [u8; 32] = hasher.finalize().into();

                        if let Some(ref meta) = self.metadata {
                            if sha256_hash == meta.sha256_hash {
                                return Some(decompressed);
                            }
                        }
                    }
                }
                None
            }
        }
    }

    pub fn progress(&self) -> f32 {
        if let Some(ref meta) = self.metadata {
            if meta.total_symbols == 0 {
                return 0.0;
            }
            (self.received_symbols as f32 / meta.total_symbols as f32).min(1.0)
        } else {
            0.0
        }
    }
}
