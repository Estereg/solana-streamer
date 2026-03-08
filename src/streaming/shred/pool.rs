use solana_sdk::transaction::VersionedTransaction;

use super::TransactionWithSlot;

/// Convenience factory functions for TransactionWithSlot
pub mod factory {
    use super::*;

    /// Create TransactionWithSlot directly (no pool overhead)
    pub fn create_transaction_with_slot(
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
        tx_index: Option<u64>,
    ) -> TransactionWithSlot {
        TransactionWithSlot {
            transaction,
            slot,
            recv_us,
            tx_index,
        }
    }
}
