use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{borrow::Cow, fmt};

use crate::streaming::event_parser::DexEvent;

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum ProtocolType {
    #[default]
    PumpSwap,
    PumpFun,
    Bonk,
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
    MeteoraDammV2,
    Common,
}

/// Event type enumeration
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum EventType {
    // PumpSwap events
    #[default]
    PumpSwapBuy,
    PumpSwapSell,
    PumpSwapCreatePool,
    PumpSwapDeposit,
    PumpSwapWithdraw,

    // PumpFun events
    PumpFunCreateToken,
    PumpFunCreateV2Token,
    PumpFunBuy,
    PumpFunSell,
    PumpFunMigrate,

    // Bonk events
    BonkBuyExactIn,
    BonkBuyExactOut,
    BonkSellExactIn,
    BonkSellExactOut,
    BonkInitialize,
    BonkInitializeV2,
    BonkInitializeWithToken2022,
    BonkMigrateToAmm,
    BonkMigrateToCpswap,

    // Raydium CPMM events
    RaydiumCpmmSwapBaseInput,
    RaydiumCpmmSwapBaseOutput,
    RaydiumCpmmDeposit,
    RaydiumCpmmInitialize,
    RaydiumCpmmWithdraw,

    // Raydium CLMM events
    RaydiumClmmSwap,
    RaydiumClmmSwapV2,
    RaydiumClmmClosePosition,
    RaydiumClmmIncreaseLiquidityV2,
    RaydiumClmmDecreaseLiquidityV2,
    RaydiumClmmCreatePool,
    RaydiumClmmOpenPositionWithToken22Nft,
    RaydiumClmmOpenPositionV2,

    // Raydium AMM V4 events
    RaydiumAmmV4SwapBaseIn,
    RaydiumAmmV4SwapBaseOut,
    RaydiumAmmV4Deposit,
    RaydiumAmmV4Initialize2,
    RaydiumAmmV4Withdraw,
    RaydiumAmmV4WithdrawPnl,

    // Meteora DAMM v2 events
    MeteoraDammV2Swap,
    MeteoraDammV2Swap2,
    MeteoraDammV2InitializePool,
    MeteoraDammV2InitializeCustomizablePool,
    MeteoraDammV2InitializePoolWithDynamicConfig,

    // Account events
    AccountRaydiumAmmV4AmmInfo,
    AccountPumpSwapGlobalConfig,
    AccountPumpSwapPool,
    AccountBonkPoolState,
    AccountBonkGlobalConfig,
    AccountBonkPlatformConfig,
    AccountBonkVestingRecord,
    AccountPumpFunBondingCurve,
    AccountPumpFunGlobal,
    AccountRaydiumClmmAmmConfig,
    AccountRaydiumClmmPoolState,
    AccountRaydiumClmmTickArrayState,
    AccountRaydiumCpmmAmmConfig,
    AccountRaydiumCpmmPoolState,

    NonceAccount,
    TokenAccount,

    // Common events
    BlockMeta,
    SetComputeUnitLimit,
    SetComputeUnitPrice,
    Unknown,
}

pub const ACCOUNT_EVENT_TYPES: &[EventType] = &[
    EventType::AccountRaydiumAmmV4AmmInfo,
    EventType::AccountPumpSwapGlobalConfig,
    EventType::AccountPumpSwapPool,
    EventType::AccountBonkPoolState,
    EventType::AccountBonkGlobalConfig,
    EventType::AccountBonkPlatformConfig,
    EventType::AccountBonkVestingRecord,
    EventType::AccountPumpFunBondingCurve,
    EventType::AccountPumpFunGlobal,
    EventType::AccountRaydiumClmmAmmConfig,
    EventType::AccountRaydiumClmmPoolState,
    EventType::AccountRaydiumClmmTickArrayState,
    EventType::AccountRaydiumCpmmAmmConfig,
    EventType::AccountRaydiumCpmmPoolState,
    EventType::TokenAccount,
    EventType::NonceAccount,
];
pub const BLOCK_EVENT_TYPES: &[EventType] = &[EventType::BlockMeta];

impl fmt::Display for EventType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct SwapData {
    pub from_mint: Pubkey,
    pub to_mint: Pubkey,
    pub from_amount: u64,
    pub to_amount: u64,
    pub description: Option<Cow<'static, str>>,
}

/// Event metadata
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMetadata {
    pub signature: Signature,
    pub slot: u64,
    pub tx_index: Option<u64>, // Transaction index within the slot
    pub block_time: i64,
    pub block_time_ms: i64,
    pub recv_us: i64,
    pub handle_us: i64,
    pub protocol: ProtocolType,
    pub event_type: EventType,
    pub program_id: Pubkey,
    pub swap_data: Option<SwapData>,
    pub outer_index: i64,
    pub inner_index: Option<i64>,
    /// Transaction message recent blockhash as base58 string (same encoding as signature), when available.
    pub recent_blockhash: Option<String>,
}

impl EventMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        signature: Signature,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        protocol: ProtocolType,
        event_type: EventType,
        program_id: Pubkey,
        outer_index: i64,
        inner_index: Option<i64>,
        recv_us: i64,
        tx_index: Option<u64>,
        recent_blockhash: Option<String>,
    ) -> Self {
        Self {
            signature,
            slot,
            block_time,
            block_time_ms,
            recv_us,
            handle_us: 0,
            protocol,
            event_type,
            program_id,
            swap_data: None,
            outer_index,
            inner_index,
            tx_index,
            recent_blockhash,
        }
    }

    pub fn set_swap_data(&mut self, swap_data: SwapData) {
        self.swap_data = Some(swap_data);
    }
}

static SOL_MINT: std::sync::LazyLock<Pubkey> =
    std::sync::LazyLock::new(spl_token::native_mint::id);
static SYSTEM_PROGRAMS: std::sync::LazyLock<[Pubkey; 3]> = std::sync::LazyLock::new(|| [
    spl_token::id(),
    spl_token_2022::id(),
    solana_sdk::pubkey!("11111111111111111111111111111111"),
]);

/// Trait abstracting over different inner-instruction types for swap data extraction
pub trait InnerInstructionLike {
    fn program_id_index(&self) -> usize;
    fn accounts(&self) -> &[u8];
    fn data(&self) -> &[u8];
}

impl<T: InnerInstructionLike> InnerInstructionLike for &T {
    fn program_id_index(&self) -> usize {
        (**self).program_id_index()
    }
    fn accounts(&self) -> &[u8] {
        (**self).accounts()
    }
    fn data(&self) -> &[u8] {
        (**self).data()
    }
}

/// Adapter for standard Solana compiled instructions
impl InnerInstructionLike for solana_sdk::message::compiled_instruction::CompiledInstruction {
    fn program_id_index(&self) -> usize {
        self.program_id_index as usize
    }
    fn accounts(&self) -> &[u8] {
        &self.accounts
    }
    fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Adapter for gRPC inner instructions (yellowstone)
impl InnerInstructionLike for yellowstone_grpc_proto::prelude::InnerInstruction {
    fn program_id_index(&self) -> usize {
        self.program_id_index as usize
    }
    fn accounts(&self) -> &[u8] {
        &self.accounts
    }
    fn data(&self) -> &[u8] {
        &self.data
    }
}


/// Extract event context (mint/token account/vault info) from a DexEvent
fn extract_swap_context(event: &DexEvent) -> (
    SwapData,
    Option<Pubkey>, Option<Pubkey>,
    Option<Pubkey>, Option<Pubkey>,
    Option<Pubkey>, Option<Pubkey>,
) {
    let mut swap_data = SwapData::default();
    let mut from_mint: Option<Pubkey> = None;
    let mut to_mint: Option<Pubkey> = None;
    let mut user_from_token: Option<Pubkey> = None;
    let mut user_to_token: Option<Pubkey> = None;
    let mut from_vault: Option<Pubkey> = None;
    let mut to_vault: Option<Pubkey> = None;

    match event {
        DexEvent::BonkTradeEvent(e) => {
            from_mint = Some(e.base_token_mint);
            to_mint = Some(e.quote_token_mint);
            user_from_token = Some(e.user_base_token);
            user_to_token = Some(e.user_quote_token);
            from_vault = Some(e.base_vault);
            to_vault = Some(e.quote_vault);
        }
        DexEvent::PumpFunTradeEvent(e) => {
            swap_data.from_mint = if e.is_buy { *SOL_MINT } else { e.mint };
            swap_data.to_mint = if e.is_buy { e.mint } else { *SOL_MINT };
        }
        DexEvent::PumpSwapBuyEvent(e) => {
            swap_data.from_mint = e.quote_mint;
            swap_data.to_mint = e.base_mint;
        }
        DexEvent::PumpSwapSellEvent(e) => {
            swap_data.from_mint = e.base_mint;
            swap_data.to_mint = e.quote_mint;
        }
        DexEvent::RaydiumCpmmSwapEvent(e) => {
            from_mint = Some(e.input_token_mint);
            to_mint = Some(e.output_token_mint);
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumClmmSwapEvent(e) => {
            swap_data.description =
                Some("Unable to get from_mint and to_mint from RaydiumClmmSwapEvent".into());
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumClmmSwapV2Event(e) => {
            from_mint = Some(e.input_vault_mint);
            to_mint = Some(e.output_vault_mint);
            user_from_token = Some(e.input_token_account);
            user_to_token = Some(e.output_token_account);
            from_vault = Some(e.input_vault);
            to_vault = Some(e.output_vault);
        }
        DexEvent::RaydiumAmmV4SwapEvent(e) => {
            swap_data.description =
                Some("Unable to get from_mint and to_mint from RaydiumAmmV4SwapEvent".into());
            user_from_token = Some(e.user_source_token_account);
            user_to_token = Some(e.user_destination_token_account);
            from_vault = Some(e.pool_pc_token_account);
            to_vault = Some(e.pool_coin_token_account);
        }
        _ => {}
    }

    (swap_data, from_mint, to_mint, user_from_token, user_to_token, from_vault, to_vault)
}

/// Generic swap data extraction that works with any instruction type implementing InnerInstructionLike
fn extract_swap_data_from_instructions<I: InnerInstructionLike>(
    event: &DexEvent,
    instructions: impl Iterator<Item = I>,
    current_index: i8,
    accounts: &[Pubkey],
) -> Option<SwapData> {
    let (mut swap_data, from_mint_opt, to_mint_opt, user_from_token_opt, user_to_token_opt, from_vault_opt, to_vault_opt) = extract_swap_context(event);

    let user_to_token = user_to_token_opt.unwrap_or_default();
    let user_from_token = user_from_token_opt.unwrap_or_default();
    let to_vault = to_vault_opt.unwrap_or_default();
    let from_vault = from_vault_opt.unwrap_or_default();
    let to_mint = to_mint_opt.unwrap_or_default();
    let from_mint = from_mint_opt.unwrap_or_default();

    for instruction in instructions.skip((current_index + 1) as usize) {
        let program_id = accounts.get(instruction.program_id_index()).copied().unwrap_or_default();
        if !SYSTEM_PROGRAMS.contains(&program_id) {
            break;
        }
        let data = instruction.data();
        let accs = instruction.accounts();

        if data.len() < 8 {
            continue;
        }

        let get_pubkey = |i: usize| -> Option<Pubkey> {
            accounts.get(*accs.get(i)? as usize).copied()
        };
        let (source, destination, amount) = match data[0] {
            12 if accs.len() >= 4 => {
                let amt = u64::from_le_bytes(data[1..9].try_into().ok()?);
                match (get_pubkey(0), get_pubkey(2)) {
                    (Some(s), Some(d)) => (s, d, amt),
                    _ => continue,
                }
            }
            3 if accs.len() >= 3 => {
                let amt = u64::from_le_bytes(data[1..9].try_into().ok()?);
                match (get_pubkey(0), get_pubkey(1)) {
                    (Some(s), Some(d)) => (s, d, amt),
                    _ => continue,
                }
            }
            2 if accs.len() >= 2 => {
                let amt = u64::from_le_bytes(data[4..12].try_into().ok()?);
                match (get_pubkey(0), get_pubkey(1)) {
                    (Some(s), Some(d)) => (s, d, amt),
                    _ => continue,
                }
            }
            _ => continue,
        };

        match (source, destination) {
            (s, d) if s == user_to_token && d == to_vault => {
                swap_data.from_mint = to_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == from_vault && d == user_from_token => {
                swap_data.to_mint = from_mint;
                swap_data.to_amount = amount;
            }
            (s, d) if s == user_from_token && d == from_vault => {
                swap_data.from_mint = from_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == to_vault && d == user_to_token => {
                swap_data.to_mint = to_mint;
                swap_data.to_amount = amount;
            }
            (s, d) if s == user_from_token && d == to_vault => {
                swap_data.from_mint = from_mint;
                swap_data.from_amount = amount;
            }
            (s, d) if s == from_vault && d == user_to_token => {
                swap_data.to_mint = to_mint;
                swap_data.to_amount = amount;
            }
            _ => {}
        }
        if swap_data.from_mint != Pubkey::default() && swap_data.to_mint != Pubkey::default() {
            break;
        }
        if swap_data.from_amount != 0 && swap_data.to_amount != 0 {
            break;
        }
    }

    if swap_data.from_mint != Pubkey::default()
        || swap_data.to_mint != Pubkey::default()
        || swap_data.from_amount != 0
        || swap_data.to_amount != 0
    {
        Some(swap_data)
    } else {
        None
    }
}

/// Parse token transfer data from standard Solana inner instructions
pub fn parse_swap_data_from_next_instructions(
    event: &DexEvent,
    inner_instruction: &solana_transaction_status::InnerInstructions,
    current_index: i8,
    accounts: &[Pubkey],
) -> Option<SwapData> {
    extract_swap_data_from_instructions(
        event,
        inner_instruction.instructions.iter().map(|ix| &ix.instruction),
        current_index,
        accounts,
    )
}

/// Parse token transfer data from gRPC inner instructions
pub fn parse_swap_data_from_next_grpc_instructions(
    event: &DexEvent,
    inner_instruction: &yellowstone_grpc_proto::prelude::InnerInstructions,
    current_index: i8,
    accounts: &[Pubkey],
) -> Option<SwapData> {
    extract_swap_data_from_instructions(
        event,
        inner_instruction.instructions.iter(),
        current_index,
        accounts,
    )
}
