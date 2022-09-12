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

pub mod instrumental;
