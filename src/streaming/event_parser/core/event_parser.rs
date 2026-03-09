use crate::streaming::event_parser::{
    DexEvent, Protocol, common::{
        EventMetadata, EventType, ProtocolType, high_performance_clock::elapsed_micros_since, parse_swap_data_from_next_grpc_instructions, parse_swap_data_from_next_instructions,
        ParseOptions, TransactionParams
    }, core::{
        dispatcher::EventDispatcher,
        merger_event::merge,
        traits::InstructionLike,
    }, protocols::raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID
};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::InnerInstructions;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;

pub struct EventParser {}

impl EventParser {
    // ================================================================================================
    // Public API - Entry Points
    // ================================================================================================

    /// Parse transaction from gRPC stream
    ///
    /// # Errors
    ///
    /// Returns an error if transaction data is corrupted or parsing fails.
    pub fn parse_grpc_transaction(
        options: ParseOptions<'_>,
        grpc_tx: SubscribeUpdateTransactionInfo,
        mut params: TransactionParams,
        callback: &mut impl FnMut(&DexEvent),
    ) -> anyhow::Result<()> {
        let Some(transaction) = grpc_tx.transaction else { return Ok(()) };
        let Some(message) = transaction.message else { return Ok(()) };

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

            params.recent_blockhash = if message.recent_blockhash.is_empty() {
                None
            } else {
                Some(solana_sdk::bs58::encode(&message.recent_blockhash).into_string())
            };

            Self::parse_instruction_events_from_grpc_transaction(
                options,
                &message.instructions,
                &accounts,
                &meta.inner_instructions,
                &params,
                callback,
            );
        }

        Ok(())
    }

    /// Parse transaction from `VersionedTransaction`
    ///
    /// # Errors
    ///
    /// Returns an error if transaction data is corrupted or parsing fails.
    pub fn parse_instruction_events_from_versioned_transaction(
        options: ParseOptions<'_>,
        transaction: &VersionedTransaction,
        params: TransactionParams,
        accounts: &[Pubkey],
        inner_instructions: &[InnerInstructions],
        callback: &mut impl FnMut(&DexEvent),
    ) -> anyhow::Result<()> {
        if !accounts.iter().any(|account| Self::should_handle(options, account)) {
            return Ok(());
        }

        let mut current_params = params;
        current_params.recent_blockhash = Some(transaction.message.recent_blockhash().to_string());
        let compiled_instructions = transaction.message.instructions();

        for (index, instruction) in compiled_instructions.iter().enumerate() {
            if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                if !Self::should_handle(options, program_id) {
                    continue;
                }

                let inner_ix = inner_instructions.iter().find(|inner| inner.index == u8::try_from(index).unwrap_or(u8::MAX));
                
                Self::parse_events_from_instruction_generic(
                    options,
                    instruction,
                    accounts,
                    current_params.clone(),
                    i64::try_from(index).unwrap_or(i64::MAX),
                    None,
                    inner_ix,
                    callback,
                );

                if let Some(inner_ix) = inner_ix {
                    for (inner_index, inner_instruction) in inner_ix.instructions.iter().enumerate() {
                        Self::parse_events_from_instruction_generic(
                            options,
                            &inner_instruction.instruction,
                            accounts,
                            current_params.clone(),
                            i64::try_from(index).unwrap_or(i64::MAX),
                            Some(i64::try_from(inner_index).unwrap_or(i64::MAX)),
                            Some(inner_ix),
                            callback,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    // ================================================================================================
    // gRPC Transaction Processing
    // ================================================================================================

    fn parse_instruction_events_from_grpc_transaction(
        options: ParseOptions<'_>,
        compiled_instructions: &[yellowstone_grpc_proto::prelude::CompiledInstruction],
        accounts: &[Pubkey],
        inner_instructions: &[yellowstone_grpc_proto::prelude::InnerInstructions],
        params: &TransactionParams,
        callback: &mut impl FnMut(&DexEvent),
    ) {
        if !accounts.iter().any(|account| Self::should_handle(options, account)) {
            return;
        }

        for (index, instruction) in compiled_instructions.iter().enumerate() {
            if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                if !Self::should_handle(options, program_id) {
                    continue;
                }

                let inner_ix = inner_instructions.iter().find(|inner| inner.index == u32::try_from(index).unwrap_or(u32::MAX));
                
                Self::parse_events_from_grpc_instruction_generic(
                    options,
                    instruction,
                    accounts,
                    params.clone(),
                    i64::try_from(index).unwrap_or(i64::MAX),
                    None,
                    inner_ix,
                    callback,
                );

                if let Some(inner_ix) = inner_ix {
                    for (inner_index, inner_instruction) in inner_ix.instructions.iter().enumerate() {
                        Self::parse_events_from_grpc_instruction_generic(
                            options,
                            inner_instruction,
                            accounts,
                            params.clone(),
                            i64::try_from(index).unwrap_or(i64::MAX),
                            Some(i64::try_from(inner_index).unwrap_or(i64::MAX)),
                            Some(inner_ix),
                            callback,
                        );
                    }
                }
            }
        }
    }

    // ================================================================================================
    // Generic Instruction Processing (The "Rust Pro" Consolidation)
    // ================================================================================================

    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_instruction_generic(
        options: ParseOptions<'_>,
        instruction: &impl InstructionLike,
        accounts: &[Pubkey],
        params: TransactionParams,
        outer_index: i64,
        inner_index: Option<i64>,
        inner_instructions: Option<&InnerInstructions>,
        callback: &mut impl FnMut(&DexEvent),
    ) {
        let program_id_index = instruction.program_id_index();
        if program_id_index >= accounts.len() { return ; }
        let program_id = accounts[program_id_index];
        
        Self::execute_parsing(
            options,
            program_id,
            instruction.data(),
            instruction.accounts(),
            accounts,
            params,
            outer_index,
            inner_index,
            |event, idx, accounts| parse_swap_data_from_next_instructions(event, inner_instructions?, i8::try_from(idx).unwrap_or(i8::MAX), accounts),
            |protocol, _discriminator, _instruction_data, meta| {
                let inner_instrs = inner_instructions?;
                let current_idx = i32::try_from(inner_index.unwrap_or(-1)).unwrap_or(i32::MAX);
                for (idx, inner_ix) in inner_instrs.instructions.iter().enumerate() {
                    if i32::try_from(idx).unwrap_or(i32::MAX) <= current_idx { continue; }
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
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_grpc_instruction_generic(
        options: ParseOptions<'_>,
        instruction: &impl InstructionLike,
        accounts: &[Pubkey],
        params: TransactionParams,
        outer_index: i64,
        inner_index: Option<i64>,
        inner_instructions: Option<&yellowstone_grpc_proto::prelude::InnerInstructions>,
        callback: &mut impl FnMut(&DexEvent),
    ) {
        let program_id_index = instruction.program_id_index();
        if program_id_index >= accounts.len() { return ; }
        let program_id = accounts[program_id_index];

        Self::execute_parsing(
            options,
            program_id,
            instruction.data(),
            instruction.accounts(),
            accounts,
            params,
            outer_index,
            inner_index,
            |event, idx, accounts| parse_swap_data_from_next_grpc_instructions(event, inner_instructions?, i8::try_from(idx).unwrap_or(i8::MAX), accounts),
            |protocol, _discriminator, _instruction_data, meta| {
                let inner_instrs = inner_instructions?;
                let current_idx = i32::try_from(inner_index.unwrap_or(-1)).unwrap_or(i32::MAX);
                for (idx, inner_ix) in inner_instrs.instructions.iter().enumerate() {
                    if i32::try_from(idx).unwrap_or(i32::MAX) <= current_idx { continue; }
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
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_parsing(
        options: ParseOptions<'_>,
        program_id: Pubkey,
        data: &[u8],
        instruction_accounts: &[u8],
        all_accounts: &[Pubkey],
        params: TransactionParams,
        outer_index: i64,
        inner_index: Option<i64>,
        swap_parser: impl FnOnce(&DexEvent, i64, &[Pubkey]) -> Option<crate::streaming::event_parser::common::SwapData>,
        inner_event_finder: impl FnOnce(Protocol, &[u8], &[u8], EventMetadata) -> Option<DexEvent>,
        callback: &mut impl FnMut(&DexEvent),
    ) {
        if !Self::should_handle(options, &program_id) { return; }

        let is_cu_program = EventDispatcher::is_compute_budget_program(&program_id);
        let discriminator_length = if program_id == RAYDIUM_AMM_V4_PROGRAM_ID { 1 } else { 8 };

        if !is_cu_program && data.len() < discriminator_length { return; }

        let timestamp = params.block_time.unwrap_or(prost_types::Timestamp { seconds: 0, nanos: 0 });
        let metadata = EventMetadata::new(
            params.signature, params.slot, timestamp.seconds, 
            timestamp.seconds * 1000 + i64::from(timestamp.nanos) / 1_000_000,
            ProtocolType::default(), EventType::default(), program_id,
            outer_index, inner_index, params.recv_us, params.tx_index, params.recent_blockhash
        );

        if is_cu_program {
            if let Some(event) = EventDispatcher::dispatch_compute_budget_instruction(data, metadata) {
                callback(&event);
            }
            return;
        }

        let Some(protocol) = EventDispatcher::match_protocol_by_program_id(&program_id) else {
            return;
        };

        let Some(mut event) = crate::streaming::event_parser::core::parser_cache::with_account_pubkeys_cache(
            instruction_accounts, all_accounts,
            |accounts| EventDispatcher::dispatch_instruction(protocol, &data[..discriminator_length], &data[discriminator_length..], accounts, metadata.clone())
        ) else {
            return;
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

        event.metadata_mut().handle_us = elapsed_micros_since(params.recv_us);
        Self::process_event(&mut event);
        callback(&event);
    }

    fn should_handle(
        options: ParseOptions<'_>,
        program_id: &Pubkey,
    ) -> bool {
        EventDispatcher::match_protocol_by_program_id(program_id).map_or_else(
            || EventDispatcher::is_compute_budget_program(program_id),
            |protocol| options.protocols.contains(&protocol),
        )
    }

    const fn process_event(event: &mut DexEvent) {
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
