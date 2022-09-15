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

pub trait Instrumental {
    type AccountId: core::cmp::Ord;
    type AssetId;
    type Balance;
    type VaultId: Clone + Codec + Debug + PartialEq + Default + Parameter;

    fn account_id() -> Self::AccountId;

    fn create(
        config: InstrumentalVaultConfig<Self::AssetId, Perquintill>,
    ) -> Result<Self::VaultId, DispatchError>;

    fn add_liquidity(
        issuer: &Self::AccountId,
        asset: &Self::AssetId,
        amount: Self::Balance,
    ) -> DispatchResult;

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
