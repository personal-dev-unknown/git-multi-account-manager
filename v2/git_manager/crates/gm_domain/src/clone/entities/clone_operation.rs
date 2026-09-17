use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Tracks the lifecycle of a single clone operation.
#[derive(Debug, Clone)]
pub struct CloneOperation {
    pub uuid:            Uuid,
    pub account_id:      Uuid,
    pub repository_id:   Option<Uuid>,
    pub url:             String,
    pub destination:     String,
    pub strategy:        String,
    pub protocol:        String,
    pub status:          CloneOperationStatus,
    pub attempts:        u32,
    pub duration_ms:     Option<u64>,
    pub error_message:   Option<String>,
    pub started_at:      DateTime<Utc>,
    pub completed_at:    Option<DateTime<Utc>>,
}

/// The lifecycle status of a clone operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneOperationStatus {
    Started,
    Completed,
    Failed,
}

impl CloneOperation {
    pub fn new(
        account_id: Uuid,
        url: String,
        destination: String,
        strategy: String,
        protocol: String,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            account_id,
            repository_id: None,
            url,
            destination,
            strategy,
            protocol,
            status: CloneOperationStatus::Started,
            attempts: 1,
            duration_ms: None,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn mark_completed(&mut self, duration_ms: u64) {
        self.status = CloneOperationStatus::Completed;
        self.duration_ms = Some(duration_ms);
        self.completed_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self, error: String, duration_ms: u64) {
        self.status = CloneOperationStatus::Failed;
        self.error_message = Some(error);
        self.duration_ms = Some(duration_ms);
        self.completed_at = Some(Utc::now());
    }

    pub fn is_finished(&self) -> bool {
        self.status != CloneOperationStatus::Started
    }
}
