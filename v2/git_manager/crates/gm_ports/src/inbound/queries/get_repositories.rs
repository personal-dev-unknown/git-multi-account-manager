// crates/gm_ports/src/inbound/queries/get_repositories.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::models::repository::RepositoryDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoriesQuery {
    pub account_id:   Option<Uuid>,
    pub cloned_only:  bool,
    pub limit:        Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoriesResult {
    pub repositories: Vec<RepositoryDto>,
    pub total:        usize,
}