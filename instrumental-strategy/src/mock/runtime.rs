use frame_support::{parameter_types, traits::Everything, PalletId};
use frame_system::{EnsureRoot, EnsureSigned};
use orml_traits::parameter_type_with_key;
use pallet_collective::EnsureProportionAtLeast;
use primitives::currency::{CurrencyId, ValidateCurrencyId};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{ConvertInto, IdentityLookup},
    Permill,
};

use super::fnft;
use crate as instrumental_strategy;

pub type AccountId = u128;
pub type Amount = i128;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type Moment = composable_traits::time::Timestamp;
pub type PoolId = u128;
pub type PositionId = u128;
pub type RewardPoolId = u16;
pub type VaultId = u64;

// These time units are defined in number of blocks.
pub const MILLISECS_PER_BLOCK: Moment = 3000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const MAX_ASSOCIATED_VAULTS: u32 = 10;

// -------------------------------------------------------------------------------------------------
//                                              Config
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for MockRuntime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = Everything;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = BlockNumber;
    type BlockWeights = ();
    type Call = Call;
    type DbWeight = ();
    type Event = Event;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

// -------------------------------------------------------------------------------------------------
//                                             Balances
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const BalanceExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for MockRuntime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = BalanceExistentialDeposit;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                              Tokens
// -------------------------------------------------------------------------------------------------

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        0_u128
    };
}

type ReserveIdentifier = [u8; 8];
impl orml_tokens::Config for MockRuntime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type DustRemovalWhitelist = Everything;
    type Event = Event;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = ();
    type MaxReserves = frame_support::traits::ConstU32<2>;
    type OnDust = ();
    type OnKilledTokenAccount = ();
    type OnNewTokenAccount = ();
    type ReserveIdentifier = ReserveIdentifier;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                         Currency Factory
// -------------------------------------------------------------------------------------------------

impl pallet_currency_factory::Config for MockRuntime {
    type AddOrigin = EnsureRoot<AccountId>;
    type AssetId = CurrencyId;
    type Balance = Balance;
    type Event = Event;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                               Vault
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const MaxStrategies: usize = 255;
    pub const CreationDeposit: Balance = 10;
    pub const ExistentialDeposit: Balance = 1000;
    pub const RentPerBlock: Balance = 1;
    pub const MinimumDeposit: Balance = 0;
    pub const MinimumWithdrawal: Balance = 0;
    pub const VaultPalletId: PalletId = PalletId(*b"cubic___");
    pub const TombstoneDuration: u64 = 42;
}

impl pallet_vault::Config for MockRuntime {
    type AssetId = CurrencyId;
    type Balance = Balance;
    type Convert = ConvertInto;
    type CreationDeposit = CreationDeposit;
    type Currency = Tokens;
    type CurrencyFactory = LpTokenFactory;
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxStrategies = MaxStrategies;
    type MinimumDeposit = MinimumDeposit;
    type MinimumWithdrawal = MinimumWithdrawal;
    type NativeCurrency = Balances;
    type PalletId = VaultPalletId;
    type RentPerBlock = RentPerBlock;
    type TombstoneDuration = TombstoneDuration;
    type VaultId = VaultId;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                            Collective
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type InstrumentalPabloCollective = pallet_collective::Instance1;
impl pallet_collective::Config<InstrumentalPabloCollective> for MockRuntime {
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type Event = Event;
    type MaxMembers = CouncilMaxMembers;
    type MaxProposals = CouncilMaxProposals;
    type MotionDuration = CouncilMotionDuration;
    type Origin = Origin;
    type Proposal = Call;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<MockRuntime>;
}

// -------------------------------------------------------------------------------------------------
//                                        Governance Registry
// -------------------------------------------------------------------------------------------------

impl pallet_governance_registry::Config for MockRuntime {
    type AssetId = CurrencyId;
    type Event = Event;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                              Assets
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const NativeAssetId: CurrencyId = CurrencyId::PICA;
}

impl pallet_assets::Config for MockRuntime {
    type AdminOrigin = EnsureRoot<AccountId>;
    type AssetId = CurrencyId;
    type Balance = Balance;
    type CurrencyValidator = ValidateCurrencyId;
    type GenerateCurrencyId = LpTokenFactory;
    type GovernanceRegistry = GovernanceRegistry;
    type MultiCurrency = Tokens;
    type NativeAssetId = NativeAssetId;
    type NativeCurrency = Balances;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                             Timestamp
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const MinimumPeriod: u64 = MILLISECS_PER_BLOCK / 2;
}

impl pallet_timestamp::Config for MockRuntime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = Moment;
    type OnTimestampSet = ();
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                          Staking Rewards
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const StakingRewardsPalletId: PalletId = PalletId(*b"stk_rwrd");
    pub const MaxStakingDurationPresets: u32 = 10;
    pub const MaxRewardConfigsPerPool: u32 = 10;
}

impl pallet_staking_rewards::Config for MockRuntime {
    type AssetId = CurrencyId;
    type Assets = Tokens;
    type Balance = Balance;
    type CurrencyFactory = LpTokenFactory;
    type Event = Event;
    type FinancialNft = fnft::MockFnft;
    type FinancialNftInstanceId = u64;
    type MaxRewardConfigsPerPool = MaxRewardConfigsPerPool;
    type MaxStakingDurationPresets = MaxStakingDurationPresets;
    type PalletId = StakingRewardsPalletId;
    type PositionId = PositionId;
    type ReleaseRewardsPoolsBatchSize = frame_support::traits::ConstU8<13>;
    type RewardPoolCreationOrigin = EnsureRoot<Self::AccountId>;
    type RewardPoolId = RewardPoolId;
    type RewardPoolUpdateOrigin = EnsureRoot<Self::AccountId>;
    type UnixTime = Timestamp;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                            Pablo (AMM)
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const PabloPalletId: PalletId = PalletId(*b"pablo_pa");
    pub const MinSaleDuration: BlockNumber = 3600 / 12;
    pub const MaxSaleDuration: BlockNumber = 30 * 24 * 3600 / 12;
    pub const MaxInitialWeight: Permill = Permill::from_percent(95);
    pub const MinFinalWeight: Permill = Permill::from_percent(5);
    pub const TWAPInterval: Moment = MILLISECS_PER_BLOCK * 10;
    pub const MaxStakingRewardPools: u32 = 10;
    pub const MillisecsPerBlock: u32 = MILLISECS_PER_BLOCK as u32;
}

impl pallet_pablo::Config for MockRuntime {
    type AssetId = CurrencyId;
    type Assets = Assets;
    type Balance = Balance;
    type Convert = ConvertInto;
    type CurrencyFactory = LpTokenFactory;
    type EnableTwapOrigin = EnsureRoot<AccountId>;
    type Event = Event;
    type LbpMaxInitialWeight = MaxInitialWeight;
    type LbpMaxSaleDuration = MaxSaleDuration;
    type LbpMinFinalWeight = MinFinalWeight;
    type LbpMinSaleDuration = MinSaleDuration;
    type LocalAssets = LpTokenFactory;
    type ManageStaking = StakingRewards;
    type MaxRewardConfigsPerPool = MaxRewardConfigsPerPool;
    type MaxStakingDurationPresets = MaxStakingDurationPresets;
    type MaxStakingRewardPools = MaxStakingRewardPools;
    type MsPerBlock = MillisecsPerBlock;
    type PalletId = PabloPalletId;
    type PoolCreationOrigin = EnsureSigned<Self::AccountId>;
    type PoolId = PoolId;
    type ProtocolStaking = StakingRewards;
    type RewardPoolId = RewardPoolId;
    type TWAPInterval = TWAPInterval;
    type Time = Timestamp;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                    Instrumental Pablo Strategy
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const MaxAssociatedVaults: u32 = MAX_ASSOCIATED_VAULTS;
    pub const InstrumentalPabloStrategyPalletId: PalletId = PalletId(*b"strmxpab");
}

impl pallet_instrumental_strategy_pablo::Config for MockRuntime {
    type AssetId = CurrencyId;
    type Balance = Balance;
    type Convert = ConvertInto;
    type Currency = Tokens;
    type Event = Event;
    type ExternalOrigin = EnsureProportionAtLeast<AccountId, InstrumentalPabloCollective, 2, 3>;
    type MaxAssociatedVaults = MaxAssociatedVaults;
    type Pablo = Pablo;
    type PalletId = InstrumentalPabloStrategyPalletId;
    type PoolId = PoolId;
    type Vault = Vault;
    type VaultId = VaultId;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                       Instrumental Strategy
// -------------------------------------------------------------------------------------------------

parameter_types! {
    pub const InstrumentalStrategyPalletId: PalletId = PalletId(*b"dynamic_");
}

impl instrumental_strategy::Config for MockRuntime {
    type AssetId = CurrencyId;
    type Balance = Balance;
    type Event = Event;
    type MaxAssociatedVaults = MaxAssociatedVaults;
    type PabloStrategy = PabloStrategy;
    type PalletId = InstrumentalStrategyPalletId;
    type PoolId = PoolId;
    type Vault = Vault;
    type VaultId = VaultId;
    type WeightInfo = ();
}

// -------------------------------------------------------------------------------------------------
//                                         Construct Runtime
// -------------------------------------------------------------------------------------------------

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<MockRuntime>;
type Block = frame_system::mocking::MockBlock<MockRuntime>;

frame_support::construct_runtime!(
    pub enum MockRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage},
        CollectiveInstrumental:
        pallet_collective::<Instance1>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},

        LpTokenFactory: pallet_currency_factory::{Pallet, Storage, Event<T>},
        Vault: pallet_vault::{Pallet, Call, Storage, Event<T>},
        GovernanceRegistry: pallet_governance_registry::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage},
        StakingRewards: pallet_staking_rewards::{Pallet, Storage, Call, Event<T>},
        Pablo: pallet_pablo::{Pallet, Call, Storage, Event<T>},

        PabloStrategy: pallet_instrumental_strategy_pablo::{Pallet, Call, Storage, Event<T>},
        InstrumentalStrategy: instrumental_strategy::{Pallet, Call, Storage, Event<T>},
    }
);

// -------------------------------------------------------------------------------------------------
//                                       Externalities Builder
// -------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct ExtBuilder {}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<MockRuntime>()
            .unwrap();

        t.into()
    }
}
