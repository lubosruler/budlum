//! Faz A — Executor/Transaction entegrasyon seam'i.
//!
//! Lubot çıkarım talebi, `TransactionType::AiInferenceRequest` olarak executor'a
//! taşınacak. Bu modül, AiInferenceRequest → executor-ready mapping noktasını
//! tanımlar (seam). Tam Transaction construction + signing + executor akışı v0.3.

use crate::ai::types::{AiInferenceRequest, AiRequestId};
use crate::core::address::Address;

/// Executor'a gönderilecek Lubot transaction request'i (mapping seam).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LubotExecutorRequest {
    pub request_id: AiRequestId,
    pub requester: Address,
    pub max_fee: u64,
    pub deadline_block: u64,
}

impl LubotExecutorRequest {
    /// Bir AiInferenceRequest'ten executor-ready form oluştur.
    #[must_use]
    pub fn from_inference_request(req: &AiInferenceRequest) -> Self {
        Self {
            request_id: req.request_id,
            requester: req.requester,
            max_fee: req.max_fee,
            deadline_block: req.deadline_block,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::types::{AiModelId, AiRequestId, BoundedBytes};

    #[test]
    fn executor_request_from_inference_request() {
        let req = AiInferenceRequest {
            request_id: AiRequestId([1; 32]),
            requester: Address([2; 32]),
            model_id: AiModelId([3; 32]),
            input_commitment: [4; 32],
            input_ref: BoundedBytes::empty(),
            max_fee: 100,
            callback: None,
            submitted_at_block: 1,
            deadline_block: 1000,
        };
        let exec = LubotExecutorRequest::from_inference_request(&req);
        assert_eq!(exec.request_id, AiRequestId([1; 32]));
        assert_eq!(exec.requester, Address([2; 32]));
        assert_eq!(exec.max_fee, 100);
        assert_eq!(exec.deadline_block, 1000);
    }
}
