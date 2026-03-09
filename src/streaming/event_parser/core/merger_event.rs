use crate::streaming::event_parser::DexEvent;

#[allow(clippy::too_many_lines)]
pub fn merge(instruction_event: &mut DexEvent, cpi_log_event: DexEvent) {
    match instruction_event {
        // PumpFun events
        DexEvent::PumpFunTradeEvent(event) => if let DexEvent::PumpFunTradeEvent(cpi_event) = cpi_log_event {
                event.mint = cpi_event.mint;
                event.sol_amount = cpi_event.sol_amount;
                event.token_amount = cpi_event.token_amount;
                event.is_buy = cpi_event.is_buy;
                event.user = cpi_event.user;
                event.timestamp = cpi_event.timestamp;
                event.virtual_sol_reserves = cpi_event.virtual_sol_reserves;
                event.virtual_token_reserves = cpi_event.virtual_token_reserves;
                event.real_sol_reserves = cpi_event.real_sol_reserves;
                event.real_token_reserves = cpi_event.real_token_reserves;
                event.fee_recipient = cpi_event.fee_recipient;
                event.fee_basis_points = cpi_event.fee_basis_points;
                event.fee = cpi_event.fee;
                event.creator = cpi_event.creator;
                event.creator_fee_basis_points = cpi_event.creator_fee_basis_points;
                event.creator_fee = cpi_event.creator_fee;
                event.track_volume = cpi_event.track_volume;
                event.total_unclaimed_tokens = cpi_event.total_unclaimed_tokens;
                event.total_claimed_tokens = cpi_event.total_claimed_tokens;
                event.current_sol_volume = cpi_event.current_sol_volume;
                event.last_update_timestamp = cpi_event.last_update_timestamp;
                event.ix_name.clone_from(&cpi_event.ix_name);
                event.mayhem_mode = cpi_event.mayhem_mode;
                event.cashback_fee_basis_points = cpi_event.cashback_fee_basis_points;
                event.cashback = cpi_event.cashback;
                event.is_cashback_coin = cpi_event.is_cashback_coin;
                if cpi_event.account.is_some() {
                    event.account = cpi_event.account;
                }
            },
        DexEvent::PumpFunCreateTokenEvent(event) => if let DexEvent::PumpFunCreateV2TokenEvent(cpi_event) = cpi_log_event {
                if cpi_event.mint != solana_sdk::pubkey::Pubkey::default() {
                    event.mint = cpi_event.mint;
                }
                if cpi_event.bonding_curve != solana_sdk::pubkey::Pubkey::default() {
                    event.bonding_curve = cpi_event.bonding_curve;
                }
                if cpi_event.user != solana_sdk::pubkey::Pubkey::default() {
                    event.user = cpi_event.user;
                }
                if cpi_event.creator != solana_sdk::pubkey::Pubkey::default() {
                    event.creator = cpi_event.creator;
                }
                if cpi_event.timestamp != 0 {
                    event.timestamp = cpi_event.timestamp;
                }
                if cpi_event.virtual_token_reserves != 0 {
                    event.virtual_token_reserves = cpi_event.virtual_token_reserves;
                }
                if cpi_event.virtual_sol_reserves != 0 {
                    event.virtual_sol_reserves = cpi_event.virtual_sol_reserves;
                }
                if cpi_event.real_token_reserves != 0 {
                    event.real_token_reserves = cpi_event.real_token_reserves;
                }
                if cpi_event.token_total_supply != 0 {
                    event.token_total_supply = cpi_event.token_total_supply;
                }
                if cpi_event.token_program != solana_sdk::pubkey::Pubkey::default() {
                    event.token_program = cpi_event.token_program;
                }
                event.is_mayhem_mode = cpi_event.is_mayhem_mode;
                event.is_cashback_enabled = cpi_event.is_cashback_enabled;
            },
        DexEvent::PumpFunCreateV2TokenEvent(event) => if let DexEvent::PumpFunCreateV2TokenEvent(cpi_event) = cpi_log_event {
                if cpi_event.mint != solana_sdk::pubkey::Pubkey::default() {
                    event.mint = cpi_event.mint;
                }
                if cpi_event.bonding_curve != solana_sdk::pubkey::Pubkey::default() {
                    event.bonding_curve = cpi_event.bonding_curve;
                }
                if cpi_event.user != solana_sdk::pubkey::Pubkey::default() {
                    event.user = cpi_event.user;
                }
                if cpi_event.creator != solana_sdk::pubkey::Pubkey::default() {
                    event.creator = cpi_event.creator;
                }
                if cpi_event.timestamp != 0 {
                    event.timestamp = cpi_event.timestamp;
                }
                if cpi_event.virtual_token_reserves != 0 {
                    event.virtual_token_reserves = cpi_event.virtual_token_reserves;
                }
                if cpi_event.virtual_sol_reserves != 0 {
                    event.virtual_sol_reserves = cpi_event.virtual_sol_reserves;
                }
                if cpi_event.real_token_reserves != 0 {
                    event.real_token_reserves = cpi_event.real_token_reserves;
                }
                if cpi_event.token_total_supply != 0 {
                    event.token_total_supply = cpi_event.token_total_supply;
                }
                if cpi_event.token_program != solana_sdk::pubkey::Pubkey::default() {
                    event.token_program = cpi_event.token_program;
                }
                event.is_mayhem_mode = cpi_event.is_mayhem_mode;
                event.is_cashback_enabled = cpi_event.is_cashback_enabled;
            },
        DexEvent::PumpFunMigrateEvent(event) => if let DexEvent::PumpFunMigrateEvent(cpi_event) = cpi_log_event {
                event.user = cpi_event.user;
                event.mint = cpi_event.mint;
                event.mint_amount = cpi_event.mint_amount;
                event.sol_amount = cpi_event.sol_amount;
                event.pool_migration_fee = cpi_event.pool_migration_fee;
                event.bonding_curve = cpi_event.bonding_curve;
                event.timestamp = cpi_event.timestamp;
                event.pool = cpi_event.pool;
            },

        // Bonk events
        DexEvent::BonkTradeEvent(event) => if let DexEvent::BonkTradeEvent(cpi_event) = cpi_log_event {
                event.pool_state = cpi_event.pool_state;
                event.total_base_sell = cpi_event.total_base_sell;
                event.virtual_base = cpi_event.virtual_base;
                event.virtual_quote = cpi_event.virtual_quote;
                event.real_base_before = cpi_event.real_base_before;
                event.real_quote_before = cpi_event.real_quote_before;
                event.real_base_after = cpi_event.real_base_after;
                event.real_quote_after = cpi_event.real_quote_after;
                event.amount_in = cpi_event.amount_in;
                event.amount_out = cpi_event.amount_out;
                event.protocol_fee = cpi_event.protocol_fee;
                event.platform_fee = cpi_event.platform_fee;
                event.creator_fee = cpi_event.creator_fee;
                event.share_fee = cpi_event.share_fee;
                event.trade_direction = cpi_event.trade_direction;
                event.pool_status = cpi_event.pool_status;
                event.exact_in = cpi_event.exact_in;
            },
        DexEvent::BonkPoolCreateEvent(event) => if let DexEvent::BonkPoolCreateEvent(cpi_event) = cpi_log_event {
                event.pool_state = cpi_event.pool_state;
                event.creator = cpi_event.creator;
                event.config = cpi_event.config;
                event.base_mint_param = cpi_event.base_mint_param;
                event.curve_param = cpi_event.curve_param;
                event.vesting_param = cpi_event.vesting_param;
                event.amm_fee_on = cpi_event.amm_fee_on;
            },
        DexEvent::BonkMigrateToAmmEvent(event) => if let DexEvent::BonkMigrateToAmmEvent(cpi_event) = cpi_log_event {
                event.base_lot_size = cpi_event.base_lot_size;
                event.quote_lot_size = cpi_event.quote_lot_size;
                event.market_vault_signer_nonce = cpi_event.market_vault_signer_nonce;
            },

        // PumpSwap events
        DexEvent::PumpSwapBuyEvent(event) => if let DexEvent::PumpSwapBuyEvent(cpi_event) = cpi_log_event {
                event.timestamp = cpi_event.timestamp;
                event.base_amount_out = cpi_event.base_amount_out;
                event.max_quote_amount_in = cpi_event.max_quote_amount_in;
                event.user_base_token_reserves = cpi_event.user_base_token_reserves;
                event.user_quote_token_reserves = cpi_event.user_quote_token_reserves;
                event.pool_base_token_reserves = cpi_event.pool_base_token_reserves;
                event.pool_quote_token_reserves = cpi_event.pool_quote_token_reserves;
                event.quote_amount_in = cpi_event.quote_amount_in;
                event.lp_fee_basis_points = cpi_event.lp_fee_basis_points;
                event.lp_fee = cpi_event.lp_fee;
                event.protocol_fee_basis_points = cpi_event.protocol_fee_basis_points;
                event.protocol_fee = cpi_event.protocol_fee;
                event.quote_amount_in_with_lp_fee = cpi_event.quote_amount_in_with_lp_fee;
                event.user_quote_amount_in = cpi_event.user_quote_amount_in;
                event.pool = cpi_event.pool;
                event.user = cpi_event.user;
                event.user_base_token_account = cpi_event.user_base_token_account;
                event.user_quote_token_account = cpi_event.user_quote_token_account;
                event.protocol_fee_recipient = cpi_event.protocol_fee_recipient;
                event.protocol_fee_recipient_token_account = cpi_event.protocol_fee_recipient_token_account;
                event.coin_creator = cpi_event.coin_creator;
                event.coin_creator_fee_basis_points = cpi_event.coin_creator_fee_basis_points;
                event.coin_creator_fee = cpi_event.coin_creator_fee;
                event.track_volume = cpi_event.track_volume;
                event.total_unclaimed_tokens = cpi_event.total_unclaimed_tokens;
                event.total_claimed_tokens = cpi_event.total_claimed_tokens;
                event.current_sol_volume = cpi_event.current_sol_volume;
                event.last_update_timestamp = cpi_event.last_update_timestamp;
            },
        DexEvent::PumpSwapSellEvent(event) => if let DexEvent::PumpSwapSellEvent(cpi_event) = cpi_log_event {
                event.timestamp = cpi_event.timestamp;
                event.base_amount_in = cpi_event.base_amount_in;
                event.min_quote_amount_out = cpi_event.min_quote_amount_out;
                event.user_base_token_reserves = cpi_event.user_base_token_reserves;
                event.user_quote_token_reserves = cpi_event.user_quote_token_reserves;
                event.pool_base_token_reserves = cpi_event.pool_base_token_reserves;
                event.pool_quote_token_reserves = cpi_event.pool_quote_token_reserves;
                event.quote_amount_out = cpi_event.quote_amount_out;
                event.lp_fee_basis_points = cpi_event.lp_fee_basis_points;
                event.lp_fee = cpi_event.lp_fee;
                event.protocol_fee_basis_points = cpi_event.protocol_fee_basis_points;
                event.protocol_fee = cpi_event.protocol_fee;
                event.quote_amount_out_without_lp_fee = cpi_event.quote_amount_out_without_lp_fee;
                event.user_quote_amount_out = cpi_event.user_quote_amount_out;
                event.pool = cpi_event.pool;
                event.user = cpi_event.user;
                event.user_base_token_account = cpi_event.user_base_token_account;
                event.user_quote_token_account = cpi_event.user_quote_token_account;
                event.protocol_fee_recipient = cpi_event.protocol_fee_recipient;
                event.protocol_fee_recipient_token_account = cpi_event.protocol_fee_recipient_token_account;
                event.coin_creator = cpi_event.coin_creator;
                event.coin_creator_fee_basis_points = cpi_event.coin_creator_fee_basis_points;
                event.coin_creator_fee = cpi_event.coin_creator_fee;
            },
        DexEvent::PumpSwapCreatePoolEvent(event) => if let DexEvent::PumpSwapCreatePoolEvent(cpi_event) = cpi_log_event {
                event.timestamp = cpi_event.timestamp;
                event.index = cpi_event.index;
                event.creator = cpi_event.creator;
                event.base_mint = cpi_event.base_mint;
                event.quote_mint = cpi_event.quote_mint;
                event.base_mint_decimals = cpi_event.base_mint_decimals;
                event.quote_mint_decimals = cpi_event.quote_mint_decimals;
                event.base_amount_in = cpi_event.base_amount_in;
                event.quote_amount_in = cpi_event.quote_amount_in;
                event.pool_base_amount = cpi_event.pool_base_amount;
                event.pool_quote_amount = cpi_event.pool_quote_amount;
                event.minimum_liquidity = cpi_event.minimum_liquidity;
                event.initial_liquidity = cpi_event.initial_liquidity;
                event.lp_token_amount_out = cpi_event.lp_token_amount_out;
                event.pool_bump = cpi_event.pool_bump;
                event.pool = cpi_event.pool;
                event.lp_mint = cpi_event.lp_mint;
                event.user_base_token_account = cpi_event.user_base_token_account;
                event.user_quote_token_account = cpi_event.user_quote_token_account;
                event.coin_creator = cpi_event.coin_creator;
            },
        DexEvent::PumpSwapDepositEvent(event) => if let DexEvent::PumpSwapDepositEvent(cpi_event) = cpi_log_event {
                event.timestamp = cpi_event.timestamp;
                event.lp_token_amount_out = cpi_event.lp_token_amount_out;
                event.max_base_amount_in = cpi_event.max_base_amount_in;
                event.max_quote_amount_in = cpi_event.max_quote_amount_in;
                event.user_base_token_reserves = cpi_event.user_base_token_reserves;
                event.user_quote_token_reserves = cpi_event.user_quote_token_reserves;
                event.pool_base_token_reserves = cpi_event.pool_base_token_reserves;
                event.pool_quote_token_reserves = cpi_event.pool_quote_token_reserves;
                event.base_amount_in = cpi_event.base_amount_in;
                event.quote_amount_in = cpi_event.quote_amount_in;
                event.lp_mint_supply = cpi_event.lp_mint_supply;
                event.pool = cpi_event.pool;
                event.user = cpi_event.user;
                event.user_base_token_account = cpi_event.user_base_token_account;
                event.user_quote_token_account = cpi_event.user_quote_token_account;
                event.user_pool_token_account = cpi_event.user_pool_token_account;
            },
        DexEvent::PumpSwapWithdrawEvent(event) => if let DexEvent::PumpSwapWithdrawEvent(cpi_event) = cpi_log_event {
                event.timestamp = cpi_event.timestamp;
                event.lp_token_amount_in = cpi_event.lp_token_amount_in;
                event.min_base_amount_out = cpi_event.min_base_amount_out;
                event.min_quote_amount_out = cpi_event.min_quote_amount_out;
                event.user_base_token_reserves = cpi_event.user_base_token_reserves;
                event.user_quote_token_reserves = cpi_event.user_quote_token_reserves;
                event.pool_base_token_reserves = cpi_event.pool_base_token_reserves;
                event.pool_quote_token_reserves = cpi_event.pool_quote_token_reserves;
                event.base_amount_out = cpi_event.base_amount_out;
                event.quote_amount_out = cpi_event.quote_amount_out;
                event.lp_mint_supply = cpi_event.lp_mint_supply;
                event.pool = cpi_event.pool;
                event.user = cpi_event.user;
                event.user_base_token_account = cpi_event.user_base_token_account;
                event.user_quote_token_account = cpi_event.user_quote_token_account;
                event.user_pool_token_account = cpi_event.user_pool_token_account;
            },
        DexEvent::MeteoraDammV2SwapEvent(event) => if let DexEvent::MeteoraDammV2SwapEvent(cpi_event) = cpi_log_event {
                event.pool = cpi_event.pool;
                event.trade_direction = cpi_event.trade_direction;
                event.collect_fee_mode = cpi_event.collect_fee_mode;
                event.has_referral = cpi_event.has_referral;
                event.amount_0 = cpi_event.amount_0;
                event.amount_1 = cpi_event.amount_1;
                event.swap_mode = cpi_event.swap_mode;
                event.included_fee_input_amount = cpi_event.included_fee_input_amount;
                event.excluded_fee_input_amount = cpi_event.excluded_fee_input_amount;
                event.amount_left = cpi_event.amount_left;
                event.output_amount = cpi_event.output_amount;
                event.next_sqrt_price = cpi_event.next_sqrt_price;
                event.trading_fee = cpi_event.trading_fee;
                event.partner_fee = cpi_event.partner_fee;
                event.referral_fee = cpi_event.referral_fee;
                event.included_transfer_fee_amount_in = cpi_event.included_transfer_fee_amount_in;
                event.included_transfer_fee_amount_out = cpi_event.included_transfer_fee_amount_out;
                event.excluded_transfer_fee_amount_out = cpi_event.excluded_transfer_fee_amount_out;
                event.current_timestamp = cpi_event.current_timestamp;
                event.reserve_a_amount = cpi_event.reserve_a_amount;
                event.reserve_b_amount = cpi_event.reserve_b_amount;
            },
        DexEvent::MeteoraDammV2Swap2Event(event) => if let DexEvent::MeteoraDammV2Swap2Event(cpi_event) = cpi_log_event {
                event.pool = cpi_event.pool;
                event.trade_direction = cpi_event.trade_direction;
                event.collect_fee_mode = cpi_event.collect_fee_mode;
                event.has_referral = cpi_event.has_referral;
                event.amount_0 = cpi_event.amount_0;
                event.amount_1 = cpi_event.amount_1;
                event.swap_mode = cpi_event.swap_mode;
                event.included_fee_input_amount = cpi_event.included_fee_input_amount;
                event.excluded_fee_input_amount = cpi_event.excluded_fee_input_amount;
                event.amount_left = cpi_event.amount_left;
                event.output_amount = cpi_event.output_amount;
                event.next_sqrt_price = cpi_event.next_sqrt_price;
                event.trading_fee = cpi_event.trading_fee;
                event.partner_fee = cpi_event.partner_fee;
                event.referral_fee = cpi_event.referral_fee;
                event.included_transfer_fee_amount_in = cpi_event.included_transfer_fee_amount_in;
                event.included_transfer_fee_amount_out = cpi_event.included_transfer_fee_amount_out;
                event.excluded_transfer_fee_amount_out = cpi_event.excluded_transfer_fee_amount_out;
                event.current_timestamp = cpi_event.current_timestamp;
                event.reserve_a_amount = cpi_event.reserve_a_amount;
                event.reserve_b_amount = cpi_event.reserve_b_amount;
            },
        DexEvent::MeteoraDammV2InitializePoolEvent(event) => if let DexEvent::MeteoraDammV2InitializePoolEvent(cpi_event) = cpi_log_event {
                event.pool = cpi_event.pool;
                event.token_a_mint = cpi_event.token_a_mint;
                event.token_b_mint = cpi_event.token_b_mint;
                event.creator = cpi_event.creator;
                event.payer = cpi_event.payer;
                event.alpha_vault = cpi_event.alpha_vault;
                event.pool_fees = cpi_event.pool_fees;
                event.sqrt_min_price = cpi_event.sqrt_min_price;
                event.sqrt_max_price = cpi_event.sqrt_max_price;
                event.activation_type = cpi_event.activation_type;
                event.collect_fee_mode = cpi_event.collect_fee_mode;
                event.liquidity = cpi_event.liquidity;
                event.sqrt_price = cpi_event.sqrt_price;
                event.activation_point = cpi_event.activation_point;
                event.token_a_flag = cpi_event.token_a_flag;
                event.token_b_flag = cpi_event.token_b_flag;
                event.token_a_amount = cpi_event.token_a_amount;
                event.token_b_amount = cpi_event.token_b_amount;
                event.total_amount_a = cpi_event.total_amount_a;
                event.total_amount_b = cpi_event.total_amount_b;
                event.pool_type = cpi_event.pool_type;
            },
        DexEvent::MeteoraDammV2InitializeCustomizablePoolEvent(event) => if let DexEvent::MeteoraDammV2InitializePoolEvent(cpi_event) = cpi_log_event {
                event.pool = cpi_event.pool;
                event.token_a_mint = cpi_event.token_a_mint;
                event.token_b_mint = cpi_event.token_b_mint;
                event.creator = cpi_event.creator;
                event.payer = cpi_event.payer;
                event.alpha_vault = cpi_event.alpha_vault;
                event.pool_fees = cpi_event.pool_fees;
                event.sqrt_min_price = cpi_event.sqrt_min_price;
                event.sqrt_max_price = cpi_event.sqrt_max_price;
                event.activation_type = cpi_event.activation_type;
                event.collect_fee_mode = cpi_event.collect_fee_mode;
                event.liquidity = cpi_event.liquidity;
                event.sqrt_price = cpi_event.sqrt_price;
                event.activation_point = cpi_event.activation_point;
                event.token_a_flag = cpi_event.token_a_flag;
                event.token_b_flag = cpi_event.token_b_flag;
                event.token_a_amount = cpi_event.token_a_amount;
                event.token_b_amount = cpi_event.token_b_amount;
                event.total_amount_a = cpi_event.total_amount_a;
                event.total_amount_b = cpi_event.total_amount_b;
                event.pool_type = cpi_event.pool_type;
            },
        DexEvent::MeteoraDammV2InitializePoolWithDynamicConfigEvent(event) => if let DexEvent::MeteoraDammV2InitializePoolEvent(cpi_event) = cpi_log_event {
                event.pool = cpi_event.pool;
                event.token_a_mint = cpi_event.token_a_mint;
                event.token_b_mint = cpi_event.token_b_mint;
                event.creator = cpi_event.creator;
                event.payer = cpi_event.payer;
                event.alpha_vault = cpi_event.alpha_vault;
                event.pool_fees = cpi_event.pool_fees;
                event.sqrt_min_price = cpi_event.sqrt_min_price;
                event.sqrt_max_price = cpi_event.sqrt_max_price;
                event.activation_type = cpi_event.activation_type;
                event.collect_fee_mode = cpi_event.collect_fee_mode;
                event.liquidity = cpi_event.liquidity;
                event.sqrt_price = cpi_event.sqrt_price;
                event.activation_point = cpi_event.activation_point;
                event.token_a_flag = cpi_event.token_a_flag;
                event.token_b_flag = cpi_event.token_b_flag;
                event.token_a_amount = cpi_event.token_a_amount;
                event.token_b_amount = cpi_event.token_b_amount;
                event.total_amount_a = cpi_event.total_amount_a;
                event.total_amount_b = cpi_event.total_amount_b;
                event.pool_type = cpi_event.pool_type;
            },

        _ => {}
    }
}
