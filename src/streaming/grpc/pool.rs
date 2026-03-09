/// Factory functions for creating Pretty types from gRPC updates.
///
/// These functions directly construct the target types without pool overhead.
/// They return `Result` to properly propagate parsing errors instead of
/// silently falling back to defaults.
pub mod factory {
    use crate::common::AnyResult;
    use crate::streaming::event_parser::common::high_performance_clock::get_high_perf_clock;
    use crate::streaming::grpc::types::{AccountPretty, BlockMetaPretty, TransactionPretty};
    use anyhow::anyhow;
    use solana_sdk::{pubkey::Pubkey, signature::Signature};
    use yellowstone_grpc_proto::{
        geyser::{SubscribeUpdateAccount, SubscribeUpdateBlockMeta, SubscribeUpdateTransaction},
        prost_types::Timestamp,
    };

    /// Create `AccountPretty` from gRPC update
    ///
    /// # Errors
    ///
    /// Returns error if account info is missing or pubkey/owner are invalid.
    pub fn create_account_pretty(update: SubscribeUpdateAccount) -> AnyResult<AccountPretty> {
        let account_info = update
            .account
            .ok_or_else(|| anyhow!("Missing account info in gRPC update"))?;

        let pubkey = Pubkey::try_from(account_info.pubkey.as_slice())
            .map_err(|_| anyhow!("Invalid pubkey bytes (len={})", account_info.pubkey.len()))?;
        let owner = Pubkey::try_from(account_info.owner.as_slice())
            .map_err(|_| anyhow!("Invalid owner pubkey bytes (len={})", account_info.owner.len()))?;

        Ok(AccountPretty {
            slot: update.slot,
            signature: account_info
                .txn_signature
                .as_ref()
                .and_then(|s| Signature::try_from(s.as_slice()).ok())
                .unwrap_or_default(),
            pubkey,
            executable: account_info.executable,
            lamports: account_info.lamports,
            owner,
            rent_epoch: account_info.rent_epoch,
            data: account_info.data,
            recv_us: get_high_perf_clock(),
        })
    }

    /// Create `BlockMetaPretty` from gRPC update
    #[must_use]
    pub fn create_block_meta_pretty(
        update: SubscribeUpdateBlockMeta,
        block_time: Option<Timestamp>,
    ) -> BlockMetaPretty {
        BlockMetaPretty {
            slot: update.slot,
            block_hash: update.blockhash,
            block_time,
            recv_us: get_high_perf_clock(),
        }
    }

    /// Create `TransactionPretty` from gRPC update
    ///
    /// # Errors
    ///
    /// Returns error if transaction info is missing or signature is invalid.
    pub fn create_transaction_pretty(
        update: SubscribeUpdateTransaction,
        block_time: Option<Timestamp>,
    ) -> AnyResult<TransactionPretty> {
        let transaction_info = update
            .transaction
            .ok_or_else(|| anyhow!("Missing transaction info in gRPC update"))?;

        let signature = Signature::try_from(transaction_info.signature.as_slice())
            .map_err(|_| anyhow!("Invalid signature bytes (len={})", transaction_info.signature.len()))?;

        Ok(TransactionPretty {
            slot: update.slot,
            tx_index: Some(transaction_info.index),
            block_time,
            signature,
            is_vote: transaction_info.is_vote,
            recv_us: get_high_perf_clock(),
            grpc_tx: transaction_info,
        })
    }
}
