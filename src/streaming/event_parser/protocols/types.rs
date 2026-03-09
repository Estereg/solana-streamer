use crate::streaming::event_parser::common::ProtocolType;
use crate::streaming::event_parser::protocols::{
    bonk::parser::BONK_PROGRAM_ID, meteora_damm_v2::parser::METEORA_DAMM_V2_PROGRAM_ID,
    pumpfun::parser::PUMPFUN_PROGRAM_ID, pumpswap::parser::PUMPSWAP_PROGRAM_ID,
    raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID, raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID,
    raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID,
};
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;

/// Supported protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Protocol {
    PumpSwap,
    PumpFun,
    Bonk,
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
    MeteoraDammV2,
}

impl Protocol {
    #[must_use]
    pub fn get_program_id(&self) -> Vec<Pubkey> {
        match self {
            Self::PumpSwap => vec![PUMPSWAP_PROGRAM_ID],
            Self::PumpFun => vec![PUMPFUN_PROGRAM_ID],
            Self::Bonk => vec![BONK_PROGRAM_ID],
            Self::RaydiumCpmm => vec![RAYDIUM_CPMM_PROGRAM_ID],
            Self::RaydiumClmm => vec![RAYDIUM_CLMM_PROGRAM_ID],
            Self::RaydiumAmmV4 => vec![RAYDIUM_AMM_V4_PROGRAM_ID],
            Self::MeteoraDammV2 => vec![METEORA_DAMM_V2_PROGRAM_ID],
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PumpSwap => write!(f, "PumpSwap"),
            Self::PumpFun => write!(f, "PumpFun"),
            Self::Bonk => write!(f, "Bonk"),
            Self::RaydiumCpmm => write!(f, "RaydiumCpmm"),
            Self::RaydiumClmm => write!(f, "RaydiumClmm"),
            Self::RaydiumAmmV4 => write!(f, "RaydiumAmmV4"),
            Self::MeteoraDammV2 => write!(f, "MeteoraDammV2"),
        }
    }
}

impl std::str::FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pumpswap" => Ok(Self::PumpSwap),
            "pumpfun" => Ok(Self::PumpFun),
            "bonk" => Ok(Self::Bonk),
            "raydiumcpmm" => Ok(Self::RaydiumCpmm),
            "raydiumclmm" => Ok(Self::RaydiumClmm),
            "raydiumammv4" => Ok(Self::RaydiumAmmV4),
            "meteoradamm_v2" => Ok(Self::MeteoraDammV2),
            _ => Err(anyhow!("Unsupported protocol: {s}")),
        }
    }
}

impl From<Protocol> for ProtocolType {
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::PumpFun => Self::PumpFun,
            Protocol::PumpSwap => Self::PumpSwap,
            Protocol::Bonk => Self::Bonk,
            Protocol::RaydiumCpmm => Self::RaydiumCpmm,
            Protocol::RaydiumClmm => Self::RaydiumClmm,
            Protocol::RaydiumAmmV4 => Self::RaydiumAmmV4,
            Protocol::MeteoraDammV2 => Self::MeteoraDammV2,
        }
    }
}
