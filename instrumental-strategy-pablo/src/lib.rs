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
    use composable_support::math::safe::{safe_multiply_by_rational, SafeDiv, SafeSub};
    use composable_traits::{
        dex::Amm,
        vault::{CapabilityVault, FundsAvailability, StrategicVault, Vault},
    };
    use frame_support::{
        dispatch::{DispatchError, DispatchResult},
        pallet_prelude::*,
        storage::bounded_btree_set::BoundedBTreeSet,
        traits::fungibles::{Inspect, Mutate, MutateHold, Transfer},
        transactional, Blake2_128Concat, PalletId, RuntimeDebug,
    };
    use frame_system::pallet_prelude::OriginFor;
    use sp_runtime::{
        traits::{
            AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub, Convert,
            Zero,
        },
        Percent,
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

        /// Conversion function from [`Self::Balance`] to u128 and from u128 to [`Self::Balance`].
        type Convert: Convert<Self::Balance, u128> + Convert<u128, Self::Balance>;
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

        /// Vault successfully rebalanced.
        RebalancedVault {
            /// Vault ID of rebalanced vault.
            vault_id: T::VaultId,
        },

        /// During Vault rebalancing withdraw action occurred.
        WithdrawFunctionalityOccuredDuringRebalance {
            /// Vault ID of rebalanced vault.
            vault_id: T::VaultId,
        },

        /// During Vault rebalancing deposit action occurred.
        DepositFunctionalityOccuredDuringRebalance {
            /// Vault ID of rebalanced vault.
            vault_id: T::VaultId,
        },

        /// During Vault rebalancing liquidate action occurred.
        LiquidateFunctionalityOccuredDuringRebalance {
            /// Vault ID of rebalanced vault.
            vault_id: T::VaultId,
        },

        /// During Vault rebalancing none action occurred.
        NoneFunctionalityOccuredDuringRebalance {
            /// Vault ID of rebalanced vault.
            vault_id: T::VaultId,
        },

        /// Occurred when it's unable to rebalance Vault.
        UnableToRebalanceVault {
            /// Vault ID of vault that can't be rebalanced.
            vault_id: T::VaultId,
        },

        /// The event is deposited when pool associated with asset.
        AssociatedPoolWithAsset {
            /// Asset ID which will be associated with pool.
            asset_id: T::AssetId,
            /// Pool ID which will be associated with asset.
            pool_id: T::PoolId,
        },

        /// The event is deposited when funds transferred from one pool to another.
        FundsTransfferedToNewPool {
            /// Pool ID of new pool in which money transferred.
            new_pool_id: T::PoolId,
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

        /// Occurs when we try to set a new pool_id, during a transferring from or to an old one.
        TransferringInProgress,

        /// Storage is not initialized (have `None` value).
        StorageIsNotInitialized,

        /// Occurs when the strategy is halted, and someone is trying to perform any operations
        /// (only rebalancing actually) with it.
        Halted,

        /// No strategy is associated with the Vault.
        NoStrategies,
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
            vault_id: T::VaultId,
            percentage_of_funds: Option<Percent>,
        ) -> DispatchResultWithPostInfo {
            T::ExternalOrigin::ensure_origin(origin)?;
            Self::do_set_pool_id_for_asset(asset_id, pool_id, &vault_id, percentage_of_funds)?;
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

        #[transactional]
        fn halt() -> DispatchResult {
            for vault_id in AssociatedVaults::<T>::get().iter() {
                <T::Vault as CapabilityVault>::stop(vault_id)?;
            }
            Halted::<T>::put(true);
            Self::deposit_event(Event::Halted);
            Ok(())
        }

        #[transactional]
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
        #[transactional]
        fn do_set_pool_id_for_asset(
            asset_id: T::AssetId,
            pool_id: T::PoolId,
            vault_id: &T::VaultId,
            percentage_of_funds: Option<Percent>,
        ) -> DispatchResult {
            match Pools::<T>::try_get(asset_id) {
                Ok(pool) => {
                    ensure!(
                        pool.state == State::Normal,
                        Error::<T>::TransferringInProgress
                    );
                    // For MVP the default percentage of transferring funds per transaction will be
                    // 10%. It can be changed in the future.
                    let default_percentage_of_funds = Percent::from_percent(10);
                    Self::transferring_funds(
                        vault_id,
                        asset_id,
                        pool_id,
                        percentage_of_funds.unwrap_or(default_percentage_of_funds),
                    )?;
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

        fn transferring_funds(
            vault_id: &T::VaultId,
            asset_id: T::AssetId,
            new_pool_id: T::PoolId,
            percentage_of_funds: Percent,
        ) -> DispatchResult {
            let pool_id_and_state = Self::pools(asset_id).ok_or(Error::<T>::PoolNotFound)?;
            let pool_id_deduce = pool_id_and_state.pool_id;
            let strategy_vaults = T::Vault::get_strategies(vault_id)?;
            let strategy_vault_account = strategy_vaults.last().ok_or(Error::<T>::NoStrategies)?.0;
            let lp_token_id = T::Pablo::lp_token(pool_id_deduce)?;
            let mut balance_of_lp_token =
                T::Currency::balance(lp_token_id, &strategy_vault_account);
            Pools::<T>::mutate(asset_id, |pool| {
                *pool = Some(PoolState {
                    pool_id: pool_id_deduce,
                    state: State::Transferring,
                });
            });
            let pertcentage_of_funds: u128 = percentage_of_funds.deconstruct().into();
            let balance_of_lp_tokens_decimal = T::Convert::convert(balance_of_lp_token);
            let balance_to_withdraw_per_transaction =
                T::Convert::convert(safe_multiply_by_rational(
                    balance_of_lp_tokens_decimal,
                    pertcentage_of_funds,
                    100_u128,
                )?);
            while balance_of_lp_token > balance_to_withdraw_per_transaction {
                Self::do_tranferring_funds(
                    vault_id,
                    &strategy_vault_account,
                    new_pool_id,
                    pool_id_deduce,
                    balance_to_withdraw_per_transaction,
                )?;
                balance_of_lp_token =
                    balance_of_lp_token.safe_sub(&balance_to_withdraw_per_transaction)?;
            }
            if balance_of_lp_token > T::Balance::zero() {
                Self::do_tranferring_funds(
                    vault_id,
                    &strategy_vault_account,
                    new_pool_id,
                    pool_id_deduce,
                    balance_to_withdraw_per_transaction,
                )?;
            }
            Pools::<T>::mutate(asset_id, |pool| {
                *pool = Some(PoolState {
                    pool_id: new_pool_id,
                    state: State::Normal,
                });
            });
            Self::deposit_event(Event::FundsTransfferedToNewPool { new_pool_id });
            Ok(())
        }

        fn do_rebalance(vault_id: &T::VaultId) -> DispatchResult {
            let asset_id = T::Vault::asset_id(vault_id)?;
            let strategy_vaults = T::Vault::get_strategies(vault_id)?;
            let strategy_vault_account = strategy_vaults.last().ok_or(Error::<T>::NoStrategies)?.0;
            let pool_id_and_state = Self::pools(asset_id).ok_or(Error::<T>::PoolNotFound)?;
            let pool_id = pool_id_and_state.pool_id;
            match T::Vault::available_funds(vault_id, &Self::account_id())? {
                FundsAvailability::Withdrawable(balance) => {
                    Self::withdraw(vault_id, &strategy_vault_account, pool_id, balance)?;
                    Self::deposit_event(Event::WithdrawFunctionalityOccuredDuringRebalance {
                        vault_id: *vault_id,
                    });
                }
                FundsAvailability::Depositable(balance) => {
                    Self::deposit(vault_id, &strategy_vault_account, pool_id, balance)?;
                    Self::deposit_event(Event::DepositFunctionalityOccuredDuringRebalance {
                        vault_id: *vault_id,
                    });
                }
                FundsAvailability::MustLiquidate => {
                    Self::liquidate(vault_id, &strategy_vault_account, pool_id)?;
                    Self::deposit_event(Event::LiquidateFunctionalityOccuredDuringRebalance {
                        vault_id: *vault_id,
                    });
                }
                FundsAvailability::None => {
                    Self::deposit_event(Event::NoneFunctionalityOccuredDuringRebalance {
                        vault_id: *vault_id,
                    });
                }
            };
            Ok(())
        }

        fn withdraw(
            vault_id: &T::VaultId,
            vault_strategy_account: &T::AccountId,
            pool_id: T::PoolId,
            balance: T::Balance,
        ) -> DispatchResult {
            <T::Vault as StrategicVault>::withdraw(vault_id, vault_strategy_account, balance)?;
            T::Pablo::add_liquidity(
                vault_strategy_account,
                pool_id,
                balance,
                T::Balance::zero(),
                T::Balance::zero(),
                true,
            )
        }

        fn deposit(
            vault_id: &T::VaultId,
            vault_strategy_account: &T::AccountId,
            pool_id: T::PoolId,
            balance: T::Balance,
        ) -> DispatchResult {
            let lp_price = T::Pablo::get_price_of_lp_token(pool_id)?;
            let lp_redeem = balance.safe_div(&lp_price)?;
            T::Pablo::remove_liquidity_single_asset(
                vault_strategy_account,
                pool_id,
                lp_redeem,
                T::Balance::zero(),
            )?;
            <T::Vault as StrategicVault>::deposit(vault_id, vault_strategy_account, balance)
        }

        fn liquidate(
            vault_id: &T::VaultId,
            vault_strategy_account: &T::AccountId,
            pool_id: T::PoolId,
        ) -> DispatchResult {
            let lp_token_id = T::Pablo::lp_token(pool_id)?;
            let balance_of_lp_token = T::Currency::balance(lp_token_id, vault_strategy_account);
            T::Pablo::remove_liquidity_single_asset(
                vault_strategy_account,
                pool_id,
                balance_of_lp_token,
                T::Balance::zero(),
            )?;
            let balance =
                T::Currency::balance(T::Vault::asset_id(vault_id)?, vault_strategy_account);
            <T::Vault as StrategicVault>::deposit(vault_id, vault_strategy_account, balance)
        }

        #[transactional]
        fn do_tranferring_funds(
            vault_id: &T::VaultId,
            vault_account: &T::AccountId,
            new_pool_id: T::PoolId,
            pool_id_deduce: T::PoolId,
            balance: T::Balance,
        ) -> DispatchResult {
            Self::deposit(vault_id, vault_account, pool_id_deduce, balance)?;
            Self::withdraw(vault_id, vault_account, new_pool_id, balance)?;
            Ok(())
        }
    }
}

// -------------------------------------------------------------------------------------------------
//                                            Unit Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod unit_tests {}
