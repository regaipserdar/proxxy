//! Payload position parsing for attack templates
//! 
//! This module provides functionality to parse and validate payload positions
//! marked with §marker§ syntax in request templates.

use crate::error::{AttackError, AttackResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a payload position in a request template
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayloadPosition {
    /// Start position in the template string
    pub start: usize,
    /// End position in the template string (exclusive)
    pub end: usize,
    /// Original marker text (including § symbols)
    pub marker: String,
    /// Payload set identifier extracted from marker
    pub payload_set_id: String,
    /// Position index for ordering
    pub index: usize,
}

/// Result of parsing payload positions from a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTemplate {
    /// Original template string
    pub template: String,
    /// List of payload positions found
    pub positions: Vec<PayloadPosition>,
    /// Template with positions replaced by placeholders
    pub processed_template: String,
}

/// Parser for payload position markers
pub struct PayloadPositionParser;

impl PayloadPositionParser {
    /// Parse payload positions from a request template
    pub fn parse(template: &str) -> AttackResult<ParsedTemplate> {
        let mut positions = Vec::new();
        let mut processed_template = template.to_string();
        
        // Find all §marker§ patterns using char indices for proper UTF-8 handling
        let mut current_pos = 0;
        let mut position_index = 0;
        let chars: Vec<char> = template.chars().collect();
        
        while current_pos < chars.len() {
            // Find the opening §
            if let Some(start_marker_idx) = chars[current_pos..].iter().position(|&c| c == '§') {
                let absolute_start_idx = current_pos + start_marker_idx;
                
                // Find the closing §
                if let Some(end_marker_idx) = chars[absolute_start_idx + 1..].iter().position(|&c| c == '§') {
                    let absolute_end_idx = absolute_start_idx + 1 + end_marker_idx + 1; // +1 for the closing §
                    
                    // Convert char indices to byte indices
                    let absolute_start = chars[..absolute_start_idx].iter().map(|c| c.len_utf8()).sum::<usize>();
                    let absolute_end = chars[..absolute_end_idx].iter().map(|c| c.len_utf8()).sum::<usize>();
                    
                    // Extract the marker content
                    let marker_content = &template[absolute_start..absolute_end];
                    let payload_id = &template[absolute_start + '§'.len_utf8()..absolute_end - '§'.len_utf8()]; // Remove § symbols
                    
                    // Validate marker content
                    Self::validate_marker(payload_id)?;
                    
                    // Create payload position (using original template positions)
                    let position = PayloadPosition {
                        start: absolute_start,
                        end: absolute_end,
                        marker: marker_content.to_string(),
                        payload_set_id: payload_id.to_string(),
                        index: position_index,
                    };
                    
                    positions.push(position);
                    position_index += 1;
                    current_pos = absolute_end_idx;
                } else {
                    // Unmatched § - this is an error
                    return Err(AttackError::InvalidPayloadConfig {
                        reason: format!("Unmatched § at position {}", absolute_start_idx),
                    });
                }
            } else {
                break;
            }
        }
        
        // Check for unmatched closing §
        if template.chars().filter(|&c| c == '§').count() % 2 != 0 {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Unmatched § markers in template".to_string(),
            });
        }
        
        // Now create the processed template by replacing markers with placeholders
        // We need to do this in reverse order to maintain correct positions
        let mut sorted_positions = positions.clone();
        sorted_positions.sort_by(|a, b| b.start.cmp(&a.start)); // Sort in reverse order
        
        for position in &sorted_positions {
            let placeholder = format!("{{PAYLOAD_{}}}", position.index);
            processed_template.replace_range(position.start..position.end, &placeholder);
        }
        
        Ok(ParsedTemplate {
            template: template.to_string(),
            positions,
            processed_template,
        })
    }
    
    /// Validate a marker identifier
    fn validate_marker(marker: &str) -> AttackResult<()> {
        if marker.is_empty() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Empty payload marker".to_string(),
            });
        }
        
        if marker.contains('§') {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Payload marker cannot contain § symbol".to_string(),
            });
        }
        
        // Check for valid identifier characters (alphanumeric, underscore, hyphen)
        if !marker.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(AttackError::InvalidPayloadConfig {
                reason: format!("Invalid characters in payload marker: {}", marker),
            });
        }
        
        Ok(())
    }
    
    /// Inject payloads into a parsed template
    pub fn inject_payloads(
        parsed: &ParsedTemplate,
        payload_values: &HashMap<String, String>,
    ) -> AttackResult<String> {
        let mut result = parsed.processed_template.clone();
        
        // Replace placeholders with actual payload values
        for position in &parsed.positions {
            let placeholder = format!("{{PAYLOAD_{}}}", position.index);
            
            let payload_value = payload_values
                .get(&position.payload_set_id)
                .ok_or_else(|| AttackError::InvalidPayloadConfig {
                    reason: format!("No payload value provided for marker: {}", position.payload_set_id),
                })?;
            
            result = result.replace(&placeholder, payload_value);
        }
        
        Ok(result)
    }
    
    /// Get unique payload set IDs from parsed template
    pub fn get_payload_set_ids(parsed: &ParsedTemplate) -> Vec<String> {
        let mut ids: Vec<String> = parsed
            .positions
            .iter()
            .map(|p| p.payload_set_id.clone())
            .collect();
        
        ids.sort();
        ids.dedup();
        ids
    }
    
    /// Validate that all required payload sets are provided
    pub fn validate_payload_sets(
        parsed: &ParsedTemplate,
        available_sets: &[String],
    ) -> AttackResult<()> {
        let required_sets = Self::get_payload_set_ids(parsed);
        
        for required_set in &required_sets {
            if !available_sets.contains(required_set) {
                return Err(AttackError::InvalidPayloadConfig {
                    reason: format!("Missing payload set: {}", required_set),
                });
            }
        }
        
        Ok(())
    }
    
    /// Highlight payload positions in template for UI display
    pub fn highlight_positions(template: &str) -> AttackResult<Vec<(usize, usize, String)>> {
        let parsed = Self::parse(template)?;
        
        Ok(parsed
            .positions
            .into_iter()
            .map(|p| (p.start, p.end, p.payload_set_id))
            .collect())
    }
}

/// Utility functions for template manipulation
pub struct TemplateUtils;

impl TemplateUtils {
    /// Check if a template contains payload markers
    pub fn has_payload_markers(template: &str) -> bool {
        template.contains('§')
    }
    
    /// Count payload positions in template
    pub fn count_payload_positions(template: &str) -> AttackResult<usize> {
        let parsed = PayloadPositionParser::parse(template)?;
        Ok(parsed.positions.len())
    }
    
    /// Extract all unique payload set IDs from template
    pub fn extract_payload_set_ids(template: &str) -> AttackResult<Vec<String>> {
        let parsed = PayloadPositionParser::parse(template)?;
        Ok(PayloadPositionParser::get_payload_set_ids(&parsed))
    }
    
    /// Validate template syntax without parsing positions
    pub fn validate_template_syntax(template: &str) -> AttackResult<()> {
        PayloadPositionParser::parse(template)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_single_position() {
        let template = "GET /api/users/§user_id§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        
        assert_eq!(parsed.positions.len(), 1);
        assert_eq!(parsed.positions[0].payload_set_id, "user_id");
        assert_eq!(parsed.positions[0].marker, "§user_id§");
        assert_eq!(parsed.positions[0].index, 0);
        assert!(parsed.processed_template.contains("{PAYLOAD_0}"));
    }
    
    #[test]
    fn test_parse_multiple_positions() {
        let template = "POST /api/§endpoint§/§action§ HTTP/1.1\nContent-Type: application/json\n{\"user\":\"§username§\"}";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        
        assert_eq!(parsed.positions.len(), 3);
        assert_eq!(parsed.positions[0].payload_set_id, "endpoint");
        assert_eq!(parsed.positions[1].payload_set_id, "action");
        assert_eq!(parsed.positions[2].payload_set_id, "username");
    }
    
    #[test]
    fn test_parse_duplicate_markers() {
        let template = "GET /api/§param§/test/§param§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        
        assert_eq!(parsed.positions.len(), 2);
        assert_eq!(parsed.positions[0].payload_set_id, "param");
        assert_eq!(parsed.positions[1].payload_set_id, "param");
    }
    
    #[test]
    fn test_parse_no_markers() {
        let template = "GET /api/users HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        
        assert_eq!(parsed.positions.len(), 0);
        assert_eq!(parsed.processed_template, template);
    }
    
    #[test]
    fn test_parse_unmatched_marker() {
        let template = "GET /api/§user_id HTTP/1.1";
        let result = PayloadPositionParser::parse(template);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unmatched §"));
    }
    
    #[test]
    fn test_parse_empty_marker() {
        let template = "GET /api/§§ HTTP/1.1";
        let result = PayloadPositionParser::parse(template);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty payload marker"));
    }
    
    #[test]
    fn test_parse_invalid_marker_characters() {
        let template = "GET /api/§user@id§ HTTP/1.1";
        let result = PayloadPositionParser::parse(template);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid characters"));
    }
    
    #[test]
    fn test_inject_payloads() {
        let template = "GET /api/users/§user_id§/posts/§post_id§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        
        let mut payload_values = HashMap::new();
        payload_values.insert("user_id".to_string(), "123".to_string());
        payload_values.insert("post_id".to_string(), "456".to_string());
        
        let result = PayloadPositionParser::inject_payloads(&parsed, &payload_values).unwrap();
        assert_eq!(result, "GET /api/users/123/posts/456 HTTP/1.1");
    }
    
    #[test]
    fn test_inject_payloads_missing_value() {
        let template = "GET /api/users/§user_id§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        
        let payload_values = HashMap::new();
        let result = PayloadPositionParser::inject_payloads(&parsed, &payload_values);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No payload value provided"));
    }
    
    #[test]
    fn test_get_payload_set_ids() {
        let template = "GET /api/§endpoint§/§action§/§endpoint§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        
        let ids = PayloadPositionParser::get_payload_set_ids(&parsed);
        assert_eq!(ids, vec!["action", "endpoint"]); // Sorted and deduplicated
    }
    
    #[test]
    fn test_validate_payload_sets() {
        let template = "GET /api/§endpoint§/§action§ HTTP/1.1";
        let parsed = PayloadPositionParser::parse(template).unwrap();
        
        let available_sets = vec!["endpoint".to_string(), "action".to_string()];
        assert!(PayloadPositionParser::validate_payload_sets(&parsed, &available_sets).is_ok());
        
        let incomplete_sets = vec!["endpoint".to_string()];
        assert!(PayloadPositionParser::validate_payload_sets(&parsed, &incomplete_sets).is_err());
    }
    
    #[test]
    fn test_highlight_positions() {
        let template = "GET /api/§endpoint§/§action§ HTTP/1.1";
        let highlights = PayloadPositionParser::highlight_positions(template).unwrap();
        
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].2, "endpoint");
        assert_eq!(highlights[1].2, "action");
    }
    
    #[test]
    fn test_template_utils() {
        let template_with_markers = "GET /api/§endpoint§ HTTP/1.1";
        let template_without_markers = "GET /api/users HTTP/1.1";
        
        assert!(TemplateUtils::has_payload_markers(template_with_markers));
        assert!(!TemplateUtils::has_payload_markers(template_without_markers));
        
        assert_eq!(TemplateUtils::count_payload_positions(template_with_markers).unwrap(), 1);
        assert_eq!(TemplateUtils::count_payload_positions(template_without_markers).unwrap(), 0);
        
        let ids = TemplateUtils::extract_payload_set_ids(template_with_markers).unwrap();
        assert_eq!(ids, vec!["endpoint"]);
        
        assert!(TemplateUtils::validate_template_syntax(template_with_markers).is_ok());
        assert!(TemplateUtils::validate_template_syntax("GET /api/§invalid").is_err());
    }
}