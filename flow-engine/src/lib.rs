//! Flow Engine - Browser Flow Recorder & Replayer
//!
//! This crate provides browser automation capabilities for recording and replaying
//! any user-defined browser flow. Use cases include:
//!
//! - **Login flows** for session management and Intruder integration
//! - **Checkout flows** for e-commerce testing
//! - **Multi-step forms** for data entry automation
//! - **Navigation sequences** for web scraping setup
//!
//! ## Core Concepts
//!
//! - **FlowProfile**: A recorded sequence of browser actions
//! - **FlowStep**: Individual actions (Click, Type, Navigate, Wait, etc.)
//! - **SmartSelector**: Self-healing element selectors with fallback strategies
//!
//! ## Example
//!
//! ```rust,ignore
//! use flow_engine::FlowProfile;
//!
//! // Load a previously recorded flow
//! let profile = FlowProfile::load("my_login_flow").await?;
//!
//! // Replay the flow and extract session cookies
//! let session = profile.replay().await?;
//! ```

pub mod error;
pub mod flow;

// Re-exports
pub use error::FlowEngineError;
pub use flow::model::{FlowProfile, FlowStep, SmartSelector, FlowMeta, FlowType};
pub use secrecy::SecretString;
pub use flow::browser::{BrowserManager, BrowserOptions, ProxyConfig, ManagedBrowser};
pub use flow::page::PageController;
pub use flow::analyzer::{SelectorAnalyzer, AnalyzerConfig, ElementInfo, SelectorBlacklist};
pub use flow::recorder::{FlowRecorder, RecordingConfig, RecordingState, RecordedEvent};
pub use flow::replayer::{FlowReplayer, ReplayOptions, ReplayResult};
