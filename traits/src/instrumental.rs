//! # Instrumental
//!
//! From the users perspective, they will solely be interacting with Instrumental. Behind the
//! scenes, their assets will be sent to Picasso and further dispersed into the numerous other
//! pallets in the parachain to earn yield.
use codec::{Codec, Decode, Encode, MaxEncodedLen};
use frame_support::{sp_std::fmt::Debug, Parameter, RuntimeDebug};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, DispatchResult, Perquintill};

/// An indication of pool state. Shows whether the transfer of assets is currently taking place with
/// the current pool.
#[derive(Copy, Clone, Encode, Decode, RuntimeDebug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum State {
    /// Indicates that there is currently no asset transferring going on for this asset
    /// and it can be initialized.
    Normal,
    /// Indicates that an asset is currently being transferred from one pool to another
    /// for this asset, so it is not possible to initialize a new transfer.
    Transferring,
}

#[derive(Clone, Copy, Encode, Decode, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct InstrumentalVaultConfig<AssetId, Percent> {
    pub asset_id: AssetId,
    pub percent_deployable: Percent,
}

/// Provide functionality for working with Instrumental pallet.
pub trait Instrumental {
    /// The ID that uniquely identify Instrumental pallet.
    type AccountId: core::cmp::Ord;
    /// The ID that uniquely identify an asset.
    type AssetId;
    /// The type used for bookkeeping.
    type Balance;
    /// The ID that uniquely identify a vault associated with the strategy.
    type VaultId: Clone + Codec + Debug + PartialEq + Default + Parameter;

    /// Get unique ID of the pallet.
    fn account_id() -> Self::AccountId;

    /// Create a new Instrumental vault for the specified asset; throws an error if the asset
    /// already has an associated vault.
    fn create(
        config: InstrumentalVaultConfig<Self::AssetId, Perquintill>,
    ) -> Result<Self::VaultId, DispatchError>;

    /// Specify an asset ID and amount to deposit. Behind the scenes the function will connect with
    /// the Vault pallet to deposit into the associated vault.
    fn add_liquidity(
        issuer: &Self::AccountId,
        asset: &Self::AssetId,
        amount: Self::Balance,
    ) -> DispatchResult;

    /// Specify the asset ID and amount to withdraw. Behind the scenes the function will speak to
    /// the Vault pallet to withdraw assets from the associated vault.
    fn remove_liquidity(
        issuer: &Self::AccountId,
        asset: &Self::AssetId,
        amount: Self::Balance,
    ) -> DispatchResult;
}

pub trait InstrumentalDynamicStrategy {
    type AccountId: core::cmp::Ord;
    type AssetId;

    fn get_optimum_strategy_for(asset: Self::AssetId) -> Result<Self::AccountId, DispatchError>;
}
