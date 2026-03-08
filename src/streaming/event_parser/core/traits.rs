use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::core::account_event_parser::{
    NonceAccountEvent, TokenAccountEvent, TokenInfoEvent,
};
use crate::streaming::event_parser::core::common_event_parser::{
    SetComputeUnitLimitEvent, SetComputeUnitPriceEvent,
};
use crate::streaming::event_parser::protocols::block::block_meta_event::BlockMetaEvent;
use crate::streaming::event_parser::protocols::bonk::events::*;
use crate::streaming::event_parser::protocols::meteora_damm_v2::events::*;
use crate::streaming::event_parser::protocols::pumpfun::events::*;
use crate::streaming::event_parser::protocols::pumpswap::events::*;
use crate::streaming::event_parser::protocols::raydium_amm_v4::events::*;
use crate::streaming::event_parser::protocols::raydium_clmm::events::*;
use crate::streaming::event_parser::protocols::raydium_cpmm::events::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Unified Event Enum - Replaces the trait-based approach with a type-safe enum
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DexEvent {
    // Bonk events
    BonkTradeEvent(BonkTradeEvent),
    BonkPoolCreateEvent(BonkPoolCreateEvent),
    BonkMigrateToAmmEvent(BonkMigrateToAmmEvent),
    BonkMigrateToCpswapEvent(BonkMigrateToCpswapEvent),
    BonkPoolStateAccountEvent(BonkPoolStateAccountEvent),
    BonkGlobalConfigAccountEvent(BonkGlobalConfigAccountEvent),
    BonkPlatformConfigAccountEvent(BonkPlatformConfigAccountEvent),

    // PumpFun events
    PumpFunCreateTokenEvent(PumpFunCreateTokenEvent),
    PumpFunCreateV2TokenEvent(PumpFunCreateV2TokenEvent),
    PumpFunTradeEvent(PumpFunTradeEvent),
    PumpFunMigrateEvent(PumpFunMigrateEvent),
    PumpFunBondingCurveAccountEvent(PumpFunBondingCurveAccountEvent),
    PumpFunGlobalAccountEvent(PumpFunGlobalAccountEvent),

    // PumpSwap events
    PumpSwapBuyEvent(PumpSwapBuyEvent),
    PumpSwapSellEvent(PumpSwapSellEvent),
    PumpSwapCreatePoolEvent(PumpSwapCreatePoolEvent),
    PumpSwapDepositEvent(PumpSwapDepositEvent),
    PumpSwapWithdrawEvent(PumpSwapWithdrawEvent),
    PumpSwapGlobalConfigAccountEvent(PumpSwapGlobalConfigAccountEvent),
    PumpSwapPoolAccountEvent(PumpSwapPoolAccountEvent),

    // Raydium AMM V4 events
    RaydiumAmmV4SwapEvent(RaydiumAmmV4SwapEvent),
    RaydiumAmmV4DepositEvent(RaydiumAmmV4DepositEvent),
    RaydiumAmmV4WithdrawEvent(RaydiumAmmV4WithdrawEvent),
    RaydiumAmmV4WithdrawPnlEvent(RaydiumAmmV4WithdrawPnlEvent),
    RaydiumAmmV4Initialize2Event(RaydiumAmmV4Initialize2Event),
    RaydiumAmmV4AmmInfoAccountEvent(RaydiumAmmV4AmmInfoAccountEvent),

    // Raydium CLMM events
    RaydiumClmmSwapEvent(RaydiumClmmSwapEvent),
    RaydiumClmmSwapV2Event(RaydiumClmmSwapV2Event),
    RaydiumClmmClosePositionEvent(RaydiumClmmClosePositionEvent),
    RaydiumClmmIncreaseLiquidityV2Event(RaydiumClmmIncreaseLiquidityV2Event),
    RaydiumClmmDecreaseLiquidityV2Event(RaydiumClmmDecreaseLiquidityV2Event),
    RaydiumClmmCreatePoolEvent(RaydiumClmmCreatePoolEvent),
    RaydiumClmmOpenPositionWithToken22NftEvent(RaydiumClmmOpenPositionWithToken22NftEvent),
    RaydiumClmmOpenPositionV2Event(RaydiumClmmOpenPositionV2Event),
    RaydiumClmmAmmConfigAccountEvent(RaydiumClmmAmmConfigAccountEvent),
    RaydiumClmmPoolStateAccountEvent(RaydiumClmmPoolStateAccountEvent),
    RaydiumClmmTickArrayStateAccountEvent(RaydiumClmmTickArrayStateAccountEvent),

    // Raydium CPMM events
    RaydiumCpmmSwapEvent(RaydiumCpmmSwapEvent),
    RaydiumCpmmDepositEvent(RaydiumCpmmDepositEvent),
    RaydiumCpmmWithdrawEvent(RaydiumCpmmWithdrawEvent),
    RaydiumCpmmInitializeEvent(RaydiumCpmmInitializeEvent),
    RaydiumCpmmAmmConfigAccountEvent(RaydiumCpmmAmmConfigAccountEvent),
    RaydiumCpmmPoolStateAccountEvent(RaydiumCpmmPoolStateAccountEvent),

    // Meteora DAMM v2 events
    MeteoraDammV2SwapEvent(MeteoraDammV2SwapEvent),
    MeteoraDammV2Swap2Event(MeteoraDammV2Swap2Event),
    MeteoraDammV2InitializePoolEvent(MeteoraDammV2InitializePoolEvent),
    MeteoraDammV2InitializeCustomizablePoolEvent(MeteoraDammV2InitializeCustomizablePoolEvent),
    MeteoraDammV2InitializePoolWithDynamicConfigEvent(MeteoraDammV2InitializePoolWithDynamicConfigEvent),

    // Common events
    TokenAccountEvent(TokenAccountEvent),
    NonceAccountEvent(NonceAccountEvent),
    TokenInfoEvent(TokenInfoEvent),
    BlockMetaEvent(BlockMetaEvent),
    SetComputeUnitLimitEvent(SetComputeUnitLimitEvent),
    SetComputeUnitPriceEvent(SetComputeUnitPriceEvent),
}

/// Macro to generate metadata accessors for all DexEvent variants
macro_rules! impl_dex_event_metadata {
    ($($variant:ident),* $(,)?) => {
        impl DexEvent {
            pub fn metadata(&self) -> &EventMetadata {
                match self {
                    $(DexEvent::$variant(event) => &event.metadata,)*
                }
            }

            pub fn metadata_mut(&mut self) -> &mut EventMetadata {
                match self {
                    $(DexEvent::$variant(event) => &mut event.metadata,)*
                }
            }
        }
    };
}

impl_dex_event_metadata!(
    // Bonk events
    BonkTradeEvent,
    BonkPoolCreateEvent,
    BonkMigrateToAmmEvent,
    BonkMigrateToCpswapEvent,
    BonkPoolStateAccountEvent,
    BonkGlobalConfigAccountEvent,
    BonkPlatformConfigAccountEvent,
    // PumpFun events
    PumpFunCreateTokenEvent,
    PumpFunCreateV2TokenEvent,
    PumpFunTradeEvent,
    PumpFunMigrateEvent,
    PumpFunBondingCurveAccountEvent,
    PumpFunGlobalAccountEvent,
    // PumpSwap events
    PumpSwapBuyEvent,
    PumpSwapSellEvent,
    PumpSwapCreatePoolEvent,
    PumpSwapDepositEvent,
    PumpSwapWithdrawEvent,
    PumpSwapGlobalConfigAccountEvent,
    PumpSwapPoolAccountEvent,
    // Raydium AMM V4 events
    RaydiumAmmV4SwapEvent,
    RaydiumAmmV4DepositEvent,
    RaydiumAmmV4WithdrawEvent,
    RaydiumAmmV4WithdrawPnlEvent,
    RaydiumAmmV4Initialize2Event,
    RaydiumAmmV4AmmInfoAccountEvent,
    // Raydium CLMM events
    RaydiumClmmSwapEvent,
    RaydiumClmmSwapV2Event,
    RaydiumClmmClosePositionEvent,
    RaydiumClmmIncreaseLiquidityV2Event,
    RaydiumClmmDecreaseLiquidityV2Event,
    RaydiumClmmCreatePoolEvent,
    RaydiumClmmOpenPositionWithToken22NftEvent,
    RaydiumClmmOpenPositionV2Event,
    RaydiumClmmAmmConfigAccountEvent,
    RaydiumClmmPoolStateAccountEvent,
    RaydiumClmmTickArrayStateAccountEvent,
    // Raydium CPMM events
    RaydiumCpmmSwapEvent,
    RaydiumCpmmDepositEvent,
    RaydiumCpmmWithdrawEvent,
    RaydiumCpmmInitializeEvent,
    RaydiumCpmmAmmConfigAccountEvent,
    RaydiumCpmmPoolStateAccountEvent,
    // Meteora DAMM v2 events
    MeteoraDammV2SwapEvent,
    MeteoraDammV2Swap2Event,
    MeteoraDammV2InitializePoolEvent,
    MeteoraDammV2InitializeCustomizablePoolEvent,
    MeteoraDammV2InitializePoolWithDynamicConfigEvent,
    // Common events
    TokenAccountEvent,
    NonceAccountEvent,
    TokenInfoEvent,
    BlockMetaEvent,
    SetComputeUnitLimitEvent,
    SetComputeUnitPriceEvent,
);

/// Trait abstracting over different instruction types for unified parsing
pub trait InstructionLike {
    fn program_id_index(&self) -> usize;
    fn accounts(&self) -> &[u8];
    fn data(&self) -> &[u8];
}

impl InstructionLike for solana_sdk::message::compiled_instruction::CompiledInstruction {
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

impl InstructionLike for yellowstone_grpc_proto::prelude::CompiledInstruction {
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

impl InstructionLike for yellowstone_grpc_proto::prelude::InnerInstruction {
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
