use crate::protocol::{DataFrame, Frame, MetadataFrame};
use raptorq::{Encoder, EncodingPacket, ObjectTransmissionInformation};
use sha2::{Digest, Sha256};
use std::io::Write;

pub struct QrEncoder {
    metadata: MetadataFrame,
    encoder: Encoder,
    current_symbol: u32,
}

impl QrEncoder {
    pub fn new(
        data: &[u8],
        filename: String,
        symbol_size: u16,
    ) -> Result<Self, String> {
        let uncompressed_size = data.len() as u64;
        
        // Zstd compression
        let mut compressed_data = Vec::new();
        let mut zstd_encoder = zstd::stream::Encoder::new(&mut compressed_data, 3)
            .map_err(|e| e.to_string())?;
        zstd_encoder.write_all(data).map_err(|e| e.to_string())?;
        zstd_encoder.finish().map_err(|e| e.to_string())?;
        
        let compressed_size = compressed_data.len() as u64;
        
        // SHA-256
        let mut hasher = Sha256::new();
        hasher.update(data);
        let sha256_hash: [u8; 32] = hasher.finalize().into();
        
        // RaptorQ Encoding
        let rq_encoder = Encoder::with_defaults(&compressed_data, symbol_size);
        let oti = rq_encoder.get_config();
        
        let metadata = MetadataFrame {
            version: 1,
            transfer_id: 12345, // In real app, random
            uncompressed_size,
            compressed_size,
            compression_type: 1, // Zstd
            total_symbols: (oti.transfer_length() / (symbol_size as u64)) as u32,
            symbol_size,
            sha256_hash,
            filename,
        };
        
        Ok(Self {
            metadata,
            encoder: rq_encoder,
            current_symbol: 0,
        })
    }
    
    pub fn get_metadata_frame(&self) -> Frame {
        Frame::Metadata(self.metadata.clone())
    }
    
    pub fn next_data_frame(&mut self) -> Frame {
        // RaptorQ generates an endless stream of symbols if needed, 
        // we just request the next one based on an incrementing ID.
        let packet = self.encoder.get_encoded_packets(self.current_symbol)[0].clone();
        
        let frame = Frame::Data(DataFrame {
            transfer_id: self.metadata.transfer_id,
            encoding_symbol_id: packet.payload_id().encoding_symbol_id(),
            payload: packet.data().to_vec(),
        });
        
        self.current_symbol += 1;
        frame
    }
}
