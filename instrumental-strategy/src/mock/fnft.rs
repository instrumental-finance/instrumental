use composable_traits::fnft::FinancialNft;
use frame_support::{
    dispatch::DispatchResult,
    traits::tokens::nonfungibles::{Create, Inspect, Mutate},
};
use sp_runtime::DispatchError;

pub struct MockFnft;

impl Inspect<u128> for MockFnft {
    type CollectionId = ();
    type ItemId = ();

    fn owner(_collection: &Self::CollectionId, _item: &Self::ItemId) -> Option<u128> {
        unimplemented!()
    }
}

impl FinancialNft<u128> for MockFnft {
    fn asset_account(_collection: &Self::CollectionId, _instance: &Self::ItemId) -> u128 {
        unimplemented!()
    }

    fn get_next_nft_id(_collection: &Self::CollectionId) -> Result<Self::ItemId, DispatchError> {
        unimplemented!()
    }
}

impl Create<u128> for MockFnft {
    fn create_collection(
        _collection: &Self::CollectionId,
        _who: &u128,
        _admin: &u128,
    ) -> DispatchResult {
        unimplemented!()
    }
}

impl Mutate<u128> for MockFnft {}
