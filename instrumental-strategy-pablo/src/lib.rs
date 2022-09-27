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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    // ---------------------------------------------------------------------------------------------
    //                                     Imports and Dependencies
    // ---------------------------------------------------------------------------------------------

    use codec::{Codec, FullCodec};
    use composable_traits::{
        dex::Amm,
        vault::{CapabilityVault, StrategicVault},
    };
    use frame_support::{
        dispatch::{DispatchError, DispatchResult},
        pallet_prelude::*,
        storage::bounded_btree_set::BoundedBTreeSet,
        traits::{
            fungibles::{Mutate, MutateHold, Transfer},
            GenesisBuild,
        },
        transactional, Blake2_128Concat, PalletId, RuntimeDebug,
    };
    use frame_system::pallet_prelude::OriginFor;
    use sp_runtime::traits::{
        AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub, Zero,
    };
    use sp_std::fmt::Debug;
    use traits::{instrumental::State, strategy::InstrumentalProtocolStrategy};

    use crate::weights::WeightInfo;

    // ---------------------------------------------------------------------------------------------
    //                                  Declaration Of The Pallet Type
    // ---------------------------------------------------------------------------------------------

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // ---------------------------------------------------------------------------------------------
    //                                           Config Trait
    // ---------------------------------------------------------------------------------------------

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(missing_docs)]
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Some sort of check on the origin is performed by this object.
        type ExternalOrigin: EnsureOrigin<Self::Origin>;

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
        type AssetId: FullCodec
            + MaxEncodedLen
            + Eq
            + PartialEq
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + Default
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

        /// Currency is used for the assets managed by the vaults.
        type Currency: Transfer<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>
            + Mutate<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>
            + MutateHold<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>;

        /// Used for interacting with Pablo pallet.
        type Pablo: Amm<
            AssetId = Self::AssetId,
            Balance = Self::Balance,
            AccountId = Self::AccountId,
            PoolId = Self::PoolId,
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
    //                                           Pallet Types
    // ---------------------------------------------------------------------------------------------

    #[derive(
        Encode, Decode, MaxEncodedLen, Clone, Copy, Default, RuntimeDebug, PartialEq, Eq, TypeInfo,
    )]
    pub struct PoolState<PoolId, State> {
        pub pool_id: PoolId,
        pub state: State,
    }

    // ---------------------------------------------------------------------------------------------
    //                                          Runtime Storage
    // ---------------------------------------------------------------------------------------------

    /// The storage where we store all the vault's IDs that are associated with this strategy.
    #[pallet::storage]
    #[pallet::getter(fn associated_vaults)]
    #[allow(clippy::disallowed_types)]
    pub type AssociatedVaults<T: Config> =
        StorageValue<_, BoundedBTreeSet<T::VaultId, T::MaxAssociatedVaults>, ValueQuery>;

    /// An asset whitelisted by Instrumental.
    ///
    /// The corresponding Pool to invest the whitelisted asset into.
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AssetId, PoolState<T::PoolId, State>>;

    /// Stores information about whether the strategy is halted or not.
    #[pallet::storage]
    pub type Halted<T: Config> = StorageValue<_, bool>;

    // ---------------------------------------------------------------------------------------------
    //                                           Genesis config
    // ---------------------------------------------------------------------------------------------

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {
        pub is_halted: bool,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            Halted::<T>::put(self.is_halted);
        }
    }

    // ---------------------------------------------------------------------------------------------
    //                                          Runtime Events
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

        RebalancedVault {
            /// Vault ID of rebalanced vault.
            vault_id: T::VaultId,
        },

        UnableToRebalanceVault {
            /// Vault ID of vault that can't be rebalanced.
            vault_id: T::VaultId,
        },

        AssociatedPoolWithAsset {
            asset_id: T::AssetId,
            pool_id: T::PoolId,
        },

        /// The event is deposited when the strategy is halted.
        Halted,

        /// The event is deposited when the strategy is started again after halting.
        Unhalted,
    }

    // ---------------------------------------------------------------------------------------------
    //                                          Runtime Errors
    // ---------------------------------------------------------------------------------------------

    /// Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// The Vault already associated with this strategy. See [`AssociatedVaults`] for details.
        VaultAlreadyAssociated,

        /// Exceeds the maximum number of vaults that can be associated with this strategy. See
        /// [`Config::MaxAssociatedVaults`] for details.
        TooManyAssociatedStrategies,

        /// TODO(belousm): only for MVP version we can assume the `pool_id` is already known and
        /// exist. We should remove it in V1.
        PoolNotFound,

        /// Occurs when we try to set a new pool_id, during a transferring from or to an old one
        TransferringInProgress,

        /// Storage is not initialized (have `None` value).
        StorageIsNotInitialized,

        /// Occurs when the strategy is halted, and someone is trying to perform any operations
        /// (only rebalancing actually) with it
        Halted,
    }

    // ---------------------------------------------------------------------------------------------
    //                                               Hooks
    // ---------------------------------------------------------------------------------------------

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    // ---------------------------------------------------------------------------------------------
    //                                            Extrinsics
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

        /// Store a mapping of asset_id -> pool_id in the pools runtime storage object.
        ///
        /// Emits [`AssociatedPoolWithAsset`](Event::AssociatedPoolWithAsset) event when successful.
        #[pallet::weight(T::WeightInfo::set_pool_id_for_asset())]
        pub fn set_pool_id_for_asset(
            origin: OriginFor<T>,
            asset_id: T::AssetId,
            pool_id: T::PoolId,
        ) -> DispatchResultWithPostInfo {
            T::ExternalOrigin::ensure_origin(origin)?;
            <Self as InstrumentalProtocolStrategy>::set_pool_id_for_asset(asset_id, pool_id)?;
            Ok(().into())
        }

        /// Occur rebalance of liquidity of each vault.
        ///
        /// Emits [`RebalancedVault`](Event::RebalancedVault) event when successful.
        #[pallet::weight(T::WeightInfo::liquidity_rebalance())]
        pub fn liquidity_rebalance(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::ExternalOrigin::ensure_origin(origin)?;
            <Self as InstrumentalProtocolStrategy>::rebalance()?;
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
        fn set_pool_id_for_asset(asset_id: T::AssetId, pool_id: T::PoolId) -> DispatchResult {
            match Pools::<T>::try_get(asset_id) {
                Ok(pool) => {
                    ensure!(
                        pool.state == State::Normal,
                        Error::<T>::TransferringInProgress
                    );
                    Pools::<T>::mutate(asset_id, |_| PoolState {
                        pool_id,
                        state: State::Normal,
                    });
                }
                Err(_) => Pools::<T>::insert(
                    asset_id,
                    PoolState {
                        pool_id,
                        state: State::Normal,
                    },
                ),
            }
            Self::deposit_event(Event::AssociatedPoolWithAsset { asset_id, pool_id });
            Ok(())
        }

        #[transactional]
        fn associate_vault(vault_id: &Self::VaultId) -> DispatchResult {
            AssociatedVaults::<T>::try_mutate(|vaults| -> DispatchResult {
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

        #[transactional]
        fn rebalance() -> DispatchResult {
            if Self::is_halted()? {
                return Err(Error::<T>::Halted.into());
            }
            AssociatedVaults::<T>::try_mutate(|vaults| -> DispatchResult {
                vaults.iter().for_each(|vault_id| {
                    if Self::do_rebalance(vault_id).is_ok() {
                        Self::deposit_event(Event::RebalancedVault {
                            vault_id: *vault_id,
                        });
                    } else {
                        Self::deposit_event(Event::UnableToRebalanceVault {
                            vault_id: *vault_id,
                        });
                    }
                });

                Ok(())
            })
        }

        fn get_apy(_asset: Self::AssetId) -> Result<u128, DispatchError> {
            Ok(0)
        }

        fn halt() -> DispatchResult {
            for vault_id in AssociatedVaults::<T>::get().iter() {
                <T::Vault as CapabilityVault>::stop(vault_id)?;
            }
            Halted::<T>::put(true);
            Self::deposit_event(Event::Halted);
            Ok(())
        }

        fn start() -> DispatchResult {
            for vault_id in AssociatedVaults::<T>::get().iter() {
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

    // ---------------------------------------------------------------------------------------------
    //                                      Low Level Functionality
    // ---------------------------------------------------------------------------------------------

    impl<T: Config> Pallet<T> {
        fn do_rebalance(_vault_id: &T::VaultId) -> DispatchResult {
            // TODO(saruman9): reimplement
            Ok(())
        }
    }
}

// -------------------------------------------------------------------------------------------------
//                                            Unit Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod unit_tests {}
