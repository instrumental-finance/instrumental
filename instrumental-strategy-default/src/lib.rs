//! Default strategy.
//!
//! Instrumental strategy that only sends funds into a vault. It's APY is thus 0. Exist for cases
//! when we are shutting down a strategy, updating a strategy, adding a new asset to Instrumental,
//! etc.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::todo,
    clippy::unseparated_literal_suffix,
    clippy::unwrap_used
)]
#![cfg_attr(
    test,
    allow(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used,
    )
)]

mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    // ---------------------------------------------------------------------------------------------
    //                                      Imports and Dependencies
    // ---------------------------------------------------------------------------------------------

    use codec::{Codec, FullCodec, MaxEncodedLen};
    use composable_traits::vault::{CapabilityVault, StrategicVault};
    use frame_support::{
        ensure,
        pallet_prelude::{DispatchResultWithPostInfo, MaybeSerializeDeserialize},
        storage::types::StorageValue,
        traits::{EnsureOrigin, GenesisBuild, Get, IsType},
        transactional, BoundedBTreeSet, PalletId, Parameter,
    };
    use frame_system::pallet_prelude::OriginFor;
    use scale_info::TypeInfo;
    use sp_runtime::{
        traits::{
            AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub, Zero,
        },
        DispatchError, DispatchResult,
    };
    use sp_std::fmt::Debug;
    use traits::strategy::InstrumentalProtocolStrategy;

    use crate::weights::WeightInfo;

    // ---------------------------------------------------------------------------------------------
    //                                   Declaration Of The Pallet Type
    // ---------------------------------------------------------------------------------------------

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // ---------------------------------------------------------------------------------------------
    //                                            Config Trait
    // ---------------------------------------------------------------------------------------------

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event type emitted by this pallet. Depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Some sort of check on the origin is performed by this object.
        type ExternalOrigin: EnsureOrigin<Self::Origin>;

        /// Weight information for this pallet's extrinsics.
        type WeightInfo: WeightInfo;

        /// The type used by the pallet for bookkeeping.
        type Balance: Default
            + Parameter
            + Codec
            + MaxEncodedLen
            + Copy
            + Ord
            + CheckedAdd
            + CheckedSub
            + CheckedMul
            + AtLeast32BitUnsigned
            + Zero;

        /// The ID that uniquely identify an asset.
        type AssetId: Default
            + FullCodec
            + MaxEncodedLen
            + Eq
            + PartialEq
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + TypeInfo;

        /// Corresponds to the Ids used by the Vault pallet.
        type VaultId: FullCodec
            + MaxEncodedLen
            + Eq
            + PartialEq
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + Default
            + Ord
            + TypeInfo
            + Into<u128>;

        /// Vault used in strategy to obtain funds from, report balances and return funds to.
        type Vault: StrategicVault<
                AssetId = Self::AssetId,
                Balance = Self::Balance,
                AccountId = Self::AccountId,
                VaultId = Self::VaultId,
            > + CapabilityVault<
                AccountId = Self::AccountId,
                AssetId = Self::AssetId,
                Balance = Self::Balance,
                VaultId = Self::VaultId,
            >;

        /// Type representing the unique ID of a pool.
        type PoolId: FullCodec
            + MaxEncodedLen
            + Default
            + Debug
            + TypeInfo
            + Eq
            + PartialEq
            + Ord
            + Copy;

        /// The maximum number of vaults that can be associated with this strategy.
        #[pallet::constant]
        type MaxAssociatedVaults: Get<u32>;

        /// The id used as the
        /// [`AccountId`](traits::instrumental::Instrumental::AccountId) of the vault.
        /// This should be unique across all pallets to avoid name collisions with other pallets and
        /// vaults.
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    // ---------------------------------------------------------------------------------------------
    //                                          Runtime Storage
    // ---------------------------------------------------------------------------------------------

    /// The storage where we store all the vault's IDs that are associated with this strategy.
    #[pallet::storage]
    pub type AssociatedVaults<T: Config> =
        StorageValue<_, BoundedBTreeSet<T::VaultId, T::MaxAssociatedVaults>>;

    /// Stores information about whether the strategy is halted or not.
    #[pallet::storage]
    pub type Halted<T: Config> = StorageValue<_, bool>;

    // ---------------------------------------------------------------------------------------------
    //                                           Genesis config
    // ---------------------------------------------------------------------------------------------

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {
        is_halted: bool,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            Halted::<T>::put(self.is_halted);
        }
    }

    // ---------------------------------------------------------------------------------------------
    //                                           Runtime Events
    // ---------------------------------------------------------------------------------------------

    /// Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Vault successfully associated with this strategy.
        AssociatedVault {
            /// Vault ID of associated vault.
            vault_id: T::VaultId,
        },

        /// The event is deposited when the strategy is halted.
        Halted,

        /// The event is deposited when the strategy is started again after halting.
        Unhalted,
    }

    // ---------------------------------------------------------------------------------------------
    //                                           Runtime Errors
    // ---------------------------------------------------------------------------------------------

    /// Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// The Vault already associated with this strategy. See [`AssociatedVaults`] for details.
        VaultAlreadyAssociated,

        /// Exceeds the maximum number of vaults that can be associated with this strategy. See
        /// [`Config::MaxAssociatedVaults`] for details.
        TooManyAssociatedStrategies,

        /// Storage is not initialized (have `None` value).
        StorageIsNotInitialized,

        /// Occurs when the strategy is halted, and someone is trying to perform any operations
        /// (only rebalancing actually) with it
        Halted,
    }

    // ---------------------------------------------------------------------------------------------
    //                                             Extrinsics
    // ---------------------------------------------------------------------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add [`Config::VaultId`] to [`AssociatedVaults`] storage.
        ///
        /// Emits [`Event::AssociatedVault`] event when successful.
        #[pallet::weight(T::WeightInfo::associate_vault())]
        pub fn associate_vault(
            origin: OriginFor<T>,
            vault_id: T::VaultId,
        ) -> DispatchResultWithPostInfo {
            T::ExternalOrigin::ensure_origin(origin)?;
            <Self as InstrumentalProtocolStrategy>::associate_vault(&vault_id)?;
            Ok(().into())
        }

        /// Halt the strategy.
        ///
        /// Emits [`Halted`](Event::Halted) event when successful.
        #[pallet::weight(T::WeightInfo::halt())]
        pub fn halt(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::ExternalOrigin::ensure_origin(origin)?;
            <Self as InstrumentalProtocolStrategy>::halt()?;
            Ok(().into())
        }

        /// Continue the strategy after halting.
        ///
        /// Emits [`Unhalted`](Event::Unhalted) event when successful.
        #[pallet::weight(T::WeightInfo::start())]
        pub fn start(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::ExternalOrigin::ensure_origin(origin)?;
            <Self as InstrumentalProtocolStrategy>::start()?;
            Ok(().into())
        }
    }

    // ---------------------------------------------------------------------------------------------
    //                                         Protocol Strategy
    // ---------------------------------------------------------------------------------------------

    impl<T: Config> InstrumentalProtocolStrategy for Pallet<T> {
        type AccountId = T::AccountId;
        type AssetId = T::AssetId;
        type PoolId = T::PoolId;
        type VaultId = T::VaultId;

        fn account_id() -> Self::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        #[transactional]
        fn associate_vault(vault_id: &Self::VaultId) -> DispatchResult {
            AssociatedVaults::<T>::try_mutate(|vaults| {
                let vaults = vaults.as_mut().ok_or(Error::<T>::StorageIsNotInitialized)?;
                ensure!(
                    !vaults.contains(vault_id),
                    Error::<T>::VaultAlreadyAssociated
                );

                vaults
                    .try_insert(*vault_id)
                    .map_err(|_| Error::<T>::TooManyAssociatedStrategies)?;

                Self::deposit_event(Event::AssociatedVault {
                    vault_id: *vault_id,
                });

                Ok(())
            })
        }

        fn rebalance() -> DispatchResult {
            Ok(())
        }

        fn get_apy(_asset: Self::AssetId) -> Result<u128, DispatchError> {
            Ok(0_u128)
        }

        fn set_pool_id_for_asset(
            _asset_id: Self::AssetId,
            _pool_id: Self::PoolId,
        ) -> DispatchResult {
            Ok(())
        }

        #[transactional]
        fn halt() -> DispatchResult {
            for vault_id in AssociatedVaults::<T>::get()
                .ok_or(Error::<T>::StorageIsNotInitialized)?
                .iter()
            {
                <T::Vault as CapabilityVault>::stop(vault_id)?;
            }
            Halted::<T>::put(true);
            Self::deposit_event(Event::Halted);
            Ok(())
        }

        #[transactional]
        fn start() -> DispatchResult {
            for vault_id in AssociatedVaults::<T>::get()
                .ok_or(Error::<T>::StorageIsNotInitialized)?
                .iter()
            {
                <T::Vault as CapabilityVault>::start(vault_id)?;
            }
            Halted::<T>::put(false);
            Self::deposit_event(Event::Unhalted);
            Ok(())
        }

        fn is_halted() -> Result<bool, DispatchError> {
            Halted::<T>::get().ok_or_else(|| Error::<T>::StorageIsNotInitialized.into())
        }
    }
}
