#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	// -------------------------------------------------------------------------------------------
	//                                   Imports and Dependencies                                
	// -------------------------------------------------------------------------------------------
	use crate::weights::WeightInfo;

	use frame_support::pallet_prelude::*;
	use frame_system::{
		ensure_signed,
		pallet_prelude::*,
	};

	// -------------------------------------------------------------------------------------------
	//                                Declaration Of The Pallet Type                              
	// -------------------------------------------------------------------------------------------

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// -------------------------------------------------------------------------------------------
	//                                         Config Trait                                       
	// -------------------------------------------------------------------------------------------

	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[allow(missing_docs)]
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;
	}

	// -------------------------------------------------------------------------------------------
    //                                         Pallet Types                                       
	// -------------------------------------------------------------------------------------------

	// -------------------------------------------------------------------------------------------
    //                                       Runtime  Storage                                     
	// -------------------------------------------------------------------------------------------

	// -------------------------------------------------------------------------------------------
    //                                        Runtime Events                                      
	// -------------------------------------------------------------------------------------------

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Test {
			issuer: T::AccountId
		},
	}

	// -------------------------------------------------------------------------------------------
    //                                        Runtime Errors                                      
	// -------------------------------------------------------------------------------------------

	#[pallet::error]
	pub enum Error<T> {
	}

	// -------------------------------------------------------------------------------------------
    //                                            Hooks                                                
	// -------------------------------------------------------------------------------------------

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {

	}

	// -------------------------------------------------------------------------------------------
    //                                          Extrinsics                                         
	// -------------------------------------------------------------------------------------------

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn test(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			// Requirement 0) This extrinsic must be signed 
			let from = ensure_signed(origin)?;

			Self::deposit_event(Event::Test { issuer: from });

			Ok(().into())
		}
	}

	// -------------------------------------------------------------------------------------------
    //                                  Instrumental Strategy                                     
	// -------------------------------------------------------------------------------------------

	impl<T: Config> Pallet<T> {

	}
}

// -----------------------------------------------------------------------------------------------
//                                             Unit Tests                                         
// -----------------------------------------------------------------------------------------------

#[cfg(test)]
mod unit_tests {
}