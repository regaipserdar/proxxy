//! Attack Engine - Core infrastructure for Repeater and Intruder modules
//! 
//! This crate provides the foundational components for executing HTTP attacks
//! across distributed agents in the Proxxy system.

pub mod types;
pub mod traits;
pub mod error;
pub mod execution;
pub mod resource;
pub mod payload;
pub mod parser;
pub mod attack_modes;
pub mod security;

#[cfg(test)]
mod tests;

// Re-export core types with explicit names to avoid conflicts
pub use types::{
    HttpRequestData, HttpResponseData, HttpHeaders, TlsDetails,
    AttackRequest, ExecutionConfig, DistributionStrategy,
    AttackResult as AttackResultData, // Rename to avoid conflict with error::AttackResult
    AgentInfo, AgentStatus, AttackContext, ModuleType, Priority
};

pub use traits::{
    AttackExecutor, AgentManager, PayloadDistributor, 
    ResultProcessor, AttackStatistics
};

pub use error::{
    AttackError, AttackResult, ErrorRecoveryStrategy, BackoffStrategy,
    ErrorSeverity, ErrorCategory, CircuitBreaker, CircuitBreakerState, ErrorContext, ValidationError
};

pub use execution::{
    AttackEngine, DefaultPayloadDistributor
};

pub use resource::{
    ResourceManagerAdapter, ResourceAllocation, ResourceUsageStats, ResourceMonitor
};

pub use payload::{
    PayloadGenerator, PayloadConfig, WordlistGenerator, NumberRangeGenerator, 
    CustomGenerator, PayloadGeneratorFactory
};

pub use parser::{
    PayloadPosition, ParsedTemplate, PayloadPositionParser, TemplateUtils
};

pub use attack_modes::{
    AttackMode, AttackRequest as AttackModeRequest, AttackModeExecutor,
    SniperMode, BatteringRamMode, PitchforkMode, ClusterBombMode, AttackModeFactory
};

pub use security::{
    SecurityManager, MaskingConfig, SecureString, SecurityViolation, 
    ViolationType, Severity
};