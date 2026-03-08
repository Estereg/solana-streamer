use super::types::{AccountPretty, BlockMetaPretty, TransactionPretty};
use crate::streaming::event_parser::common::high_performance_clock::get_high_perf_clock;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use yellowstone_grpc_proto::{
    geyser::{SubscribeUpdateAccount, SubscribeUpdateBlockMeta, SubscribeUpdateTransaction},
    prost_types::Timestamp,
};

/// Factory functions for creating Pretty types from gRPC updates.
///
/// These functions directly construct the target types without pool overhead.
/// The previous object pool pattern (acquire → fill → mem::replace → drop) was
/// counterproductive: it filled pools with empty Default objects, causing 100%
/// cache misses and double mutex locks per creation.
pub mod factory {
    use super::*;

    /// Create AccountPretty from gRPC update
    pub fn create_account_pretty(update: SubscribeUpdateAccount) -> AccountPretty {
        let Some(account_info) = update.account else {
            return AccountPretty::default();
        };

        AccountPretty {
            slot: update.slot,
            signature: account_info
                .txn_signature
                .as_ref()
                .and_then(|s| Signature::try_from(s.as_slice()).ok())
                .unwrap_or_default(),
            pubkey: Pubkey::try_from(account_info.pubkey.as_slice()).unwrap_or_default(),
            executable: account_info.executable,
            lamports: account_info.lamports,
            owner: Pubkey::try_from(account_info.owner.as_slice()).unwrap_or_default(),
            rent_epoch: account_info.rent_epoch,
            data: account_info.data,
            recv_us: get_high_perf_clock(),
        }
    }

    /// Create BlockMetaPretty from gRPC update
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

    /// Create TransactionPretty from gRPC update
    pub fn create_transaction_pretty(
        update: SubscribeUpdateTransaction,
        block_time: Option<Timestamp>,
    ) -> TransactionPretty {
        let Some(transaction_info) = update.transaction else {
            return TransactionPretty::default();
        };

        TransactionPretty {
            slot: update.slot,
            tx_index: Some(transaction_info.index),
            block_time,
            block_hash: String::new(),
            signature: Signature::try_from(transaction_info.signature.as_slice()).unwrap_or_default(),
            is_vote: transaction_info.is_vote,
            recv_us: get_high_perf_clock(),
            grpc_tx: transaction_info,
        }
    }
}
