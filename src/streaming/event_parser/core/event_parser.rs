use crate::streaming::event_parser::{
    DexEvent, Protocol, common::{
        EventMetadata, filter::EventTypeFilter, high_performance_clock::elapsed_micros_since, parse_swap_data_from_next_grpc_instructions, parse_swap_data_from_next_instructions
    }, core::{
        dispatcher::EventDispatcher,
        merger_event::merge,
        traits::InstructionLike,
    }, protocols::raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID
};
use prost_types::Timestamp;
use solana_sdk::{
    pubkey::Pubkey, signature::Signature,
    transaction::VersionedTransaction,
};
use solana_transaction_status::InnerInstructions;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;

pub struct EventParser {}

impl EventParser {
    // ================================================================================================
    // Public API - Entry Points
    // ================================================================================================

    /// Parse transaction from gRPC stream
    pub async fn parse_grpc_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        grpc_tx: SubscribeUpdateTransactionInfo,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        tx_index: Option<u64>,
        callback: &mut impl FnMut(&DexEvent),
    ) -> anyhow::Result<()> {
        let transaction = match grpc_tx.transaction {
            Some(tx) => tx,
            None => return Ok(()),
        };
        let message = match transaction.message {
            Some(msg) => msg,
            None => return Ok(()),
        };

        // Efficiently extract accounts
        let mut accounts = Vec::with_capacity(message.account_keys.len() + 16); // Pre-allocate some extra
        for key in &message.account_keys {
            if key.len() == 32 {
                accounts.push(Pubkey::try_from(key.as_slice()).unwrap_or_default());
            }
        }

        if let Some(meta) = grpc_tx.meta {
            for address in meta.loaded_writable_addresses {
                if address.len() == 32 {
                    accounts.push(Pubkey::try_from(address.as_slice()).unwrap_or_default());
                }
            }
            for address in meta.loaded_readonly_addresses {
                if address.len() == 32 {
                    accounts.push(Pubkey::try_from(address.as_slice()).unwrap_or_default());
                }
            }

            let recent_blockhash = if message.recent_blockhash.is_empty() {
                None
            } else {
                Some(solana_sdk::bs58::encode(&message.recent_blockhash).into_string())
            };

            Self::parse_instruction_events_from_grpc_transaction(
                protocols,
                event_type_filter,
                &message.instructions,
                signature,
                slot,
                block_time,
                recv_us,
                &accounts,
                &meta.inner_instructions,
                tx_index,
                recent_blockhash,
                callback,
            )
            .await?;
        }

        Ok(())
    }

    /// Parse transaction from VersionedTransaction
    pub async fn parse_instruction_events_from_versioned_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        transaction: &VersionedTransaction,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        accounts: &[Pubkey],
        inner_instructions: &[InnerInstructions],
        tx_index: Option<u64>,
        callback: &mut impl FnMut(&DexEvent),
    ) -> anyhow::Result<()> {
        if !accounts.iter().any(|account| Self::should_handle(protocols, event_type_filter, account)) {
            return Ok(());
        }

        let recent_blockhash: Option<String> = Some(transaction.message.recent_blockhash().to_string());
        let compiled_instructions = transaction.message.instructions();

        for (index, instruction) in compiled_instructions.iter().enumerate() {
            if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                if !Self::should_handle(protocols, event_type_filter, program_id) {
                    continue;
                }

                let inner_ix = inner_instructions.iter().find(|inner| inner.index == index as u8);
                
                Self::parse_events_from_instruction_generic(
                    protocols,
                    event_type_filter,
                    instruction,
                    accounts,
                    signature,
                    slot.unwrap_or(0),
                    block_time,
                    recv_us,
                    index as i64,
                    None,
                    tx_index,
                    recent_blockhash.clone(),
                    inner_ix,
                    callback,
                )?;

                if let Some(inner_ix) = inner_ix {
                    for (inner_index, inner_instruction) in inner_ix.instructions.iter().enumerate() {
                        Self::parse_events_from_instruction_generic(
                            protocols,
                            event_type_filter,
                            &inner_instruction.instruction,
                            accounts,
                            signature,
                            slot.unwrap_or(0),
                            block_time,
                            recv_us,
                            index as i64,
                            Some(inner_index as i64),
                            tx_index,
                            recent_blockhash.clone(),
                            Some(inner_ix),
                            callback,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    // ================================================================================================
    // gRPC Transaction Processing
    // ================================================================================================

    async fn parse_instruction_events_from_grpc_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        compiled_instructions: &[yellowstone_grpc_proto::prelude::CompiledInstruction],
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        accounts: &[Pubkey],
        inner_instructions: &[yellowstone_grpc_proto::prelude::InnerInstructions],
        tx_index: Option<u64>,
        recent_blockhash: Option<String>,
        callback: &mut impl FnMut(&DexEvent),
    ) -> anyhow::Result<()> {
        if !accounts.iter().any(|account| Self::should_handle(protocols, event_type_filter, account)) {
            return Ok(());
        }

        for (index, instruction) in compiled_instructions.iter().enumerate() {
            if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                if !Self::should_handle(protocols, event_type_filter, program_id) {
                    continue;
                }

                let inner_ix = inner_instructions.iter().find(|inner| inner.index == index as u32);
                
                Self::parse_events_from_grpc_instruction_generic(
                    protocols,
                    event_type_filter,
                    instruction,
                    accounts,
                    signature,
                    slot.unwrap_or(0),
                    block_time,
                    recv_us,
                    index as i64,
                    None,
                    tx_index,
                    recent_blockhash.clone(),
                    inner_ix,
                    callback,
                )?;

                if let Some(inner_ix) = inner_ix {
                    for (inner_index, inner_instruction) in inner_ix.instructions.iter().enumerate() {
                        Self::parse_events_from_grpc_instruction_generic(
                            protocols,
                            event_type_filter,
                            inner_instruction,
                            accounts,
                            signature,
                            slot.unwrap_or(0),
                            block_time,
                            recv_us,
                            index as i64,
                            Some(inner_index as i64),
                            tx_index,
                            recent_blockhash.clone(),
                            Some(inner_ix),
                            callback,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    // ================================================================================================
    // Generic Instruction Processing (The "Rust Pro" Consolidation)
    // ================================================================================================

    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_instruction_generic(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        instruction: &impl InstructionLike,
        accounts: &[Pubkey],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        tx_index: Option<u64>,
        recent_blockhash: Option<String>,
        inner_instructions: Option<&InnerInstructions>,
        callback: &mut impl FnMut(&DexEvent),
    ) -> anyhow::Result<()> {
        let program_id_index = instruction.program_id_index();
        if program_id_index >= accounts.len() { return Ok(()); }
        let program_id = accounts[program_id_index];
        
        Self::execute_parsing(
            protocols,
            event_type_filter,
            program_id,
            instruction.data(),
            instruction.accounts(),
            accounts,
            signature,
            slot,
            block_time,
            recv_us,
            outer_index,
            inner_index,
            tx_index,
            recent_blockhash,
            |event, idx, accounts| parse_swap_data_from_next_instructions(event, inner_instructions?, idx as i8, accounts),
            |protocol, _discriminator, _instruction_data, meta| {
                let inner_instrs = inner_instructions?;
                let current_idx = inner_index.unwrap_or(-1) as i32;
                for (idx, inner_ix) in inner_instrs.instructions.iter().enumerate() {
                    if (idx as i32) <= current_idx { continue; }
                    let data = &inner_ix.instruction.data;
                    if data.len() < 16 { continue; }
                    if let Some(inner_event) = EventDispatcher::dispatch_inner_instruction(
                        protocol, &data[..16], &data[16..], meta.clone()
                    ) {
                        return Some(inner_event);
                    }
                }
                None
            },
            callback
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_grpc_instruction_generic(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        instruction: &impl InstructionLike,
        accounts: &[Pubkey],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        tx_index: Option<u64>,
        recent_blockhash: Option<String>,
        inner_instructions: Option<&yellowstone_grpc_proto::prelude::InnerInstructions>,
        callback: &mut impl FnMut(&DexEvent),
    ) -> anyhow::Result<()> {
        let program_id_index = instruction.program_id_index();
        if program_id_index >= accounts.len() { return Ok(()); }
        let program_id = accounts[program_id_index];

        Self::execute_parsing(
            protocols,
            event_type_filter,
            program_id,
            instruction.data(),
            instruction.accounts(),
            accounts,
            signature,
            slot,
            block_time,
            recv_us,
            outer_index,
            inner_index,
            tx_index,
            recent_blockhash,
            |event, idx, accounts| parse_swap_data_from_next_grpc_instructions(event, inner_instructions?, idx as i8, accounts),
            |protocol, _discriminator, _instruction_data, meta| {
                let inner_instrs = inner_instructions?;
                let current_idx = inner_index.unwrap_or(-1) as i32;
                for (idx, inner_ix) in inner_instrs.instructions.iter().enumerate() {
                    if (idx as i32) <= current_idx { continue; }
                    let data = &inner_ix.data;
                    if data.len() < 16 { continue; }
                    if let Some(inner_event) = EventDispatcher::dispatch_inner_instruction(
                        protocol, &data[..16], &data[16..], meta.clone()
                    ) {
                        return Some(inner_event);
                    }
                }
                None
            },
            callback
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_parsing(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        program_id: Pubkey,
        data: &[u8],
        instruction_accounts: &[u8],
        all_accounts: &[Pubkey],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        tx_index: Option<u64>,
        recent_blockhash: Option<String>,
        swap_parser: impl FnOnce(&DexEvent, i64, &[Pubkey]) -> Option<crate::streaming::event_parser::common::SwapData>,
        inner_event_finder: impl FnOnce(Protocol, &[u8], &[u8], EventMetadata) -> Option<DexEvent>,
        callback: &mut impl FnMut(&DexEvent),
    ) -> anyhow::Result<()> {
        if !Self::should_handle(protocols, event_type_filter, &program_id) { return Ok(()); }

        let is_cu_program = EventDispatcher::is_compute_budget_program(&program_id);
        let discriminator_length = if program_id == RAYDIUM_AMM_V4_PROGRAM_ID { 1 } else { 8 };

        if !is_cu_program && data.len() < discriminator_length { return Ok(()); }

        let timestamp = block_time.unwrap_or(Timestamp { seconds: 0, nanos: 0 });
        let metadata = EventMetadata::new(
            signature, slot, timestamp.seconds, 
            timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000,
            Default::default(), Default::default(), program_id,
            outer_index, inner_index, recv_us, tx_index, recent_blockhash
        );

        if is_cu_program {
            if let Some(event) = EventDispatcher::dispatch_compute_budget_instruction(data, metadata) {
                callback(&event);
            }
            return Ok(());
        }

        let protocol = match EventDispatcher::match_protocol_by_program_id(&program_id) {
            Some(protocol) => protocol,
            None => return Ok(()),
        };

        let mut event = match crate::streaming::event_parser::core::parser_cache::with_account_pubkeys_cache(
            instruction_accounts, all_accounts,
            |accounts| EventDispatcher::dispatch_instruction(protocol, &data[..discriminator_length], &data[discriminator_length..], accounts, metadata.clone())
        ) {
            Some(event) => event,
            None => return Ok(()),
        };

        // Find and merge inner events (CPI logs)
        if let Some(inner_event) = inner_event_finder(protocol, &data[..discriminator_length], &data[discriminator_length..], metadata) {
            merge(&mut event, inner_event);
        }

        // Fill swap data
        if event.metadata().swap_data.is_none() {
            if let Some(swap_data) = swap_parser(&event, inner_index.unwrap_or(-1), all_accounts) {
                event.metadata_mut().set_swap_data(swap_data);
            }
        }

        event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
        Self::process_event(&mut event);
        callback(&event);

        Ok(())
    }

    fn should_handle(
        protocols: &[Protocol],
        _filter: Option<&EventTypeFilter>,
        program_id: &Pubkey,
    ) -> bool {
        if let Some(protocol) = EventDispatcher::match_protocol_by_program_id(program_id) {
            protocols.contains(&protocol)
        } else {
            EventDispatcher::is_compute_budget_program(program_id)
        }
    }

    fn process_event(event: &mut DexEvent) {
        match event {
            DexEvent::PumpFunTradeEvent(trade) => {
                if let Some(swap) = trade.metadata.swap_data.as_mut() {
                    let is_buy = trade.is_buy;
                    swap.from_amount = if is_buy { trade.sol_amount } else { trade.token_amount };
                    swap.to_amount = if is_buy { trade.token_amount } else { trade.sol_amount };
                }
            }
            DexEvent::PumpSwapBuyEvent(trade) => {
                if let Some(swap) = trade.metadata.swap_data.as_mut() {
                    swap.from_amount = trade.user_quote_amount_in;
                    swap.to_amount = trade.base_amount_out;
                }
            }
            DexEvent::PumpSwapSellEvent(trade) => {
                if let Some(swap) = trade.metadata.swap_data.as_mut() {
                    swap.from_amount = trade.base_amount_in;
                    swap.to_amount = trade.user_quote_amount_out;
                }
            }
            DexEvent::BonkTradeEvent(_trade) => {
                // Future Bonk event processing
            }
            _ => {}
        }
    }
}
