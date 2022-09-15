use codec::Codec;
use frame_support::{sp_std::fmt::Debug, Parameter};
use sp_runtime::{DispatchError, DispatchResult};

pub trait InstrumentalProtocolStrategy {
    type AccountId: core::cmp::Ord;
    type VaultId: Clone + Codec + Debug + PartialEq + Default + Parameter;
    type AssetId;
    type PoolId;

    fn account_id() -> Self::AccountId;

    fn associate_vault(vault_id: &Self::VaultId) -> DispatchResult;

    fn rebalance() -> DispatchResult;

    fn get_apy(asset: Self::AssetId) -> Result<u128, DispatchError>;

    fn set_pool_id_for_asset(asset_id: Self::AssetId, pool_id: Self::PoolId) -> DispatchResult;
}
