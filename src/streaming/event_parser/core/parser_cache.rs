//! # Event parser configuration cache module
//!
//! Manages event parser caches including:
//! - Program ID cache
//! - High-performance cache tools
//!
//! ## Design goals
//! - **High-performance caching**: Avoid repeated initialization and memory allocation
//! - **Extensibility**: Dynamic dispatch through the dispatcher

use crate::streaming::event_parser::{
    common::{filter::EventTypeFilter, EventType},
    core::dispatcher::EventDispatcher,
    Protocol,
};
use solana_sdk::pubkey::Pubkey;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

// ============================================================================
// Part 1: Program ID Cache
// ============================================================================

/// Cache key: identifies a unique combination of protocols and filters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Sorted protocol list
    pub protocols: Vec<Protocol>,
    /// Optional sorted event type filter
    pub event_types: Option<Vec<EventType>>,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(mut protocols: Vec<Protocol>, filter: Option<&EventTypeFilter>) -> Self {
        // Sort protocols to ensure identical combinations produce the same key
        protocols.sort();

        let event_types = filter.map(|f| {
            let mut types = f.include.clone();
            types.sort();
            types
        });

        Self { protocols, event_types }
    }
}

/// Global program ID cache (protected by RwLock)
static GLOBAL_PROGRAM_IDS_CACHE: LazyLock<
    std::sync::RwLock<HashMap<CacheKey, Arc<Vec<Pubkey>>>>,
> = LazyLock::new(|| std::sync::RwLock::new(HashMap::new()));

/// Get program IDs for the specified protocols
///
/// Uses EventDispatcher to get program_ids and caches the result
pub fn get_global_program_ids(
    protocols: &[Protocol],
    filter: Option<&EventTypeFilter>,
) -> Arc<Vec<Pubkey>> {
    let cache_key = CacheKey::new(protocols.to_vec(), filter);

    // Fast path: try reading from cache
    {
        let cache = GLOBAL_PROGRAM_IDS_CACHE.read().unwrap_or_else(|error| error.into_inner());
        if let Some(program_ids) = cache.get(&cache_key) {
            return program_ids.clone();
        }
    }

    // Slow path: get via EventDispatcher and cache
    let program_ids = Arc::new(EventDispatcher::get_program_ids(protocols));

    // Cache the result (write lock)
    GLOBAL_PROGRAM_IDS_CACHE.write().unwrap_or_else(|error| error.into_inner()).insert(cache_key, program_ids.clone());

    program_ids
}

// ============================================================================
// Part 2: Account Pubkey Cache Tools
// ============================================================================

/// High-performance account pubkey cache
///
/// Reuses memory to avoid repeated Vec allocations, improving performance
#[derive(Debug)]
pub struct AccountPubkeyCache {
    /// Pre-allocated account pubkey vector
    cache: Vec<Pubkey>,
}

impl AccountPubkeyCache {
    /// Create a new account pubkey cache
    ///
    /// Pre-allocates 32 slots, covering most transaction scenarios
    pub fn new() -> Self {
        Self {
            cache: Vec::with_capacity(32),
        }
    }

    /// Build account pubkey vector from instruction account indices
    ///
    /// # Parameters
    /// - `instruction_accounts`: Instruction account index list
    /// - `all_accounts`: All account pubkeys
    ///
    /// # Returns
    /// Account pubkey slice reference
    ///
    /// # Performance optimization
    /// - Reuses internal cache to avoid reallocation
    /// - Only expands capacity when necessary
    #[inline]
    pub fn build_account_pubkeys(
        &mut self,
        instruction_accounts: &[u8],
        all_accounts: &[Pubkey],
    ) -> &[Pubkey] {
        self.cache.clear();

        // Ensure sufficient capacity to avoid dynamic resizing
        if self.cache.capacity() < instruction_accounts.len() {
            self.cache.reserve(instruction_accounts.len() - self.cache.capacity());
        }

        // Fast fill account pubkeys (with bounds checking)
        for &index in instruction_accounts.iter() {
            if (index as usize) < all_accounts.len() {
                self.cache.push(all_accounts[index as usize]);
            } else {
                self.cache.push(Pubkey::default());
            }
        }

        &self.cache
    }
}

impl Default for AccountPubkeyCache {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static THREAD_LOCAL_ACCOUNT_CACHE: std::cell::RefCell<AccountPubkeyCache> =
        std::cell::RefCell::new(AccountPubkeyCache::new());
}

/// Build account pubkey list from thread-local cache
///
/// # Parameters
/// - `instruction_accounts`: Instruction account index list
/// - `all_accounts`: All account pubkeys
///
/// # Returns
/// Account pubkey vector
///
/// # Thread safety
/// Uses thread-local storage, each thread has its own independent cache
#[inline]
pub fn build_account_pubkeys_with_cache(
    instruction_accounts: &[u8],
    all_accounts: &[Pubkey],
) -> Vec<Pubkey> {
    THREAD_LOCAL_ACCOUNT_CACHE.with(|tl_cache| {
        let mut cache = tl_cache.borrow_mut();
        cache.build_account_pubkeys(instruction_accounts, all_accounts).to_vec()
    })
}

/// Execute a closure with a borrowed account pubkey slice from the thread-local cache
///
/// This avoids allocating a Vec by providing a zero-allocation reference to the cached slice.
///
/// # Parameters
/// - `instruction_accounts`: Instruction account index list
/// - `all_accounts`: All account pubkeys
/// - `f`: Closure that takes the slice (`&[Pubkey]`)
///
/// # Returns
/// The result of the closure
#[inline]
pub fn with_account_pubkeys_cache<Result, Operation>(
    instruction_accounts: &[u8],
    all_accounts: &[Pubkey],
    operation: Operation,
) -> Result
where
    Operation: FnOnce(&[Pubkey]) -> Result,
{
    THREAD_LOCAL_ACCOUNT_CACHE.with(|tl_cache| {
        let mut cache = tl_cache.borrow_mut();
        let pubkeys = cache.build_account_pubkeys(instruction_accounts, all_accounts);
        operation(pubkeys)
    })
}
