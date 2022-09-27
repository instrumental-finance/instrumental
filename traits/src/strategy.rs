//! # Instrumental Protocol Strategy
//!
//! Common traits for working with strategies. For each protocol that wants to be reached through
//! Instrumental, a unique strategy pallet needs to be developed and traits below should be
//! implemented.
use codec::Codec;
use frame_support::{sp_std::fmt::Debug, Parameter};
use sp_runtime::{DispatchError, DispatchResult};

/// Provide functionality for working with the strategy.
pub trait InstrumentalProtocolStrategy {
    /// The ID that uniquely identify the strategy.
    type AccountId: core::cmp::Ord;
    /// The ID that uniquely identify an asset.
    type AssetId;
    /// The ID that uniquely identify a pool.
    type PoolId;
    /// The ID that uniquely identify a vault associated with the strategy.
    type VaultId: Clone + Codec + Debug + PartialEq + Default + Parameter;

    /// Get unique ID of the strategy.
    fn account_id() -> Self::AccountId;

    /// Associate a vault with this strategy.
    fn associate_vault(vault_id: &Self::VaultId) -> DispatchResult;

    /// Queries the total assets under management by the strategies associated vault (reserved
    /// balance plus the amount in the strategy) and performs any rebalancing if required.
    fn rebalance() -> DispatchResult;

    /// Returns the optimum (estimated) APY value for a provided asset id.
    fn get_apy(asset: Self::AssetId) -> Result<u128, DispatchError>;

    /// Store a mapping of assets's ID and a pool's ID.
    fn set_pool_id_for_asset(asset_id: Self::AssetId, pool_id: Self::PoolId) -> DispatchResult;

    fn halt() -> DispatchResult;

    fn start() -> DispatchResult;

    fn is_halted() -> Result<bool, DispatchError>;
}
