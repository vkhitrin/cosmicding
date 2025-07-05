use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, Default)]
pub enum SyncStatus {
    #[default]
    None,
    InProgress,
    Successful,
    Warning,
    Failed,
}
