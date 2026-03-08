use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::high_performance_clock::elapsed_micros_since;
use crate::streaming::event_parser::common::{EventMetadata, EventType, ProtocolType};
use crate::streaming::event_parser::core::traits::DexEvent;
use crate::streaming::event_parser::Protocol;
use crate::streaming::grpc::AccountPretty;
use serde::{Deserialize, Serialize};
use solana_account_decoder::parse_nonce::parse_nonce;
use solana_sdk::pubkey::Pubkey;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::{Account, Mint};
use spl_token_2022::{
    extension::StateWithExtensions,
    state::{Account as Account2022, Mint as Mint2022},
};

/// Generic account event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub amount: Option<u64>,
    pub token_owner: Pubkey,
}

/// Nonce account event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonceAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub nonce: String,
    pub authority: String,
}

/// Nonce account event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenInfoEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub supply: u64,
    pub decimals: u8,
}

pub struct AccountEventParser {}

impl AccountEventParser {
    pub fn parse_account_event(
        protocols: &[Protocol],
        account: AccountPretty,
        event_type_filter: Option<&EventTypeFilter>,
    ) -> Option<DexEvent> {
        use crate::streaming::event_parser::core::dispatcher::EventDispatcher;

        let metadata = EventMetadata {
            slot: account.slot,
            signature: account.signature,
            protocol: ProtocolType::Common,
            program_id: account.owner,
            recv_us: account.recv_us,
            handle_us: elapsed_micros_since(account.recv_us),
            ..Default::default()
        };

        // 1. Try protocol-specific accounts
        if account.data.len() >= 8 {
            let discriminator = &account.data[0..8];
            if let Some(protocol) = EventDispatcher::match_protocol_by_program_id(&account.owner) {
                if protocols.contains(&protocol) {
                    if let Some(event) = EventDispatcher::dispatch_account(
                        protocol,
                        discriminator,
                        &account,
                        metadata.clone(),
                    ) {
                        if event_matches_filter(&event, event_type_filter) {
                            return Some(event);
                        }
                    }
                }
            }
        }

        // 2. Generic account types
        if let Some(event) = Self::parse_nonce_account_event(&account, metadata.clone()) {
            if event_matches_filter(&event, event_type_filter) {
                return Some(event);
            }
        }

        if let Some(event) = Self::parse_token_account_event(&account, metadata) {
            if event_matches_filter(&event, event_type_filter) {
                return Some(event);
            }
        }

        None
    }

    pub fn parse_token_account_event(
        account: &AccountPretty,
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        metadata.event_type = EventType::TokenAccount;

        // Spl Token / Token2022 Mint
        if account.data.len() >= Mint::LEN {
            if let Ok(token_mint) = Mint::unpack_from_slice(&account.data) {
                return Some(DexEvent::TokenInfoEvent(TokenInfoEvent {
                    metadata: enrich_metadata(metadata, account.recv_us),
                    pubkey: account.pubkey,
                    executable: account.executable,
                    lamports: account.lamports,
                    owner: account.owner,
                    rent_epoch: account.rent_epoch,
                    supply: token_mint.supply,
                    decimals: token_mint.decimals,
                }));
            }
        }

        if account.data.len() >= Account2022::LEN {
            if let Ok(token_mint) = StateWithExtensions::<Mint2022>::unpack(&account.data) {
                return Some(DexEvent::TokenInfoEvent(TokenInfoEvent {
                    metadata: enrich_metadata(metadata, account.recv_us),
                    pubkey: account.pubkey,
                    executable: account.executable,
                    lamports: account.lamports,
                    owner: account.owner,
                    rent_epoch: account.rent_epoch,
                    supply: token_mint.base.supply,
                    decimals: token_mint.base.decimals,
                }));
            }
        }

        let amount = if account.owner == spl_token_2022::ID {
            StateWithExtensions::<Account2022>::unpack(&account.data)
                .ok()
                .map(|info| info.base.amount)
        } else {
            Account::unpack(&account.data).ok().map(|info| info.amount)
        };

        Some(DexEvent::TokenAccountEvent(TokenAccountEvent {
            metadata: enrich_metadata(metadata, account.recv_us),
            pubkey: account.pubkey,
            executable: account.executable,
            lamports: account.lamports,
            owner: account.owner,
            rent_epoch: account.rent_epoch,
            amount,
            token_owner: account.owner,
        }))
    }

    pub fn parse_nonce_account_event(
        account: &AccountPretty,
        mut metadata: EventMetadata,
    ) -> Option<DexEvent> {
        metadata.event_type = EventType::NonceAccount;

        if let Ok(solana_account_decoder::parse_nonce::UiNonceState::Initialized(details)) =
            parse_nonce(&account.data)
        {
            return Some(DexEvent::NonceAccountEvent(NonceAccountEvent {
                metadata: enrich_metadata(metadata, account.recv_us),
                pubkey: account.pubkey,
                executable: account.executable,
                lamports: account.lamports,
                owner: account.owner,
                rent_epoch: account.rent_epoch,
                nonce: details.blockhash,
                authority: details.authority,
            }));
        }
        None
    }
}

#[inline]
fn event_matches_filter(event: &DexEvent, filter: Option<&EventTypeFilter>) -> bool {
    filter.map_or(true, |f| f.include.contains(&event.metadata().event_type))
}

#[inline]
fn enrich_metadata(mut metadata: EventMetadata, recv_us: i64) -> EventMetadata {
    metadata.handle_us = elapsed_micros_since(recv_us);
    metadata
}
