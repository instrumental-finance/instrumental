use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

const I_HAVENT_CALCULATED_YET: i32 = 10_000;

pub trait WeightInfo {
    fn create() -> Weight;
    fn add_liquidity() -> Weight;
    fn remove_liquidity() -> Weight;
}

/// Weights for pallet_instrumental using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create() -> Weight {
        I_HAVENT_CALCULATED_YET as Weight
    }

    fn add_liquidity() -> Weight {
        I_HAVENT_CALCULATED_YET as Weight
    }

    fn remove_liquidity() -> Weight {
        I_HAVENT_CALCULATED_YET as Weight
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create() -> Weight {
        I_HAVENT_CALCULATED_YET as Weight
    }

    fn add_liquidity() -> Weight {
        I_HAVENT_CALCULATED_YET as Weight
    }

    fn remove_liquidity() -> Weight {
        I_HAVENT_CALCULATED_YET as Weight
    }
}
