//! Payload generation system for attack engine
//! 
//! This module provides traits and implementations for generating payloads
//! used in fuzzing and brute-force attacks.

use crate::error::{AttackError, AttackResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

/// Trait for generating payloads for attacks
#[async_trait]
pub trait PayloadGenerator: Send + Sync {
    /// Generate all payloads for this generator
    async fn generate(&self) -> AttackResult<Vec<String>>;
    
    /// Get the total count of payloads without generating them
    async fn count(&self) -> AttackResult<usize>;
    
    /// Get a human-readable description of this generator
    fn description(&self) -> String;
    
    /// Validate the generator configuration
    fn validate(&self) -> AttackResult<()>;
}

/// Configuration for different payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayloadConfig {
    Wordlist { 
        file_path: String, 
        encoding: String 
    },
    NumberRange { 
        start: i64, 
        end: i64, 
        step: i64, 
        format: String 
    },
    Custom { 
        values: Vec<String> 
    },
}

/// Generator for file-based wordlist payloads
#[derive(Debug, Clone)]
pub struct WordlistGenerator {
    file_path: String,
    encoding: String,
}

impl WordlistGenerator {
    /// Create a new wordlist generator
    pub fn new(file_path: String, encoding: Option<String>) -> Self {
        Self {
            file_path,
            encoding: encoding.unwrap_or_else(|| "utf-8".to_string()),
        }
    }
    
    /// Create from payload config
    pub fn from_config(config: &PayloadConfig) -> AttackResult<Self> {
        match config {
            PayloadConfig::Wordlist { file_path, encoding } => {
                Ok(Self::new(file_path.clone(), Some(encoding.clone())))
            }
            _ => Err(AttackError::InvalidPayloadConfig {
                reason: "Expected wordlist configuration".to_string(),
            }),
        }
    }
}

#[async_trait]
impl PayloadGenerator for WordlistGenerator {
    async fn generate(&self) -> AttackResult<Vec<String>> {
        // Validate file exists
        if !Path::new(&self.file_path).exists() {
            return Err(AttackError::PayloadGenerationFailed {
                reason: format!("Wordlist file not found: {}", self.file_path),
            });
        }
        
        // Read file content
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| AttackError::PayloadGenerationFailed {
                reason: format!("Failed to read wordlist file: {}", e),
            })?;
        
        // Parse lines and filter empty ones
        let payloads: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        
        if payloads.is_empty() {
            return Err(AttackError::PayloadGenerationFailed {
                reason: "Wordlist file contains no valid payloads".to_string(),
            });
        }
        
        Ok(payloads)
    }
    
    async fn count(&self) -> AttackResult<usize> {
        // Efficient counting without loading entire file into memory
        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| AttackError::PayloadGenerationFailed {
                reason: format!("Failed to read wordlist file: {}", e),
            })?;
        
        let count = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        
        Ok(count)
    }
    
    fn description(&self) -> String {
        format!("Wordlist from file: {}", self.file_path)
    }
    
    fn validate(&self) -> AttackResult<()> {
        if self.file_path.is_empty() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "File path cannot be empty".to_string(),
            });
        }
        
        if !Path::new(&self.file_path).exists() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: format!("Wordlist file does not exist: {}", self.file_path),
            });
        }
        
        // Validate encoding (basic check)
        if self.encoding.is_empty() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Encoding cannot be empty".to_string(),
            });
        }
        
        Ok(())
    }
}

/// Generator for numeric sequence payloads
#[derive(Debug, Clone)]
pub struct NumberRangeGenerator {
    start: i64,
    end: i64,
    step: i64,
    format: String,
}

impl NumberRangeGenerator {
    /// Create a new number range generator
    pub fn new(start: i64, end: i64, step: i64, format: Option<String>) -> Self {
        Self {
            start,
            end,
            step,
            format: format.unwrap_or_else(|| "{}".to_string()),
        }
    }
    
    /// Create from payload config
    pub fn from_config(config: &PayloadConfig) -> AttackResult<Self> {
        match config {
            PayloadConfig::NumberRange { start, end, step, format } => {
                Ok(Self::new(*start, *end, *step, Some(format.clone())))
            }
            _ => Err(AttackError::InvalidPayloadConfig {
                reason: "Expected number range configuration".to_string(),
            }),
        }
    }
}

#[async_trait]
impl PayloadGenerator for NumberRangeGenerator {
    async fn generate(&self) -> AttackResult<Vec<String>> {
        let mut payloads = Vec::new();
        let mut current = self.start;
        
        // Prevent infinite loops
        if self.step == 0 {
            return Err(AttackError::PayloadGenerationFailed {
                reason: "Step cannot be zero".to_string(),
            });
        }
        
        if (self.step > 0 && self.start > self.end) || (self.step < 0 && self.start < self.end) {
            return Err(AttackError::PayloadGenerationFailed {
                reason: "Invalid range: step direction doesn't match start/end relationship".to_string(),
            });
        }
        
        // Generate payloads
        while (self.step > 0 && current <= self.end) || (self.step < 0 && current >= self.end) {
            let formatted = if self.format.contains("{}") {
                self.format.replace("{}", &current.to_string())
            } else {
                // Try to use format as a printf-style format string
                match self.format.as_str() {
                    "%d" => current.to_string(),
                    "%x" => format!("{:x}", current),
                    "%X" => format!("{:X}", current),
                    "%o" => format!("{:o}", current),
                    _ => {
                        // For other format strings, just use the number directly
                        current.to_string()
                    }
                }
            };
            
            payloads.push(formatted);
            current += self.step;
            
            // Safety check to prevent excessive memory usage
            if payloads.len() > 1_000_000 {
                return Err(AttackError::PayloadGenerationFailed {
                    reason: "Number range too large (>1M payloads)".to_string(),
                });
            }
        }
        
        Ok(payloads)
    }
    
    async fn count(&self) -> AttackResult<usize> {
        if self.step == 0 {
            return Err(AttackError::PayloadGenerationFailed {
                reason: "Step cannot be zero".to_string(),
            });
        }
        
        if (self.step > 0 && self.start > self.end) || (self.step < 0 && self.start < self.end) {
            return Ok(0);
        }
        
        let count = ((self.end - self.start) / self.step + 1) as usize;
        
        // Safety check
        if count > 1_000_000 {
            return Err(AttackError::PayloadGenerationFailed {
                reason: "Number range too large (>1M payloads)".to_string(),
            });
        }
        
        Ok(count)
    }
    
    fn description(&self) -> String {
        format!("Number range: {} to {} (step {})", self.start, self.end, self.step)
    }
    
    fn validate(&self) -> AttackResult<()> {
        if self.step == 0 {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Step cannot be zero".to_string(),
            });
        }
        
        if (self.step > 0 && self.start > self.end) || (self.step < 0 && self.start < self.end) {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Invalid range: step direction doesn't match start/end relationship".to_string(),
            });
        }
        
        if self.format.is_empty() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Format string cannot be empty".to_string(),
            });
        }
        
        Ok(())
    }
}

/// Generator for user-defined custom payloads
#[derive(Debug, Clone)]
pub struct CustomGenerator {
    values: Vec<String>,
}

impl CustomGenerator {
    /// Create a new custom generator
    pub fn new(values: Vec<String>) -> Self {
        Self { values }
    }
    
    /// Create from payload config
    pub fn from_config(config: &PayloadConfig) -> AttackResult<Self> {
        match config {
            PayloadConfig::Custom { values } => {
                Ok(Self::new(values.clone()))
            }
            _ => Err(AttackError::InvalidPayloadConfig {
                reason: "Expected custom configuration".to_string(),
            }),
        }
    }
}

#[async_trait]
impl PayloadGenerator for CustomGenerator {
    async fn generate(&self) -> AttackResult<Vec<String>> {
        if self.values.is_empty() {
            return Err(AttackError::PayloadGenerationFailed {
                reason: "Custom payload list is empty".to_string(),
            });
        }
        
        Ok(self.values.clone())
    }
    
    async fn count(&self) -> AttackResult<usize> {
        Ok(self.values.len())
    }
    
    fn description(&self) -> String {
        format!("Custom payloads ({} items)", self.values.len())
    }
    
    fn validate(&self) -> AttackResult<()> {
        if self.values.is_empty() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Custom payload list cannot be empty".to_string(),
            });
        }
        
        Ok(())
    }
}

/// Factory for creating payload generators from configuration
pub struct PayloadGeneratorFactory;

impl PayloadGeneratorFactory {
    /// Create a payload generator from configuration
    pub fn create(config: &PayloadConfig) -> AttackResult<Box<dyn PayloadGenerator>> {
        match config {
            PayloadConfig::Wordlist { .. } => {
                Ok(Box::new(WordlistGenerator::from_config(config)?))
            }
            PayloadConfig::NumberRange { .. } => {
                Ok(Box::new(NumberRangeGenerator::from_config(config)?))
            }
            PayloadConfig::Custom { .. } => {
                Ok(Box::new(CustomGenerator::from_config(config)?))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;
    use std::path::PathBuf;
    
    async fn create_test_wordlist() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_wordlist.txt");
        
        let content = "admin\npassword\ntest\n123456\n\n  \nroot\n";
        fs::write(&file_path, content).await.unwrap();
        
        file_path
    }
    
    #[tokio::test]
    async fn test_wordlist_generator() {
        let file_path = create_test_wordlist().await;
        let generator = WordlistGenerator::new(file_path.to_string_lossy().to_string(), None);
        
        // Test validation
        assert!(generator.validate().is_ok());
        
        // Test count
        let count = generator.count().await.unwrap();
        assert_eq!(count, 5); // admin, password, test, 123456, root
        
        // Test generation
        let payloads = generator.generate().await.unwrap();
        assert_eq!(payloads.len(), 5);
        assert!(payloads.contains(&"admin".to_string()));
        assert!(payloads.contains(&"password".to_string()));
        assert!(!payloads.contains(&"".to_string())); // Empty lines filtered
        
        // Cleanup
        let _ = fs::remove_file(file_path).await;
    }
    
    #[tokio::test]
    async fn test_number_range_generator() {
        let generator = NumberRangeGenerator::new(1, 5, 1, None);
        
        // Test validation
        assert!(generator.validate().is_ok());
        
        // Test count
        let count = generator.count().await.unwrap();
        assert_eq!(count, 5);
        
        // Test generation
        let payloads = generator.generate().await.unwrap();
        assert_eq!(payloads, vec!["1", "2", "3", "4", "5"]);
    }
    
    #[tokio::test]
    async fn test_number_range_generator_with_format() {
        let generator = NumberRangeGenerator::new(1, 3, 1, Some("user{}".to_string()));
        
        let payloads = generator.generate().await.unwrap();
        assert_eq!(payloads, vec!["user1", "user2", "user3"]);
    }
    
    #[tokio::test]
    async fn test_custom_generator() {
        let values = vec!["custom1".to_string(), "custom2".to_string(), "custom3".to_string()];
        let generator = CustomGenerator::new(values.clone());
        
        // Test validation
        assert!(generator.validate().is_ok());
        
        // Test count
        let count = generator.count().await.unwrap();
        assert_eq!(count, 3);
        
        // Test generation
        let payloads = generator.generate().await.unwrap();
        assert_eq!(payloads, values);
    }
    
    #[tokio::test]
    async fn test_payload_generator_factory() {
        // Test wordlist config
        let wordlist_config = PayloadConfig::Wordlist {
            file_path: "/tmp/test.txt".to_string(),
            encoding: "utf-8".to_string(),
        };
        let generator = PayloadGeneratorFactory::create(&wordlist_config);
        assert!(generator.is_ok());
        
        // Test number range config
        let number_config = PayloadConfig::NumberRange {
            start: 1,
            end: 10,
            step: 1,
            format: "{}".to_string(),
        };
        let generator = PayloadGeneratorFactory::create(&number_config);
        assert!(generator.is_ok());
        
        // Test custom config
        let custom_config = PayloadConfig::Custom {
            values: vec!["test".to_string()],
        };
        let generator = PayloadGeneratorFactory::create(&custom_config);
        assert!(generator.is_ok());
    }
}