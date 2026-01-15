//! Flow Profile GraphQL Types
//!
//! GraphQL types for browser flow recording and replay.

use async_graphql::{Object, SimpleObject, InputObject, Enum};
use crate::database::flow::{FlowProfileRow, FlowExecutionRow};

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "PascalCase")]
pub enum FlowTypeGql {
    Login,
    Checkout,
    FormSubmission,
    Navigation,
    Custom,
}

impl From<&str> for FlowTypeGql {
    fn from(s: &str) -> Self {
        match s {
            "Login" => FlowTypeGql::Login,
            "Checkout" => FlowTypeGql::Checkout,
            "FormSubmission" => FlowTypeGql::FormSubmission,
            "Navigation" => FlowTypeGql::Navigation,
            _ => FlowTypeGql::Custom,
        }
    }
}

impl From<FlowTypeGql> for String {
    fn from(t: FlowTypeGql) -> Self {
        match t {
            FlowTypeGql::Login => "Login".to_string(),
            FlowTypeGql::Checkout => "Checkout".to_string(),
            FlowTypeGql::FormSubmission => "FormSubmission".to_string(),
            FlowTypeGql::Navigation => "Navigation".to_string(),
            FlowTypeGql::Custom => "Custom".to_string(),
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProfileStatusGql {
    Active,
    Archived,
    Failed,
    Recording,
}

impl From<&str> for ProfileStatusGql {
    fn from(s: &str) -> Self {
        match s {
            "Active" => ProfileStatusGql::Active,
            "Archived" => ProfileStatusGql::Archived,
            "Failed" => ProfileStatusGql::Failed,
            "Recording" => ProfileStatusGql::Recording,
            _ => ProfileStatusGql::Active,
        }
    }
}

impl From<ProfileStatusGql> for String {
    fn from(s: ProfileStatusGql) -> Self {
        match s {
            ProfileStatusGql::Active => "Active".to_string(),
            ProfileStatusGql::Archived => "Archived".to_string(),
            ProfileStatusGql::Failed => "Failed".to_string(),
            ProfileStatusGql::Recording => "Recording".to_string(),
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ExecutionStatusGql {
    Running,
    Success,
    Failed,
    Cancelled,
}

impl From<&str> for ExecutionStatusGql {
    fn from(s: &str) -> Self {
        match s {
            "running" => ExecutionStatusGql::Running,
            "success" => ExecutionStatusGql::Success,
            "failed" => ExecutionStatusGql::Failed,
            "cancelled" => ExecutionStatusGql::Cancelled,
            _ => ExecutionStatusGql::Running,
        }
    }
}

// ============================================================================
// OUTPUT TYPES
// ============================================================================

#[derive(SimpleObject, Debug, Clone)]
pub struct FlowProfileGql {
    pub id: String,
    pub name: String,
    pub flow_type: FlowTypeGql,
    pub start_url: String,
    pub steps: String, // JSON array of steps
    pub meta: Option<String>, // JSON metadata
    pub created_at: i64,
    pub updated_at: i64,
    pub agent_id: Option<String>,
    pub status: ProfileStatusGql,
    pub step_count: i32,
}

impl From<FlowProfileRow> for FlowProfileGql {
    fn from(row: FlowProfileRow) -> Self {
        // Count steps from JSON
        let step_count = serde_json::from_str::<Vec<serde_json::Value>>(&row.steps)
            .map(|v| v.len() as i32)
            .unwrap_or(0);

        Self {
            id: row.id,
            name: row.name,
            flow_type: FlowTypeGql::from(row.flow_type.as_str()),
            start_url: row.start_url,
            steps: row.steps,
            meta: row.meta,
            created_at: row.created_at,
            updated_at: row.updated_at,
            agent_id: row.agent_id,
            status: ProfileStatusGql::from(row.status.as_str()),
            step_count,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct FlowExecutionGql {
    pub id: String,
    pub profile_id: String,
    pub agent_id: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub status: ExecutionStatusGql,
    pub error_message: Option<String>,
    pub steps_completed: i64,
    pub total_steps: i64,
    pub session_cookies: Option<String>,
    pub extracted_data: Option<String>,
    pub duration_ms: Option<i64>,
}

impl From<FlowExecutionRow> for FlowExecutionGql {
    fn from(row: FlowExecutionRow) -> Self {
        let duration_ms = row.completed_at.map(|c| c - row.started_at);
        
        Self {
            id: row.id,
            profile_id: row.profile_id,
            agent_id: row.agent_id,
            started_at: row.started_at,
            completed_at: row.completed_at,
            status: ExecutionStatusGql::from(row.status.as_str()),
            error_message: row.error_message,
            steps_completed: row.steps_completed,
            total_steps: row.total_steps,
            session_cookies: row.session_cookies,
            extracted_data: row.extracted_data,
            duration_ms,
        }
    }
}

#[derive(SimpleObject)]
pub struct FlowOperationResult {
    pub success: bool,
    pub message: String,
    pub profile_id: Option<String>,
}

// ============================================================================
// INPUT TYPES
// ============================================================================

#[derive(InputObject)]
pub struct CreateFlowProfileInput {
    pub name: String,
    pub flow_type: FlowTypeGql,
    pub start_url: String,
    pub agent_id: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateFlowProfileInput {
    pub name: Option<String>,
    pub status: Option<ProfileStatusGql>,
}

#[derive(InputObject)]
pub struct StartRecordingInput {
    pub name: String,
    pub start_url: String,
    pub flow_type: FlowTypeGql,
    pub agent_id: Option<String>,
}

#[derive(InputObject)]
pub struct ReplayFlowInput {
    pub profile_id: String,
    pub agent_id: String,
    pub variables: Option<String>, // JSON object of variable substitutions
    pub headed: Option<bool>, // Show browser window
}

// ============================================================================
// RECORDING STATE TYPES
// ============================================================================

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum RecordingStateGql {
    Idle,
    Recording,
    Paused,
    Completed,
    Failed,
}

impl Default for RecordingStateGql {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(SimpleObject, Clone)]
pub struct RecordingSessionGql {
    /// Profile ID being recorded
    pub profile_id: Option<String>,
    /// Current recording state
    pub state: RecordingStateGql,
    /// Number of events captured
    pub event_count: i32,
    /// Current URL
    pub current_url: Option<String>,
    /// Recording started timestamp
    pub started_at: Option<i64>,
    /// Error message if failed
    pub error: Option<String>,
}

impl Default for RecordingSessionGql {
    fn default() -> Self {
        Self {
            profile_id: None,
            state: RecordingStateGql::Idle,
            event_count: 0,
            current_url: None,
            started_at: None,
            error: None,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(name = "FlowReplayResult")]
pub struct FlowReplayResult {
    pub success: bool,
    pub execution_id: Option<String>,
    pub error: Option<String>,
    pub session_cookies: Option<String>,
}

#[derive(InputObject)]
pub struct StopRecordingInput {
    pub save: bool, // Whether to save the recording
}
