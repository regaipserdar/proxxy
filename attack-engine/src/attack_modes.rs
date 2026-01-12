//! Attack mode implementations for intruder attacks
//! 
//! This module provides different attack modes for combining payloads
//! with payload positions in request templates.

use crate::error::{AttackError, AttackResult};
use crate::parser::{ParsedTemplate, PayloadPositionParser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Attack mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttackMode {
    /// Single position, iterate through all payloads
    Sniper,
    /// Multiple positions, same payload in all positions
    BatteringRam,
    /// Multiple positions, parallel iteration through payload sets
    Pitchfork,
    /// Multiple positions, all combinations of payloads
    ClusterBomb,
}

/// Represents a single attack request with payload values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackRequest {
    /// Request template with payloads injected
    pub request: String,
    /// Payload values used for this request
    pub payload_values: HashMap<String, String>,
    /// Request index in the attack sequence
    pub index: usize,
}

/// Attack mode executor trait
pub trait AttackModeExecutor: Send + Sync {
    /// Generate all attack requests for the given template and payload sets
    fn generate_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<Vec<AttackRequest>>;
    
    /// Get the total number of requests that will be generated
    fn count_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<usize>;
    
    /// Get a description of this attack mode
    fn description(&self) -> String;
}

/// Sniper mode: Single position, all payloads
pub struct SniperMode;

impl AttackModeExecutor for SniperMode {
    fn generate_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<Vec<AttackRequest>> {
        if template.positions.is_empty() {
            return Err(AttackError::InvalidAttackConfig {
                reason: "Sniper mode requires at least one payload position".to_string(),
            });
        }
        
        // Sniper mode uses only the first position
        let first_position = &template.positions[0];
        let payload_set = payload_sets
            .get(&first_position.payload_set_id)
            .ok_or_else(|| AttackError::InvalidPayloadConfig {
                reason: format!("Missing payload set: {}", first_position.payload_set_id),
            })?;
        
        let mut requests = Vec::new();
        
        for (index, payload) in payload_set.iter().enumerate() {
            let mut payload_values = HashMap::new();
            payload_values.insert(first_position.payload_set_id.clone(), payload.clone());
            
            let request = PayloadPositionParser::inject_payloads(template, &payload_values)?;
            
            requests.push(AttackRequest {
                request,
                payload_values,
                index,
            });
        }
        
        Ok(requests)
    }
    
    fn count_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<usize> {
        if template.positions.is_empty() {
            return Ok(0);
        }
        
        let first_position = &template.positions[0];
        let payload_set = payload_sets
            .get(&first_position.payload_set_id)
            .ok_or_else(|| AttackError::InvalidPayloadConfig {
                reason: format!("Missing payload set: {}", first_position.payload_set_id),
            })?;
        
        Ok(payload_set.len())
    }
    
    fn description(&self) -> String {
        "Sniper: Single position, iterate through all payloads".to_string()
    }
}

/// Battering Ram mode: Multiple positions, same payload in all
pub struct BatteringRamMode;

impl AttackModeExecutor for BatteringRamMode {
    fn generate_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<Vec<AttackRequest>> {
        if template.positions.is_empty() {
            return Err(AttackError::InvalidAttackConfig {
                reason: "Battering Ram mode requires at least one payload position".to_string(),
            });
        }
        
        // Get the first payload set (all positions use the same payloads)
        let first_position = &template.positions[0];
        let payload_set = payload_sets
            .get(&first_position.payload_set_id)
            .ok_or_else(|| AttackError::InvalidPayloadConfig {
                reason: format!("Missing payload set: {}", first_position.payload_set_id),
            })?;
        
        let mut requests = Vec::new();
        
        for (index, payload) in payload_set.iter().enumerate() {
            let mut payload_values = HashMap::new();
            
            // Use the same payload for all positions
            for position in &template.positions {
                payload_values.insert(position.payload_set_id.clone(), payload.clone());
            }
            
            let request = PayloadPositionParser::inject_payloads(template, &payload_values)?;
            
            requests.push(AttackRequest {
                request,
                payload_values,
                index,
            });
        }
        
        Ok(requests)
    }
    
    fn count_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<usize> {
        if template.positions.is_empty() {
            return Ok(0);
        }
        
        let first_position = &template.positions[0];
        let payload_set = payload_sets
            .get(&first_position.payload_set_id)
            .ok_or_else(|| AttackError::InvalidPayloadConfig {
                reason: format!("Missing payload set: {}", first_position.payload_set_id),
            })?;
        
        Ok(payload_set.len())
    }
    
    fn description(&self) -> String {
        "Battering Ram: Multiple positions, same payload in all positions".to_string()
    }
}

/// Pitchfork mode: Multiple positions, parallel iteration
pub struct PitchforkMode;

impl AttackModeExecutor for PitchforkMode {
    fn generate_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<Vec<AttackRequest>> {
        if template.positions.is_empty() {
            return Err(AttackError::InvalidAttackConfig {
                reason: "Pitchfork mode requires at least one payload position".to_string(),
            });
        }
        
        // Get all required payload sets
        let mut position_payloads = Vec::new();
        for position in &template.positions {
            let payload_set = payload_sets
                .get(&position.payload_set_id)
                .ok_or_else(|| AttackError::InvalidPayloadConfig {
                    reason: format!("Missing payload set: {}", position.payload_set_id),
                })?;
            position_payloads.push((position.payload_set_id.clone(), payload_set));
        }
        
        // Find the minimum length (parallel iteration stops at shortest list)
        let min_length = position_payloads
            .iter()
            .map(|(_, payloads)| payloads.len())
            .min()
            .unwrap_or(0);
        
        let mut requests = Vec::new();
        
        for index in 0..min_length {
            let mut payload_values = HashMap::new();
            
            // Take one payload from each set at the same index
            for (payload_set_id, payloads) in &position_payloads {
                payload_values.insert(payload_set_id.clone(), payloads[index].clone());
            }
            
            let request = PayloadPositionParser::inject_payloads(template, &payload_values)?;
            
            requests.push(AttackRequest {
                request,
                payload_values,
                index,
            });
        }
        
        Ok(requests)
    }
    
    fn count_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<usize> {
        if template.positions.is_empty() {
            return Ok(0);
        }
        
        let mut min_length = usize::MAX;
        
        for position in &template.positions {
            let payload_set = payload_sets
                .get(&position.payload_set_id)
                .ok_or_else(|| AttackError::InvalidPayloadConfig {
                    reason: format!("Missing payload set: {}", position.payload_set_id),
                })?;
            min_length = min_length.min(payload_set.len());
        }
        
        Ok(if min_length == usize::MAX { 0 } else { min_length })
    }
    
    fn description(&self) -> String {
        "Pitchfork: Multiple positions, parallel iteration through payload sets".to_string()
    }
}

/// Cluster Bomb mode: Multiple positions, all combinations
pub struct ClusterBombMode;

impl AttackModeExecutor for ClusterBombMode {
    fn generate_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<Vec<AttackRequest>> {
        if template.positions.is_empty() {
            return Err(AttackError::InvalidAttackConfig {
                reason: "Cluster Bomb mode requires at least one payload position".to_string(),
            });
        }
        
        // Get all required payload sets
        let mut position_payloads = Vec::new();
        for position in &template.positions {
            let payload_set = payload_sets
                .get(&position.payload_set_id)
                .ok_or_else(|| AttackError::InvalidPayloadConfig {
                    reason: format!("Missing payload set: {}", position.payload_set_id),
                })?;
            position_payloads.push((position.payload_set_id.clone(), payload_set));
        }
        
        // Generate all combinations using cartesian product
        let combinations = Self::cartesian_product(&position_payloads);
        let mut requests = Vec::new();
        
        for (index, combination) in combinations.into_iter().enumerate() {
            let payload_values: HashMap<String, String> = combination.into_iter().collect();
            let request = PayloadPositionParser::inject_payloads(template, &payload_values)?;
            
            requests.push(AttackRequest {
                request,
                payload_values,
                index,
            });
        }
        
        Ok(requests)
    }
    
    fn count_requests(
        &self,
        template: &ParsedTemplate,
        payload_sets: &HashMap<String, Vec<String>>,
    ) -> AttackResult<usize> {
        if template.positions.is_empty() {
            return Ok(0);
        }
        
        let mut total = 1usize;
        
        for position in &template.positions {
            let payload_set = payload_sets
                .get(&position.payload_set_id)
                .ok_or_else(|| AttackError::InvalidPayloadConfig {
                    reason: format!("Missing payload set: {}", position.payload_set_id),
                })?;
            
            total = total.saturating_mul(payload_set.len());
            
            // Safety check to prevent excessive memory usage
            if total > 10_000_000 {
                return Err(AttackError::InvalidAttackConfig {
                    reason: "Cluster Bomb would generate too many requests (>10M)".to_string(),
                });
            }
        }
        
        Ok(total)
    }
    
    fn description(&self) -> String {
        "Cluster Bomb: Multiple positions, all combinations of payloads".to_string()
    }
}

impl ClusterBombMode {
    /// Generate cartesian product of payload sets
    fn cartesian_product(
        position_payloads: &[(String, &Vec<String>)],
    ) -> Vec<Vec<(String, String)>> {
        if position_payloads.is_empty() {
            return vec![vec![]];
        }
        
        let mut result = vec![vec![]];
        
        for (payload_set_id, payloads) in position_payloads {
            let mut new_result = Vec::new();
            
            for existing_combination in result {
                for payload in *payloads {
                    let mut new_combination = existing_combination.clone();
                    new_combination.push((payload_set_id.clone(), payload.clone()));
                    new_result.push(new_combination);
                }
            }
            
            result = new_result;
        }
        
        result
    }
}

/// Factory for creating attack mode executors
pub struct AttackModeFactory;

impl AttackModeFactory {
    /// Create an attack mode executor
    pub fn create(mode: &AttackMode) -> Box<dyn AttackModeExecutor> {
        match mode {
            AttackMode::Sniper => Box::new(SniperMode),
            AttackMode::BatteringRam => Box::new(BatteringRamMode),
            AttackMode::Pitchfork => Box::new(PitchforkMode),
            AttackMode::ClusterBomb => Box::new(ClusterBombMode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::PayloadPositionParser;
    
    fn create_test_payload_sets() -> HashMap<String, Vec<String>> {
        let mut payload_sets = HashMap::new();
        payload_sets.insert("user".to_string(), vec!["admin".to_string(), "guest".to_string()]);
        payload_sets.insert("pass".to_string(), vec!["123".to_string(), "456".to_string(), "789".to_string()]);
        payload_sets
    }
    
    #[test]
    fn test_sniper_mode() {
        let template = "GET /api/users/§user§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        let payload_sets = create_test_payload_sets();
        
        let sniper = SniperMode;
        let requests = sniper.generate_requests(&parsed, &payload_sets).unwrap();
        
        assert_eq!(requests.len(), 2); // Only uses first payload set
        assert_eq!(sniper.count_requests(&parsed, &payload_sets).unwrap(), 2);
        
        assert!(requests[0].request.contains("admin"));
        assert!(requests[1].request.contains("guest"));
    }
    
    #[test]
    fn test_battering_ram_mode() {
        let template = "GET /api/login?user=§user§&pass=§user§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        let payload_sets = create_test_payload_sets();
        
        let battering_ram = BatteringRamMode;
        let requests = battering_ram.generate_requests(&parsed, &payload_sets).unwrap();
        
        assert_eq!(requests.len(), 2); // Uses first payload set for all positions
        assert_eq!(battering_ram.count_requests(&parsed, &payload_sets).unwrap(), 2);
        
        assert!(requests[0].request.contains("user=admin&pass=admin"));
        assert!(requests[1].request.contains("user=guest&pass=guest"));
    }
    
    #[test]
    fn test_pitchfork_mode() {
        let template = "GET /api/login?user=§user§&pass=§pass§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        let payload_sets = create_test_payload_sets();
        
        let pitchfork = PitchforkMode;
        let requests = pitchfork.generate_requests(&parsed, &payload_sets).unwrap();
        
        assert_eq!(requests.len(), 2); // Min length of payload sets (user has 2, pass has 3)
        assert_eq!(pitchfork.count_requests(&parsed, &payload_sets).unwrap(), 2);
        
        assert!(requests[0].request.contains("user=admin&pass=123"));
        assert!(requests[1].request.contains("user=guest&pass=456"));
    }
    
    #[test]
    fn test_cluster_bomb_mode() {
        let template = "GET /api/login?user=§user§&pass=§pass§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        let payload_sets = create_test_payload_sets();
        
        let cluster_bomb = ClusterBombMode;
        let requests = cluster_bomb.generate_requests(&parsed, &payload_sets).unwrap();
        
        assert_eq!(requests.len(), 6); // 2 users * 3 passwords = 6 combinations
        assert_eq!(cluster_bomb.count_requests(&parsed, &payload_sets).unwrap(), 6);
        
        // Check that all combinations are present
        let request_strings: Vec<String> = requests.iter().map(|r| r.request.clone()).collect();
        assert!(request_strings.iter().any(|r| r.contains("user=admin&pass=123")));
        assert!(request_strings.iter().any(|r| r.contains("user=admin&pass=456")));
        assert!(request_strings.iter().any(|r| r.contains("user=admin&pass=789")));
        assert!(request_strings.iter().any(|r| r.contains("user=guest&pass=123")));
        assert!(request_strings.iter().any(|r| r.contains("user=guest&pass=456")));
        assert!(request_strings.iter().any(|r| r.contains("user=guest&pass=789")));
    }
    
    #[test]
    fn test_attack_mode_factory() {
        let sniper = AttackModeFactory::create(&AttackMode::Sniper);
        assert_eq!(sniper.description(), "Sniper: Single position, iterate through all payloads");
        
        let battering_ram = AttackModeFactory::create(&AttackMode::BatteringRam);
        assert_eq!(battering_ram.description(), "Battering Ram: Multiple positions, same payload in all positions");
        
        let pitchfork = AttackModeFactory::create(&AttackMode::Pitchfork);
        assert_eq!(pitchfork.description(), "Pitchfork: Multiple positions, parallel iteration through payload sets");
        
        let cluster_bomb = AttackModeFactory::create(&AttackMode::ClusterBomb);
        assert_eq!(cluster_bomb.description(), "Cluster Bomb: Multiple positions, all combinations of payloads");
    }
    
    #[test]
    fn test_empty_template() {
        let template = "GET /api/users HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        let payload_sets = create_test_payload_sets();
        
        let sniper = SniperMode;
        let result = sniper.generate_requests(&parsed, &payload_sets);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_missing_payload_set() {
        let template = "GET /api/users/§missing§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        let payload_sets = create_test_payload_sets();
        
        let sniper = SniperMode;
        let result = sniper.generate_requests(&parsed, &payload_sets);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing payload set"));
    }
    
    #[test]
    fn test_cluster_bomb_cartesian_product() {
        let vec_a = vec!["1".to_string(), "2".to_string()];
        let vec_b = vec!["x".to_string(), "y".to_string()];
        let position_payloads = vec![
            ("a".to_string(), &vec_a),
            ("b".to_string(), &vec_b),
        ];
        
        let result = ClusterBombMode::cartesian_product(&position_payloads);
        
        assert_eq!(result.len(), 4);
        assert!(result.contains(&vec![("a".to_string(), "1".to_string()), ("b".to_string(), "x".to_string())]));
        assert!(result.contains(&vec![("a".to_string(), "1".to_string()), ("b".to_string(), "y".to_string())]));
        assert!(result.contains(&vec![("a".to_string(), "2".to_string()), ("b".to_string(), "x".to_string())]));
        assert!(result.contains(&vec![("a".to_string(), "2".to_string()), ("b".to_string(), "y".to_string())]));
    }
}