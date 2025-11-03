use crate::streaming::{
    common::SimdUtils,
    event_parser::{
        common::{
            filter::EventTypeFilter,
            high_performance_clock::{elapsed_micros_since, get_high_perf_clock},
            parse_swap_data_from_next_grpc_instructions, parse_swap_data_from_next_instructions,
            EventMetadata,
        },
        core::{
            dispatcher::EventDispatcher,
            global_state::{
                add_bonk_dev_address, add_dev_address, is_bonk_dev_address_in_signature,
                is_dev_address_in_signature,
            },
            parser_cache::{
                build_account_pubkeys_with_cache,
                get_global_program_ids,
            },
        },
        DexEvent, Protocol,
    },
};
use prost_types::Timestamp;
use solana_sdk::{
    bs58, message::compiled_instruction::CompiledInstruction, pubkey::Pubkey, signature::Signature,
    transaction::VersionedTransaction,
};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, InnerInstruction, InnerInstructions, UiInstruction,
};
use std::sync::Arc;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;

pub struct EventParser {}

impl EventParser {
    // 辅助函数：确保 accounts 数组有足够的容量
    fn ensure_accounts_capacity(accounts: &mut Vec<Pubkey>, instruction_accounts: &[u8]) {
        if let Some(&max_idx) = instruction_accounts.iter().max() {
            let required_len = max_idx as usize + 1;
            if required_len > accounts.len() {
                accounts.resize(required_len, Pubkey::default());
            }
        }
    }

    // 辅助函数：转换 timestamp 为毫秒
    fn timestamp_to_millis(block_time: Option<Timestamp>) -> (Timestamp, i64) {
        let timestamp = block_time.unwrap_or(Timestamp { seconds: 0, nanos: 0 });
        let millis = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;
        (timestamp, millis)
    }

    // 辅助函数：创建回调适配器
    fn create_callback_adapter(
        callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    ) -> Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync> {
        Arc::new(move |event: &DexEvent| {
            callback(event.clone());
        })
    }

    #[allow(clippy::too_many_arguments)]
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
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 获取交易的指令和账户
        let mut accounts = accounts.to_vec();
        // 检查交易中是否包含程序
        let has_program = accounts
            .iter()
            .any(|account| Self::should_handle(protocols, event_type_filter, account));
        if has_program {
            // 解析每个指令
            for (index, instruction) in compiled_instructions.iter().enumerate() {
                if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                    let program_id = *program_id; // 克隆程序ID，避免借用冲突
                    let inner_instructions = inner_instructions
                        .iter()
                        .find(|inner_instruction| inner_instruction.index == index as u32);
                    // 补齐accounts(使用Pubkey::default())
                    Self::ensure_accounts_capacity(&mut accounts, &instruction.accounts);
                    if Self::should_handle(protocols, event_type_filter, &program_id) {
                        Self::parse_events_from_grpc_instruction(
                            protocols,
                            event_type_filter,
                            instruction,
                            &accounts,
                            signature,
                            slot.unwrap_or(0),
                            block_time,
                            recv_us,
                            index as i64,
                            None,
                            bot_wallet,
                            transaction_index,
                            inner_instructions,
                            callback.clone(),
                        )?;
                    }
                    // Immediately process inner instructions for correct ordering
                    if let Some(inner_instructions) = inner_instructions {
                        for (inner_index, inner_instruction) in
                            inner_instructions.instructions.iter().enumerate()
                        {
                            let inner_accounts = &inner_instruction.accounts;
                            let data = &inner_instruction.data;
                            let instruction =
                                yellowstone_grpc_proto::prelude::CompiledInstruction {
                                    program_id_index: inner_instruction.program_id_index,
                                    accounts: inner_accounts.to_vec(),
                                    data: data.to_vec(),
                                };
                            Self::parse_events_from_grpc_instruction(
                                protocols,
                                event_type_filter,
                                &instruction,
                                &accounts,
                                signature,
                                slot.unwrap_or(0),
                                block_time,
                                recv_us,
                                inner_instructions.index as i64,
                                Some(inner_index as i64),
                                bot_wallet,
                                transaction_index,
                                Some(&inner_instructions),
                                callback.clone(),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 从VersionedTransaction中解析指令事件的通用方法
    #[allow(clippy::too_many_arguments)]
    async fn parse_instruction_events_from_versioned_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        transaction: &VersionedTransaction,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        accounts: &[Pubkey],
        inner_instructions: &[InnerInstructions],
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 获取交易的指令和账户
        let compiled_instructions = transaction.message.instructions();
        let mut accounts: Vec<Pubkey> = accounts.to_vec();
        // 检查交易中是否包含程序
        let has_program = accounts
            .iter()
            .any(|account| Self::should_handle(protocols, event_type_filter, account));
        if has_program {
            // 解析每个指令
            for (index, instruction) in compiled_instructions.iter().enumerate() {
                if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                    let program_id = *program_id; // 克隆程序ID，避免借用冲突
                    let inner_instructions = inner_instructions
                        .iter()
                        .find(|inner_instruction| inner_instruction.index == index as u8);
                    // 补齐accounts(使用Pubkey::default())
                    Self::ensure_accounts_capacity(&mut accounts, &instruction.accounts);
                    if Self::should_handle(protocols, event_type_filter, &program_id) {
                        Self::parse_events_from_instruction(
                            protocols,
                            event_type_filter,
                            instruction,
                            &accounts,
                            signature,
                            slot.unwrap_or(0),
                            block_time,
                            recv_us,
                            index as i64,
                            None,
                            bot_wallet,
                            transaction_index,
                            inner_instructions,
                            callback.clone(),
                        )?;
                    }
                    // Immediately process inner instructions for correct ordering
                    if let Some(inner_instructions) = inner_instructions {
                        for (inner_index, inner_instruction) in
                            inner_instructions.instructions.iter().enumerate()
                        {
                            Self::parse_events_from_instruction(
                                protocols,
                                event_type_filter,
                                &inner_instruction.instruction,
                                &accounts,
                                signature,
                                slot.unwrap_or(0),
                                block_time,
                                recv_us,
                                index as i64,
                                Some(inner_index as i64),
                                bot_wallet,
                                transaction_index,
                                Some(&inner_instructions),
                                callback.clone(),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn parse_versioned_transaction_owned(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        versioned_tx: VersionedTransaction,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        inner_instructions: &[InnerInstructions],
        callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        let adapter_callback = Self::create_callback_adapter(callback);
        let accounts = versioned_tx.message.static_account_keys();
        Self::parse_instruction_events_from_versioned_transaction(
            protocols,
            event_type_filter,
            &versioned_tx,
            signature,
            slot,
            block_time,
            recv_us,
            accounts,
            inner_instructions,
            bot_wallet,
            transaction_index,
            adapter_callback,
        )
        .await
    }

    pub async fn parse_grpc_transaction_owned(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        grpc_tx: SubscribeUpdateTransactionInfo,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn Fn(DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        let adapter_callback = Self::create_callback_adapter(callback);
        Self::parse_grpc_transaction(
            protocols,
            event_type_filter,
            grpc_tx,
            signature,
            slot,
            block_time,
            recv_us,
            bot_wallet,
            transaction_index,
            adapter_callback,
        )
        .await
    }

    async fn parse_grpc_transaction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        grpc_tx: SubscribeUpdateTransactionInfo,
        signature: Signature,
        slot: Option<u64>,
        block_time: Option<Timestamp>,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        if let Some(transition) = grpc_tx.transaction {
            if let Some(message) = &transition.message {
                let mut address_table_lookups: Vec<Vec<u8>> = vec![];
                let mut inner_instructions: Vec<
                    yellowstone_grpc_proto::solana::storage::confirmed_block::InnerInstructions,
                > = vec![];

                if let Some(meta) = grpc_tx.meta {
                    inner_instructions = meta.inner_instructions;
                    address_table_lookups.reserve(
                        meta.loaded_writable_addresses.len() + meta.loaded_readonly_addresses.len(),
                    );
                    let loaded_writable_addresses = meta.loaded_writable_addresses;
                    let loaded_readonly_addresses = meta.loaded_readonly_addresses;
                    address_table_lookups.extend(
                        loaded_writable_addresses.into_iter().chain(loaded_readonly_addresses),
                    );
                }

                let mut accounts_bytes: Vec<Vec<u8>> =
                    Vec::with_capacity(message.account_keys.len() + address_table_lookups.len());
                accounts_bytes.extend_from_slice(&message.account_keys);
                accounts_bytes.extend(address_table_lookups);
                // 转换为 Pubkey
                let accounts: Vec<Pubkey> = accounts_bytes
                    .iter()
                    .filter_map(|account| {
                        if account.len() == 32 {
                            Some(Pubkey::try_from(account.as_slice()).unwrap_or_default())
                        } else {
                            None
                        }
                    })
                    .collect();
                // 解析指令事件
                let instructions = &message.instructions;
                Self::parse_instruction_events_from_grpc_transaction(
                    protocols,
                    event_type_filter,
                    &instructions,
                    signature,
                    slot,
                    block_time,
                    recv_us,
                    &accounts,
                    &inner_instructions,
                    bot_wallet,
                    transaction_index,
                    callback,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn parse_encoded_confirmed_transaction_with_status_meta(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        signature: Signature,
        transaction: EncodedConfirmedTransactionWithStatusMeta,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        let versioned_tx = match transaction.transaction.transaction.decode() {
            Some(tx) => tx,
            None => {
                return Ok(());
            }
        };
        let mut inner_instructions_vec: Vec<InnerInstructions> = Vec::new();
        if let Some(meta) = &transaction.transaction.meta {
            // 从meta中获取inner_instructions，处理OptionSerializer类型
            if let solana_transaction_status::option_serializer::OptionSerializer::Some(
                ui_inner_insts,
            ) = &meta.inner_instructions
            {
                // 将UiInnerInstructions转换为InnerInstructions
                for ui_inner in ui_inner_insts {
                    let mut converted_instructions = Vec::new();

                    // 转换每个UiInstruction为InnerInstruction
                    for ui_instruction in &ui_inner.instructions {
                        if let UiInstruction::Compiled(ui_compiled) = ui_instruction {
                            // 解码base58编码的data
                            if let Ok(data) = bs58::decode(&ui_compiled.data).into_vec() {
                                // base64解码
                                let compiled_instruction = CompiledInstruction {
                                    program_id_index: ui_compiled.program_id_index,
                                    accounts: ui_compiled.accounts.to_vec(),
                                    data,
                                };

                                let inner_instruction = InnerInstruction {
                                    instruction: compiled_instruction,
                                    stack_height: ui_compiled.stack_height,
                                };

                                converted_instructions.push(inner_instruction);
                            }
                        }
                    }

                    let inner_instructions = InnerInstructions {
                        index: ui_inner.index,
                        instructions: converted_instructions,
                    };

                    inner_instructions_vec.push(inner_instructions);
                }
            }
        }
        let inner_instructions: &[InnerInstructions] = &inner_instructions_vec;

        let meta = transaction.transaction.meta;
        let mut address_table_lookups: Vec<Pubkey> = vec![];
        if let Some(meta) = meta {
            if let solana_transaction_status::option_serializer::OptionSerializer::Some(
                loaded_addresses,
            ) = &meta.loaded_addresses
            {
                address_table_lookups
                    .reserve(loaded_addresses.writable.len() + loaded_addresses.readonly.len());
                address_table_lookups.extend(
                    loaded_addresses
                        .writable
                        .iter()
                        .filter_map(|s| s.parse::<Pubkey>().ok())
                        .chain(
                            loaded_addresses
                                .readonly
                                .iter()
                                .filter_map(|s| s.parse::<Pubkey>().ok()),
                        ),
                );
            }
        }
        let mut accounts = Vec::with_capacity(
            versioned_tx.message.static_account_keys().len() + address_table_lookups.len(),
        );
        accounts.extend_from_slice(versioned_tx.message.static_account_keys());
        accounts.extend(address_table_lookups);

        let slot = transaction.slot;
        let block_time = transaction.block_time.map(|t| Timestamp { seconds: t as i64, nanos: 0 });
        let recv_us = get_high_perf_clock();
        let bot_wallet = None;
        let transaction_index = None;
        // 解析指令事件
        Self::parse_instruction_events_from_versioned_transaction(
            protocols,
            event_type_filter,
            &versioned_tx,
            signature,
            Some(slot),
            block_time,
            recv_us,
            &accounts,
            inner_instructions,
            bot_wallet,
            transaction_index,
            callback,
        )
        .await
    }

    /// 从指令中解析事件
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_instruction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        inner_instructions: Option<&InnerInstructions>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 验证 program_id
        let program_id = match Self::validate_program_id(
            instruction.program_id_index as u32,
            accounts,
            protocols,
            event_type_filter,
        ) {
            Some(id) => id,
            None => return Ok(()),
        };

        // 验证并构建账户公钥列表
        let account_pubkeys = match Self::validate_and_build_account_pubkeys(&instruction.accounts, accounts) {
            Some(pubkeys) => pubkeys,
            None => return Ok(()),
        };

        // 提取判别器
        let (instruction_discriminator, instruction_data) =
            match Self::extract_instruction_discriminator(&instruction.data) {
                Some(result) => result,
                None => return Ok(()),
            };

        // 准备 inner instruction 数据（如果有）
        let (inner_discriminator, inner_data) = if let Some(inner_instructions_ref) = inner_instructions {
            // 尝试从第一个 inner instruction 获取数据
            if let Some(first_inner) = inner_instructions_ref.instructions.first() {
                let inner_inst_data = &first_inner.instruction.data;
                if inner_inst_data.len() >= 24 {  // 16 bytes discriminator + data
                    (Some(&inner_inst_data[0..16]), Some(&inner_inst_data[16..]))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // 创建元数据（使用默认值，dispatcher会返回带正确元数据的事件）
        let (timestamp, block_time_ms) = Self::timestamp_to_millis(block_time);
        let metadata = EventMetadata::new(
            signature,
            slot,
            timestamp.seconds,
            block_time_ms,
            crate::streaming::event_parser::common::ProtocolType::Common,
            crate::streaming::event_parser::common::EventType::default(),
            program_id,
            outer_index,
            inner_index,
            recv_us,
            transaction_index,
        );

        // 使用 dispatcher 解析事件
        if let Some(mut event) = EventDispatcher::dispatch_multi(
            protocols,
            &program_id,
            instruction_discriminator,
            inner_discriminator,
            instruction_data,
            inner_data,
            &account_pubkeys,
            metadata,
        ) {
            // 解析 swap_data（如果需要）
            if let Some(inner_instructions_ref) = inner_instructions {
                if event.metadata().swap_data.is_none() {
                    if let Some(swap_data) = parse_swap_data_from_next_instructions(
                        &event,
                        inner_instructions_ref,
                        inner_index.unwrap_or(-1_i64) as i8,
                        accounts,
                    ) {
                        event.metadata_mut().set_swap_data(swap_data);
                    }
                }
            }

            // 完成事件处理
            Self::finalize_event(event, recv_us, bot_wallet, &callback);
        }
        Ok(())
    }

    /// 从指令中解析事件 (gRPC版本)
    #[allow(clippy::too_many_arguments)]
    fn parse_events_from_grpc_instruction(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        instruction: &yellowstone_grpc_proto::prelude::CompiledInstruction,
        accounts: &[Pubkey],
        signature: Signature,
        slot: u64,
        block_time: Option<Timestamp>,
        recv_us: i64,
        outer_index: i64,
        inner_index: Option<i64>,
        bot_wallet: Option<Pubkey>,
        transaction_index: Option<u64>,
        inner_instructions: Option<&yellowstone_grpc_proto::prelude::InnerInstructions>,
        callback: Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) -> anyhow::Result<()> {
        // 验证 program_id
        let program_id = match Self::validate_program_id(
            instruction.program_id_index,
            accounts,
            protocols,
            event_type_filter,
        ) {
            Some(id) => id,
            None => return Ok(()),
        };

        // 验证并构建账户公钥列表
        let account_pubkeys = match Self::validate_and_build_account_pubkeys(&instruction.accounts, accounts) {
            Some(pubkeys) => pubkeys,
            None => return Ok(()),
        };

        // 提取判别器
        let (instruction_discriminator, instruction_data) =
            match Self::extract_instruction_discriminator(&instruction.data) {
                Some(result) => result,
                None => return Ok(()),
            };

        // 准备 inner instruction 数据（如果有）
        let (inner_discriminator, inner_data) = if let Some(inner_instructions_ref) = inner_instructions {
            // 尝试从第一个 inner instruction 获取数据
            if let Some(first_inner) = inner_instructions_ref.instructions.first() {
                let inner_inst_data = &first_inner.data;
                if inner_inst_data.len() >= 24 {  // 16 bytes discriminator + data
                    (Some(&inner_inst_data[0..16]), Some(&inner_inst_data[16..]))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // 创建元数据（使用默认值，dispatcher会返回带正确元数据的事件）
        let (timestamp, block_time_ms) = Self::timestamp_to_millis(block_time);
        let metadata = EventMetadata::new(
            signature,
            slot,
            timestamp.seconds,
            block_time_ms,
            crate::streaming::event_parser::common::ProtocolType::Common,
            crate::streaming::event_parser::common::EventType::default(),
            program_id,
            outer_index,
            inner_index,
            recv_us,
            transaction_index,
        );

        // 使用 dispatcher 解析事件
        if let Some(mut event) = EventDispatcher::dispatch_multi(
            protocols,
            &program_id,
            instruction_discriminator,
            inner_discriminator,
            instruction_data,
            inner_data,
            &account_pubkeys,
            metadata,
        ) {
            // 解析 swap_data（如果需要）
            if let Some(inner_instructions_ref) = inner_instructions {
                if event.metadata().swap_data.is_none() {
                    if let Some(swap_data) = parse_swap_data_from_next_grpc_instructions(
                        &event,
                        inner_instructions_ref,
                        inner_index.unwrap_or(-1_i64) as i8,
                        accounts,
                    ) {
                        event.metadata_mut().set_swap_data(swap_data);
                    }
                }
            }

            // 完成事件处理
            Self::finalize_event(event, recv_us, bot_wallet, &callback);
        }
        Ok(())
    }

    fn should_handle(
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
        program_id: &Pubkey,
    ) -> bool {
        get_global_program_ids(protocols, event_type_filter).contains(program_id)
    }

    // 辅助函数：设置 swap_data 的 from_amount 和 to_amount
    fn set_swap_amounts(
        swap_data: &mut crate::streaming::event_parser::common::SwapData,
        from_amount: u64,
        to_amount: u64,
    ) {
        swap_data.from_amount = from_amount;
        swap_data.to_amount = to_amount;
    }

    // 辅助函数：验证 program_id 是否在 accounts 范围内并且应该被处理
    fn validate_program_id(
        program_id_index: u32,
        accounts: &[Pubkey],
        protocols: &[Protocol],
        event_type_filter: Option<&EventTypeFilter>,
    ) -> Option<Pubkey> {
        let index = program_id_index as usize;
        if index >= accounts.len() {
            return None;
        }
        let program_id = accounts[index];
        if Self::should_handle(protocols, event_type_filter, &program_id) {
            Some(program_id)
        } else {
            None
        }
    }

    // 辅助函数：验证并构建账户公钥列表
    fn validate_and_build_account_pubkeys(
        instruction_accounts: &[u8],
        accounts: &[Pubkey],
    ) -> Option<Vec<Pubkey>> {
        if !SimdUtils::validate_account_indices_simd(instruction_accounts, accounts.len()) {
            return None;
        }
        Some(build_account_pubkeys_with_cache(instruction_accounts, accounts))
    }

    // 辅助函数：提取指令数据的判别器
    fn extract_instruction_discriminator(instruction_data: &[u8]) -> Option<(&[u8], &[u8])> {
        if instruction_data.len() < 8 {
            return None;
        }
        Some((&instruction_data[0..8], &instruction_data[8..]))
    }

    // 辅助函数：处理事件的最后步骤（设置时间、处理事件、调用回调）
    fn finalize_event(
        mut event: DexEvent,
        recv_us: i64,
        bot_wallet: Option<Pubkey>,
        callback: &Arc<dyn for<'a> Fn(&'a DexEvent) + Send + Sync>,
    ) {
        event.metadata_mut().handle_us = elapsed_micros_since(recv_us);
        event = Self::process_event(event, bot_wallet);
        callback(&event);
    }

    fn process_event(event: DexEvent, bot_wallet: Option<Pubkey>) -> DexEvent {
        let signature = event.metadata().signature; // Copy the signature to avoid borrowing issues
        match event {
            DexEvent::PumpFunCreateTokenEvent(token_info) => {
                add_dev_address(&signature, token_info.user);
                if token_info.creator != Pubkey::default() && token_info.creator != token_info.user
                {
                    add_dev_address(&signature, token_info.creator);
                }
                DexEvent::PumpFunCreateTokenEvent(token_info)
            }
            DexEvent::PumpFunTradeEvent(mut trade_info) => {
                trade_info.is_dev_create_token_trade =
                    is_dev_address_in_signature(&signature, &trade_info.user)
                        || is_dev_address_in_signature(&signature, &trade_info.creator);
                trade_info.is_bot = Some(trade_info.user) == bot_wallet;

                if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                    let (from, to) = if trade_info.is_buy {
                        (trade_info.sol_amount, trade_info.token_amount)
                    } else {
                        (trade_info.token_amount, trade_info.sol_amount)
                    };
                    Self::set_swap_amounts(swap_data, from, to);
                }
                DexEvent::PumpFunTradeEvent(trade_info)
            }
            DexEvent::PumpSwapBuyEvent(mut trade_info) => {
                if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                    Self::set_swap_amounts(
                        swap_data,
                        trade_info.user_quote_amount_in,
                        trade_info.base_amount_out,
                    );
                }
                DexEvent::PumpSwapBuyEvent(trade_info)
            }
            DexEvent::PumpSwapSellEvent(mut trade_info) => {
                if let Some(swap_data) = trade_info.metadata.swap_data.as_mut() {
                    Self::set_swap_amounts(
                        swap_data,
                        trade_info.base_amount_in,
                        trade_info.user_quote_amount_out,
                    );
                }
                DexEvent::PumpSwapSellEvent(trade_info)
            }
            DexEvent::BonkPoolCreateEvent(pool_info) => {
                add_bonk_dev_address(&signature, pool_info.creator);
                DexEvent::BonkPoolCreateEvent(pool_info)
            }
            DexEvent::BonkTradeEvent(mut trade_info) => {
                trade_info.is_dev_create_token_trade =
                    is_bonk_dev_address_in_signature(&signature, &trade_info.payer);
                trade_info.is_bot = Some(trade_info.payer) == bot_wallet;
                DexEvent::BonkTradeEvent(trade_info)
            }
            _ => event,
        }
    }
}
