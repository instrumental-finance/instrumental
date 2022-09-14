use hex_literal::hex;
use proptest::strategy::Just;
use sp_core::sr25519::{Public, Signature};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub const ADMIN: AccountId = Public(hex!(
    "0000000000000000000000000000000000000000000000000000000000000000"
));
pub const ALICE: AccountId = Public(hex!(
    "0000000000000000000000000000000000000000000000000000000000000001"
));
pub const BOB: AccountId = Public(hex!(
    "0000000000000000000000000000000000000000000000000000000000000002"
));
// TODO(saruman9): remove or use in the future
pub const _CHARLIE: AccountId = Public(hex!(
    "0000000000000000000000000000000000000000000000000000000000000003"
));
pub const _DAVE: AccountId = Public(hex!(
    "0000000000000000000000000000000000000000000000000000000000000004"
));
pub const _EVEN: AccountId = Public(hex!(
    "0000000000000000000000000000000000000000000000000000000000000005"
));

// TODO(saruman9): remove or use in the future
pub const fn _accounts() -> [Just<AccountId>; 5] {
    [
        Just(ALICE),
        Just(BOB),
        Just(_CHARLIE),
        Just(_DAVE),
        Just(_EVEN),
    ]
}
