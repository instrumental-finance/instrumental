//! Weights for extrinsic functions.

use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

/// Trait for using weights for functions.
pub trait WeightInfo {
    fn associate_vault() -> Weight;
    fn halt() -> Weight;
    fn start() -> Weight;
}

/// Weights for the pallet using the Substrate node and recommended hardware.
pub struct SubstrateWight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWight<T> {
    fn associate_vault() -> Weight {
        10_000 as Weight
    }

    fn halt() -> Weight {
        10_000 as Weight
    }

    fn start() -> Weight {
        10_000 as Weight
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn associate_vault() -> Weight {
        10_000 as Weight
    }

    fn halt() -> Weight {
        10_000 as Weight
    }

    fn start() -> Weight {
        10_000 as Weight
    }
}
