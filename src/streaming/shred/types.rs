use solana_sdk::transaction::VersionedTransaction;

/// Transaction with slot information
#[derive(Debug, Clone, Default)]
pub struct TransactionWithSlot {
    pub transaction: VersionedTransaction,
    pub slot: u64,
    pub recv_us: i64,
    /// Transaction index within the entry (best-effort when shredstream has no slot-level index)
    pub tx_index: Option<u64>,
}

impl TransactionWithSlot {
    /// Create a new transaction with slot info
    pub fn new(
        transaction: VersionedTransaction,
        slot: u64,
        recv_us: i64,
        tx_index: Option<u64>,
    ) -> Self {
        Self { transaction, slot, recv_us, tx_index }
    }
}
