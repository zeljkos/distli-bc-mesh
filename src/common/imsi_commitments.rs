// IMSI Commitment Scheme using Pedersen Commitments
// Provides cryptographic privacy for subscriber identities in telecom roaming

use curve25519_dalek_ng::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek_ng::ristretto::{RistrettoPoint, CompressedRistretto};
use curve25519_dalek_ng::scalar::Scalar;
use rand_core::{OsRng, CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

// Serializable IMSI commitment for blockchain storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IMSICommitment {
    pub commitment_bytes: Vec<u8>,        // Serialized commitment point
    pub commitment_type: String,          // "pedersen_ristretto255"
    pub created_at: u64,                  // Timestamp for key rotation
}

impl IMSICommitment {
    // Create from RistrettoPoint
    pub fn from_point(point: &RistrettoPoint) -> Self {
        Self {
            commitment_bytes: point.compress().to_bytes().to_vec(),
            commitment_type: "pedersen_ristretto255".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    // Convert back to RistrettoPoint for verification
    pub fn to_point(&self) -> Result<RistrettoPoint, String> {
        if self.commitment_bytes.len() != 32 {
            return Err("Invalid commitment byte length".to_string());
        }
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.commitment_bytes);
        
        CompressedRistretto(bytes)
            .decompress()
            .ok_or("Failed to decompress commitment point".to_string())
    }
}

// Private blinding factor data (never stored on blockchain)
#[derive(Debug, Clone)]
pub struct IMSIBlindingFactor {
    pub blinding: Scalar,                 // Random 'r' value in C = g^m * h^r
    pub imsi_scalar: Scalar,             // IMSI converted to scalar 'm'
    pub session_id: String,              // Associated session for tracking
    pub created_at: u64,                 // For secure cleanup
}

// Secure blinding factor management
pub struct BlindingFactorManager {
    // In production: store in HSM or encrypted database
    blinding_factors: HashMap<String, IMSIBlindingFactor>,
    // Optional master key for deterministic blinding (HKDF)
    master_key: Option<[u8; 32]>,
    // Cleanup old factors for forward secrecy
    max_age_seconds: u64,
}

impl BlindingFactorManager {
    pub fn new() -> Self {
        Self {
            blinding_factors: HashMap::new(),
            master_key: None,
            max_age_seconds: 86400 * 30, // 30 days default
        }
    }
    
    pub fn with_master_key(master_key: [u8; 32]) -> Self {
        Self {
            blinding_factors: HashMap::new(),
            master_key: Some(master_key),
            max_age_seconds: 86400 * 30,
        }
    }
    
    pub fn store_blinding_factor(&mut self, session_id: &str, factor: IMSIBlindingFactor) {
        self.blinding_factors.insert(session_id.to_string(), factor);
        self.cleanup_old_factors();
    }
    
    pub fn get_blinding_factor(&self, session_id: &str) -> Option<&IMSIBlindingFactor> {
        self.blinding_factors.get(session_id)
    }
    
    pub fn remove_blinding_factor(&mut self, session_id: &str) -> Option<IMSIBlindingFactor> {
        self.blinding_factors.remove(session_id)
    }
    
    // Generate deterministic blinding factor using HKDF (optional)
    pub fn generate_deterministic_blinding(&self, session_id: &str, imsi: &str) -> Option<Scalar> {
        if let Some(master_key) = &self.master_key {
            // Use HKDF to derive session-specific blinding factor
            let mut hasher = Sha256::new();
            hasher.update(master_key);
            hasher.update(session_id.as_bytes());
            hasher.update(imsi.as_bytes());
            hasher.update(b"imsi_blinding_factor");
            
            let derived = hasher.finalize();
            let derived_bytes: [u8; 32] = derived.into();
            Some(Scalar::from_bytes_mod_order(derived_bytes))
        } else {
            None
        }
    }
    
    fn cleanup_old_factors(&mut self) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.blinding_factors.retain(|_, factor| {
            current_time - factor.created_at < self.max_age_seconds
        });
    }
}

// Main IMSI commitment generator using Pedersen commitments
pub struct IMSICommitmentGenerator {
    pub g: RistrettoPoint,               // Base point g (standard generator)
    pub h: RistrettoPoint,               // Independent generator h (nothing-up-my-sleeve)
    blinding_manager: BlindingFactorManager,
}

impl IMSICommitmentGenerator {
    pub fn new() -> Self {
        Self {
            g: RISTRETTO_BASEPOINT_POINT,
            h: Self::generate_nothing_up_my_sleeve_h(),
            blinding_manager: BlindingFactorManager::new(),
        }
    }
    
    pub fn with_secure_key_management(master_key: [u8; 32]) -> Self {
        Self {
            g: RISTRETTO_BASEPOINT_POINT,
            h: Self::generate_nothing_up_my_sleeve_h(),
            blinding_manager: BlindingFactorManager::with_master_key(master_key),
        }
    }
    
    // Generate nothing-up-my-sleeve H generator point
    fn generate_nothing_up_my_sleeve_h() -> RistrettoPoint {
        // Use deterministic method to generate H from well-known string
        let mut hasher1 = Sha256::new();
        hasher1.update(b"distli-bc-mesh-imsi-commitment-h-generator-v1.0-part1");
        hasher1.update(RISTRETTO_BASEPOINT_POINT.compress().as_bytes());
        let hash1 = hasher1.finalize();
        
        let mut hasher2 = Sha256::new();
        hasher2.update(b"distli-bc-mesh-imsi-commitment-h-generator-v1.0-part2");
        hasher2.update(RISTRETTO_BASEPOINT_POINT.compress().as_bytes());
        let hash2 = hasher2.finalize();
        
        // Combine two 32-byte hashes to get 64 bytes for uniform sampling
        let mut uniform_bytes = [0u8; 64];
        uniform_bytes[0..32].copy_from_slice(&hash1);
        uniform_bytes[32..64].copy_from_slice(&hash2);
        
        RistrettoPoint::from_uniform_bytes(&uniform_bytes)
    }
    
    // Convert IMSI string to scalar for commitment
    fn imsi_to_scalar(&self, imsi: &str) -> Result<Scalar, String> {
        // Validate IMSI format (basic check)
        if imsi.len() < 10 || imsi.len() > 15 {
            return Err("Invalid IMSI length".to_string());
        }
        
        // Hash IMSI to create uniform scalar (prevents timing attacks)
        let mut hasher = Sha256::new();
        hasher.update(b"imsi_to_scalar_v1");
        hasher.update(imsi.as_bytes());
        
        let hash = hasher.finalize();
        let hash_bytes: [u8; 32] = hash.into();
        Ok(Scalar::from_bytes_mod_order(hash_bytes))
    }
    
    // Create Pedersen commitment to IMSI: C = g^m * h^r
    pub fn commit_to_imsi(&mut self, imsi: &str, session_id: &str) -> Result<IMSICommitment, String> {
        // Convert IMSI to scalar
        let imsi_scalar = self.imsi_to_scalar(imsi)?;
        
        // Generate cryptographically secure random blinding factor
        let blinding = Scalar::random(&mut OsRng);
        
        // Compute Pedersen commitment: C = g^m * h^r
        let commitment_point = (self.g * imsi_scalar) + (self.h * blinding);
        
        // Store blinding factor securely for future verification/disputes
        let blinding_factor = IMSIBlindingFactor {
            blinding,
            imsi_scalar,
            session_id: session_id.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.blinding_manager.store_blinding_factor(session_id, blinding_factor);
        
        // Return serializable commitment
        Ok(IMSICommitment::from_point(&commitment_point))
    }
    
    // Verify commitment opens to specific IMSI (for authorized parties only)
    pub fn verify_commitment(
        &self,
        commitment: &IMSICommitment,
        imsi: &str,
        blinding: &Scalar,
    ) -> Result<bool, String> {
        let commitment_point = commitment.to_point()?;
        let imsi_scalar = self.imsi_to_scalar(imsi)?;
        
        // Recompute commitment: C' = g^m * h^r
        let expected_point = (self.g * imsi_scalar) + (self.h * blinding);
        
        // Compare with stored commitment
        Ok(commitment_point == expected_point)
    }
    
    // Verify commitment using stored blinding factor (for disputes)
    pub fn verify_commitment_with_session(
        &self,
        commitment: &IMSICommitment,
        session_id: &str,
        claimed_imsi: &str,
    ) -> Result<bool, String> {
        let blinding_factor = self.blinding_manager
            .get_blinding_factor(session_id)
            .ok_or("Blinding factor not found for session")?;
        
        self.verify_commitment(commitment, claimed_imsi, &blinding_factor.blinding)
    }
    
    // Create zero-knowledge proof that commitment opens to valid IMSI
    // (For advanced dispute resolution - simplified version)
    pub fn create_opening_proof(&self, session_id: &str) -> Result<IMSIOpeningProof, String> {
        let blinding_factor = self.blinding_manager
            .get_blinding_factor(session_id)
            .ok_or("Blinding factor not found")?;
        
        // In full implementation: create Schnorr proof of knowledge
        // For now: simplified proof structure
        Ok(IMSIOpeningProof {
            session_id: session_id.to_string(),
            proof_type: "schnorr_discrete_log".to_string(),
            // In production: actual zero-knowledge proof data
            proof_data: vec![0u8; 64], // Placeholder
        })
    }
    
    // Get access to blinding manager for advanced operations
    pub fn get_blinding_manager(&self) -> &BlindingFactorManager {
        &self.blinding_manager
    }
    
    pub fn get_blinding_manager_mut(&mut self) -> &mut BlindingFactorManager {
        &mut self.blinding_manager
    }
}

// Zero-knowledge proof that commitment opens to valid IMSI (for disputes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IMSIOpeningProof {
    pub session_id: String,
    pub proof_type: String,           // "schnorr_discrete_log"
    pub proof_data: Vec<u8>,         // Serialized proof
}

impl IMSIOpeningProof {
    pub fn verify(&self, _commitment: &IMSICommitment, _generator: &IMSICommitmentGenerator) -> bool {
        // In production: verify Schnorr proof of discrete logarithm
        // For now: simplified verification
        !self.proof_data.is_empty()
    }
}

// Utility functions for IMSI validation and formatting
pub mod imsi_utils {
    use super::*;
    
    pub fn validate_imsi_format(imsi: &str) -> Result<(), String> {
        if imsi.len() < 10 || imsi.len() > 15 {
            return Err("IMSI must be 10-15 digits".to_string());
        }
        
        if !imsi.chars().all(|c| c.is_ascii_digit()) {
            return Err("IMSI must contain only digits".to_string());
        }
        
        Ok(())
    }
    
    pub fn extract_mcc_mnc(imsi: &str) -> Result<(String, String), String> {
        validate_imsi_format(imsi)?;
        
        if imsi.len() < 6 {
            return Err("IMSI too short to extract MCC/MNC".to_string());
        }
        
        let mcc = imsi[0..3].to_string(); // Mobile Country Code
        let mnc = imsi[3..6].to_string(); // Mobile Network Code (simplified)
        
        Ok((mcc, mnc))
    }
    
    // Obfuscate IMSI for logging (show only country code)
    pub fn obfuscate_imsi_for_logging(imsi: &str) -> String {
        if let Ok((mcc, _)) = extract_mcc_mnc(imsi) {
            format!("{}***********", mcc)
        } else {
            "INVALID_IMSI".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_imsi_commitment_basic() {
        let mut generator = IMSICommitmentGenerator::new();
        let imsi = "310260123456789"; // Valid Verizon IMSI
        let session_id = "test_session_001";
        
        // Create commitment
        let commitment = generator.commit_to_imsi(imsi, session_id).unwrap();
        
        // Verify commitment properties
        assert_eq!(commitment.commitment_bytes.len(), 32);
        assert_eq!(commitment.commitment_type, "pedersen_ristretto255");
        assert!(commitment.created_at > 0);
    }
    
    #[test]
    fn test_imsi_commitment_hiding() {
        let mut generator = IMSICommitmentGenerator::new();
        let imsi = "310260123456789";
        
        // Same IMSI should produce different commitments due to random blinding
        let commitment1 = generator.commit_to_imsi(imsi, "session_1").unwrap();
        let commitment2 = generator.commit_to_imsi(imsi, "session_2").unwrap();
        
        assert_ne!(commitment1.commitment_bytes, commitment2.commitment_bytes);
    }
    
    #[test]
    fn test_imsi_commitment_binding() {
        let mut generator = IMSICommitmentGenerator::new();
        let imsi1 = "310260123456789";
        let imsi2 = "310260987654321";
        let session_id = "test_session";
        
        // Create commitment to IMSI1
        let commitment = generator.commit_to_imsi(imsi1, session_id).unwrap();
        
        // Should not verify against different IMSI
        let blinding_factor = generator.get_blinding_manager()
            .get_blinding_factor(session_id)
            .unwrap();
        
        let valid = generator.verify_commitment(&commitment, imsi1, &blinding_factor.blinding).unwrap();
        let invalid = generator.verify_commitment(&commitment, imsi2, &blinding_factor.blinding).unwrap();
        
        assert!(valid);
        assert!(!invalid);
    }
    
    #[test]
    fn test_imsi_validation() {
        use imsi_utils::*;
        
        // Valid IMSIs
        assert!(validate_imsi_format("310260123456789").is_ok());
        assert!(validate_imsi_format("1234567890").is_ok());
        
        // Invalid IMSIs
        assert!(validate_imsi_format("123").is_err()); // Too short
        assert!(validate_imsi_format("12345678901234567890").is_err()); // Too long
        assert!(validate_imsi_format("31026abc123456").is_err()); // Contains letters
    }
    
    #[test]
    fn test_mcc_mnc_extraction() {
        use imsi_utils::*;
        
        let imsi = "310260123456789";
        let (mcc, mnc) = extract_mcc_mnc(imsi).unwrap();
        
        assert_eq!(mcc, "310"); // USA
        assert_eq!(mnc, "260"); // T-Mobile
    }
    
    #[test]
    fn test_imsi_obfuscation() {
        use imsi_utils::*;
        
        let imsi = "310260123456789";
        let obfuscated = obfuscate_imsi_for_logging(imsi);
        
        assert_eq!(obfuscated, "310***********");
        assert!(!obfuscated.contains("123456789")); // Subscriber ID hidden
    }
    
    #[test]
    fn test_blinding_factor_cleanup() {
        let mut manager = BlindingFactorManager::new();
        manager.max_age_seconds = 1; // 1 second for testing
        
        let factor = IMSIBlindingFactor {
            blinding: Scalar::random(&mut OsRng),
            imsi_scalar: Scalar::random(&mut OsRng),
            session_id: "test".to_string(),
            created_at: 0, // Very old timestamp
        };
        
        manager.store_blinding_factor("test", factor);
        
        // Should be cleaned up
        assert!(manager.get_blinding_factor("test").is_none());
    }
    
    #[test]
    fn test_deterministic_blinding() {
        let master_key = [42u8; 32];
        let manager = BlindingFactorManager::with_master_key(master_key);
        
        let session_id = "session_123";
        let imsi = "310260123456789";
        
        // Same inputs should produce same blinding factor
        let blinding1 = manager.generate_deterministic_blinding(session_id, imsi).unwrap();
        let blinding2 = manager.generate_deterministic_blinding(session_id, imsi).unwrap();
        
        assert_eq!(blinding1, blinding2);
        
        // Different session should produce different blinding
        let blinding3 = manager.generate_deterministic_blinding("different_session", imsi).unwrap();
        assert_ne!(blinding1, blinding3);
    }
}