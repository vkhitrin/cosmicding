#[derive(Debug, Clone)]
pub struct OperationProgress {
    pub operation_id: u64,
    pub total: usize,
    pub current: usize,
    pub operation_label: String,
    pub cancellable: bool,
}
