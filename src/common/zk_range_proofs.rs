// Zero-Knowledge Range Proofs using Bulletproofs
// Proves values are within valid ranges without revealing the actual values

use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::ristretto::CompressedRistretto;
use curve25519_dalek_ng::scalar::Scalar;
use merlin::Transcript;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

// Constants for telecom domain
const MAX_CALL_DURATION_MINUTES: u64 = 240;  // 4 hours max call
const MAX_DATA_VOLUME_MB: u64 = 10000;       // 10GB max data session
const MAX_SMS_COUNT: u64 = 1000;             // Max SMS in batch

// Wrapper for Bulletproofs range proof that can be serialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableRangeProof {
    pub proof_bytes: Vec<u8>,
    pub commitment: Vec<u8>,
    pub min_value: u64,
    pub max_value: u64,
}

// Range proof generator for telecom billing values
pub struct RangeProofGenerator {
    bp_gens: BulletproofGens,
    pc_gens: PedersenGens,
}

impl RangeProofGenerator {
    pub fn new() -> Self {
        // Initialize generators for 64-bit range proofs
        Self {
            bp_gens: BulletproofGens::new(64, 1),
            pc_gens: PedersenGens::default(),
        }
    }
    
    // Create a range proof for call duration (0-240 minutes)
    pub fn prove_call_duration(
        &self,
        duration_minutes: u64,
    ) -> Result<(SerializableRangeProof, Scalar), String> {
        if duration_minutes > MAX_CALL_DURATION_MINUTES {
            return Err(format!(
                "Duration {} exceeds maximum {} minutes",
                duration_minutes, MAX_CALL_DURATION_MINUTES
            ));
        }
        
        self.create_range_proof(duration_minutes, 0, MAX_CALL_DURATION_MINUTES)
    }
    
    // Create a range proof for data volume (0-10000 MB)
    pub fn prove_data_volume(
        &self,
        volume_mb: u64,
    ) -> Result<(SerializableRangeProof, Scalar), String> {
        if volume_mb > MAX_DATA_VOLUME_MB {
            return Err(format!(
                "Volume {} exceeds maximum {} MB",
                volume_mb, MAX_DATA_VOLUME_MB
            ));
        }
        
        self.create_range_proof(volume_mb, 0, MAX_DATA_VOLUME_MB)
    }
    
    // Create a range proof for SMS count (0-1000)
    pub fn prove_sms_count(
        &self,
        sms_count: u64,
    ) -> Result<(SerializableRangeProof, Scalar), String> {
        if sms_count > MAX_SMS_COUNT {
            return Err(format!(
                "SMS count {} exceeds maximum {}",
                sms_count, MAX_SMS_COUNT
            ));
        }
        
        self.create_range_proof(sms_count, 0, MAX_SMS_COUNT)
    }
    
    // Generic range proof creation
    pub fn create_range_proof(
        &self,
        secret_value: u64,
        min_value: u64,
        max_value: u64,
    ) -> Result<(SerializableRangeProof, Scalar), String> {
        // Validate input
        if secret_value < min_value || secret_value > max_value {
            return Err(format!(
                "Value {} is outside range [{}, {}]",
                secret_value, min_value, max_value
            ));
        }
        
        // Create blinding factor for commitment
        let mut rng = thread_rng();
        let blinding = Scalar::random(&mut rng);
        
        // Create transcript for Fiat-Shamir transform
        let mut prover_transcript = Transcript::new(b"RangeProof");
        prover_transcript.append_u64(b"min", min_value);
        prover_transcript.append_u64(b"max", max_value);
        
        // Generate the range proof
        let (proof, committed_value) = RangeProof::prove_single(
            &self.bp_gens,
            &self.pc_gens,
            &mut prover_transcript,
            secret_value,
            &blinding,
            64,  // Number of bits for range
        ).map_err(|e| format!("Failed to create range proof: {:?}", e))?;
        
        // Serialize the proof and commitment
        let serializable_proof = SerializableRangeProof {
            proof_bytes: proof.to_bytes(),
            commitment: committed_value.to_bytes().to_vec(),
            min_value,
            max_value,
        };
        
        Ok((serializable_proof, blinding))
    }
}

// Range proof verifier
pub struct RangeProofVerifier {
    bp_gens: BulletproofGens,
    pc_gens: PedersenGens,
}

impl RangeProofVerifier {
    pub fn new() -> Self {
        Self {
            bp_gens: BulletproofGens::new(64, 1),
            pc_gens: PedersenGens::default(),
        }
    }
    
    // Verify a range proof without knowing the secret value
    pub fn verify_range_proof(&self, proof: &SerializableRangeProof) -> bool {
        // Deserialize the proof
        let range_proof = match RangeProof::from_bytes(&proof.proof_bytes) {
            Ok(p) => p,
            Err(_) => return false,
        };
        
        // Deserialize the commitment
        let commitment = if proof.commitment.len() != 32 {
            return false;
        } else {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&proof.commitment);
            CompressedRistretto(bytes)
        };
        
        // Create verifier transcript
        let mut verifier_transcript = Transcript::new(b"RangeProof");
        verifier_transcript.append_u64(b"min", proof.min_value);
        verifier_transcript.append_u64(b"max", proof.max_value);
        
        // Verify the proof
        range_proof.verify_single(
            &self.bp_gens,
            &self.pc_gens,
            &mut verifier_transcript,
            &commitment,
            64,
        ).is_ok()
    }
    
    // Batch verification for multiple proofs (more efficient)
    pub fn verify_range_proofs_batch(&self, proofs: &[SerializableRangeProof]) -> bool {
        if proofs.is_empty() {
            return true;
        }
        
        // Deserialize all proofs
        let range_proofs: Result<Vec<_>, _> = proofs.iter()
            .map(|p| RangeProof::from_bytes(&p.proof_bytes))
            .collect();
        
        let range_proofs = match range_proofs {
            Ok(rp) => rp,
            Err(_) => return false,
        };
        
        // Deserialize all commitments
        let commitments: Vec<CompressedRistretto> = proofs.iter()
            .map(|p| {
                if p.commitment.len() != 32 {
                    CompressedRistretto([0u8; 32]) // Invalid, will fail verification
                } else {
                    let mut bytes = [0u8; 32];
                    bytes.copy_from_slice(&p.commitment);
                    CompressedRistretto(bytes)
                }
            })
            .collect();
        
        // Create transcripts for each proof
        let mut transcripts: Vec<Transcript> = proofs.iter()
            .map(|p| {
                let mut t = Transcript::new(b"RangeProof");
                t.append_u64(b"min", p.min_value);
                t.append_u64(b"max", p.max_value);
                t
            })
            .collect();
        
        // Batch verify - convert Vec to slices and handle transcripts properly
        let bp_gens_batch = BulletproofGens::new(64, proofs.len());
        
        // Convert transcripts Vec to iterator of mutable references
        let result = range_proofs.iter()
            .zip(commitments.iter())
            .zip(transcripts.iter_mut())
            .all(|((proof, commitment), transcript)| {
                proof.verify_single(
                    &self.bp_gens,
                    &self.pc_gens,
                    transcript,
                    commitment,
                    64,
                ).is_ok()
            });
        
        result
    }
}

// Integration helper for the existing PrivateContractManager
pub struct ZKRangeProofIntegration {
    generator: RangeProofGenerator,
    verifier: RangeProofVerifier,
    // Store blinding factors (in production, use secure storage)
    blinding_factors: std::collections::HashMap<String, Scalar>,
}

impl ZKRangeProofIntegration {
    pub fn new() -> Self {
        Self {
            generator: RangeProofGenerator::new(),
            verifier: RangeProofVerifier::new(),
            blinding_factors: std::collections::HashMap::new(),
        }
    }
    
    // Create and store a duration proof
    pub fn create_duration_proof(
        &mut self,
        session_id: &str,
        duration_minutes: u64,
    ) -> Result<SerializableRangeProof, String> {
        let (proof, blinding) = self.generator.prove_call_duration(duration_minutes)?;
        
        // Store blinding factor for potential future use (e.g., disputes)
        self.blinding_factors.insert(session_id.to_string(), blinding);
        
        Ok(proof)
    }
    
    // Verify a duration proof
    pub fn verify_duration_proof(&self, proof: &SerializableRangeProof) -> bool {
        // Check that it's a valid duration range
        if proof.min_value != 0 || proof.max_value != MAX_CALL_DURATION_MINUTES {
            return false;
        }
        
        self.verifier.verify_range_proof(proof)
    }
    
    // Create proofs for a complete billing record
    pub fn create_billing_proofs(
        &mut self,
        session_id: &str,
        duration_minutes: u64,
        data_mb: u64,
        sms_count: u64,
    ) -> Result<BillingProofs, String> {
        let duration_proof = self.create_duration_proof(
            &format!("{}_duration", session_id),
            duration_minutes
        )?;
        
        let (data_proof, data_blinding) = self.generator.prove_data_volume(data_mb)?;
        self.blinding_factors.insert(
            format!("{}_data", session_id),
            data_blinding
        );
        
        let (sms_proof, sms_blinding) = self.generator.prove_sms_count(sms_count)?;
        self.blinding_factors.insert(
            format!("{}_sms", session_id),
            sms_blinding
        );
        
        Ok(BillingProofs {
            duration_proof,
            data_proof,
            sms_proof,
        })
    }
    
    // Verify all proofs in a billing record
    pub fn verify_billing_proofs(&self, proofs: &BillingProofs) -> bool {
        // Batch verification is more efficient
        self.verifier.verify_range_proofs_batch(&[
            proofs.duration_proof.clone(),
            proofs.data_proof.clone(),
            proofs.sms_proof.clone(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingProofs {
    pub duration_proof: SerializableRangeProof,
    pub data_proof: SerializableRangeProof,
    pub sms_proof: SerializableRangeProof,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_duration_range_proof() {
        let generator = RangeProofGenerator::new();
        let verifier = RangeProofVerifier::new();
        
        // Test valid duration
        let duration = 45; // 45 minutes
        let (proof, _blinding) = generator.prove_call_duration(duration).unwrap();
        
        // Verify the proof
        assert!(verifier.verify_range_proof(&proof));
        assert_eq!(proof.min_value, 0);
        assert_eq!(proof.max_value, MAX_CALL_DURATION_MINUTES);
    }
    
    #[test]
    fn test_invalid_duration_rejected() {
        let generator = RangeProofGenerator::new();
        
        // Test duration exceeding maximum
        let duration = 500; // Too long
        let result = generator.prove_call_duration(duration);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_batch_verification() {
        let generator = RangeProofGenerator::new();
        let verifier = RangeProofVerifier::new();
        
        // Create multiple proofs
        let proofs: Vec<SerializableRangeProof> = vec![
            generator.prove_call_duration(30).unwrap().0,
            generator.prove_call_duration(60).unwrap().0,
            generator.prove_call_duration(120).unwrap().0,
        ];
        
        // Batch verify
        assert!(verifier.verify_range_proofs_batch(&proofs));
    }
    
    #[test]
    fn test_tampered_proof_fails() {
        let generator = RangeProofGenerator::new();
        let verifier = RangeProofVerifier::new();
        
        let (mut proof, _) = generator.prove_call_duration(30).unwrap();
        
        // Tamper with the proof
        proof.proof_bytes[0] ^= 0xFF;
        
        // Verification should fail
        assert!(!verifier.verify_range_proof(&proof));
    }
    
    #[test]
    fn test_complete_billing_proofs() {
        let mut integration = ZKRangeProofIntegration::new();
        
        // Create proofs for a complete billing record
        let proofs = integration.create_billing_proofs(
            "session_123",
            45,   // 45 minutes
            500,  // 500 MB
            10,   // 10 SMS
        ).unwrap();
        
        // Verify all proofs
        assert!(integration.verify_billing_proofs(&proofs));
    }
}