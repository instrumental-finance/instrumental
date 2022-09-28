use codec::Codec;
use frame_support::{sp_std::fmt::Debug, Parameter};
use sp_runtime::{DispatchError, DispatchResult, Percent};

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

    fn halt() -> DispatchResult;

    fn start() -> DispatchResult;

    fn is_halted() -> bool;

    fn transferring_funds(
        vault_id: &Self::VaultId,
        asset_id: Self::AssetId,
        new_pool_id: Self::PoolId,
        percentage_of_funds: Percent,
    ) -> DispatchResult;
}
