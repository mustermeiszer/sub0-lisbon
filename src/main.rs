// Copyright 2021  Frederik Gartenmeister
//
// This is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).
// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
#![allow(unused)]

use std::path::PathBuf;

use frame_support::{
	pallet_prelude::StorageMap,
	traits::{Currency, StorageInstance},
	Blake2_128Concat,
};
use fudge::{
	primitives::{Chain, ParaId},
	state::StateProvider,
};
use pallet_vesting::VestingInfo;
use polkadot_core_primitives::AccountId;
use sp_runtime::{generic::BlockId, MultiAddress, Storage};

use crate::helper::{log, log_empty, CENTRIFUGE_ID, DUMMY_ID};

mod helper;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	// 1. Start in memory default standalone
	//    a Without genesis
	/*
	let builder = helper::default_in_mem(Storage::default()).await;
	 */

	//    b. Show usage of StateProvider
	//        - polkadot_runtime::WASM_BINARY.unwrap() needs to be there
	//        - build a block with logs
	//        - insert some funding
	/*
	{
		let genesis = StateProvider::new(polkadot_runtime::WASM_BINARY.unwrap());
		let mut builder = helper::default_in_mem(genesis).await;

		builder.build_block();
		builder.import_block();

		builder.build_block();
		builder.import_block();

		let past = builder
			.with_state_at(BlockId::Number(1), || {
				frame_system::Pallet::<polkadot_runtime::Runtime>::block_number()
			})
			.unwrap();
		log(past);

		let curr = builder
			.with_state(|| frame_system::Pallet::<polkadot_runtime::Runtime>::block_number())
			.unwrap();
		log(curr);

		let account = builder
			.with_state(|| {
				frame_system::Pallet::<polkadot_runtime::Runtime>::account(AccountId::new(
					MUSTERMEISZER,
				))
			})
			.unwrap();
		log(account);

		/*
		// Does NOT mutate state
		builder
			.with_state(|| {
				let mut account = frame_system::Pallet::<polkadot_runtime::Runtime>::account(
					as_account(MUSTERMEISZER),
				);

				account.data.free = 10000000000;

				frame_system::Account::<polkadot_runtime::Runtime>::set(
					as_account(MUSTERMEISZER),
					account,
				);
			})
			.unwrap();
		}
		 */

		builder
			.with_mut_state(|| {
				let mut account = frame_system::Pallet::<polkadot_runtime::Runtime>::account(
					as_account(MUSTERMEISZER),
				);

				account.data.free = 10000000000;

				frame_system::Account::<polkadot_runtime::Runtime>::set(
					as_account(MUSTERMEISZER),
					account,
				);
			})
			.unwrap();

		let account = builder
			.with_state(|| {
				frame_system::Pallet::<polkadot_runtime::Runtime>::account(AccountId::new(
					MUSTERMEISZER,
				))
			})
			.unwrap();
		log(account);
	}
	*/

	// 2. Build a block with the disk TestEnv with logs
	//    - explain how the parachains are updated on the relay chain side
	//    - memory database starts at the beginning everytime
	//    - disk based databases start at latest block
	/*
	{
		let mut test_env = helper::test_env().await;
		test_env.evolve().unwrap();

		let (polkadot_block, dummy_head, centrifuge_head) = test_env
			.with_state(Chain::Relay, || {
				(
					frame_system::Pallet::<polkadot_runtime::Runtime>::block_number(),
					polkadot_runtime_parachains::paras::Pallet::<polkadot_runtime::Runtime>::para_head(
						ParaId::from(DUMMY_ID),
					),
					polkadot_runtime_parachains::paras::Pallet::<polkadot_runtime::Runtime>::para_head(
						ParaId::from(CENTRIFUGE_ID),
					),
				)
			})
			.unwrap();
		log(format!("Polkadot latest block: {}", polkadot_block));

		let centrifuge_block = test_env
			.with_state(Chain::Para(CENTRIFUGE_ID), || {
				frame_system::Pallet::<centrifuge_runtime::Runtime>::block_number()
			})
			.unwrap();
		log(format!("Centrifuge latest block: {}", centrifuge_block));
		assert_eq!(test_env.centrifuge.head(), centrifuge_head.unwrap());

		let dummy_block = test_env
			.with_state(Chain::Para(DUMMY_ID), || {
				frame_system::Pallet::<test_parachain::Runtime>::block_number()
			})
			.unwrap();
		log(format!("Dummy latest block: {}", dummy_block));
		assert_eq!(test_env.dummy.head(), dummy_head.unwrap());
	}
	 */

	// 3. Query real database
	//    a. Retrieve latest block
	//       - show council
	/*
	{
		const REAL_DB: &'static str = "/Volumes/Ext/sub0/relay-chain/chains/polkadot/db/full";
		let builder = helper::query_disk_builder_polkadot(PathBuf::from(REAL_DB)).await;
		let (prime, council) = builder.with_state(|| get_council_info()).unwrap();
		log(prime);
		council.into_iter().for_each(|member| log(member));
	}
	 */

	//    b. Retrieve block of
	//       Centrifuge with old vestings
	/*
	{
		const REAL_DB: &'static str = "/Volumes/Ext/sub0/chains/centrifuge/db/full";
		let builder = helper::query_disk_builder_centrifuge(PathBuf::from(REAL_DB)).await;
		let vestings = builder
			.with_state_at(BlockId::Number(500000), || {
				let mut vestings = Vec::new();
				for (count, (pub_key, data)) in
					pallet_vesting::Vesting::<polkadot_runtime::Runtime>::iter().enumerate()
				{
					if count < 20 {
						vestings.push((pub_key, data))
					} else {
						break;
					}
				}

				vestings
			})
			.unwrap();

		log("Vestings at Block 500000:");
		vestings
			.into_iter()
			.for_each(|(counter, vesting)| log(vesting));

		let vestings = builder
			.with_state(|| {
				let mut vestings = Vec::new();
				for (count, (pub_key, data)) in
					pallet_vesting::Vesting::<polkadot_runtime::Runtime>::iter().enumerate()
				{
					if count < 20 {
						vestings.push((pub_key, data))
					} else {
						break;
					}
				}

				vestings
			})
			.unwrap();

		log("Vestings at latest block:");
		vestings
			.into_iter()
			.for_each(|(counter, vesting)| log(vesting));
	}
	*/

	//    c. Mutate latest block
	//       - Change the prime council member
	//       - Log the new council members
	//       - Log the new prime member
	/*
	{
		const REAL_DB: &'static str = "/Volumes/Ext/sub0/relay-chain/chains/polkadot/db/full";
		let mut builder = helper::query_disk_builder_polkadot(PathBuf::from(REAL_DB)).await;
		{
			let (prime, council) = builder.with_state(|| get_council_info()).unwrap();
			log(prime);
			council.into_iter().for_each(|member| log(member));
		}
		log_empty();
		log("Mutating storage now..");
		log_empty();
		{
			builder
				.with_mut_state(|| {
					pallet_collective::Members::<
						polkadot_runtime::Runtime,
						pallet_collective::Instance1,
					>::try_mutate(|council| {
						council.retain(|member| member != &as_account(GAVIN));
						council.push(as_account(MUSTERMEISZER));
						Ok::<(), ()>(())
					});

					pallet_collective::Prime::<
						polkadot_runtime::Runtime,
						pallet_collective::Instance1,
					>::set(Some(as_account(MUSTERMEISZER)));
				})
				.unwrap();
		}
		log_empty();
		log("Fetching storage after mutation:");
		log_empty();
		{
			let (prime, council) = builder.with_state(|| get_council_info()).unwrap();
			log(prime);
			council.into_iter().for_each(|member| log(member));
		}
	}
	 */

	// 4. Start in memory standalone builder
	//    a. Mutate state with `Origin::root` by calling `set_storage`
	//       - show changed outcome
	/*
	{
		let genesis = StateProvider::new(polkadot_runtime::WASM_BINARY.unwrap());
		let mut builder = helper::default_in_mem(genesis).await;
		builder
			.with_mut_state(|| {
				frame_system::Pallet::<polkadot_runtime::Runtime>::set_storage(
					polkadot_runtime::Origin::root(),
					vec![(
						sp_storage::well_known_keys::CODE.to_vec(),
						centrifuge_runtime::WASM_BINARY.unwrap().to_vec(),
					)],
				)
				.unwrap();
			})
			.unwrap();

		let new_code = builder
			.with_state(|| {
				frame_support::storage::unhashed::get_raw(sp_storage::well_known_keys::CODE)
					.unwrap()
			})
			.unwrap();

		assert_eq!(new_code, centrifuge_runtime::WASM_BINARY.unwrap().to_vec());
	}
	*/
}

/// Gavin's verified polkadot account - sub for council:
/// 13RDY9nrJpyTDBSUdBw12dGwhk19sGwsrVZ2bxkzYHBSagP2
/// 0x6af08f6bb841825b168ddf79837e70d88d75e1c5b290b74fa97cedfd668dd22c
const GAVIN: [u8; 32] = [
	106, 240, 143, 107, 184, 65, 130, 91, 22, 141, 223, 121, 131, 126, 112, 216, 141, 117, 225,
	197, 178, 144, 183, 79, 169, 124, 237, 253, 102, 141, 210, 44,
];

/// My verified polkadot account:
/// 13p7q4N8aQqnGJRGnevq8e8k8rsXkF8S7u1E5694XtRviJc
/// 0x02251a6e0194f2ee97f988a8f2e779a06a73b2ee3e1a54f413a3b9f6f8d04f6c
const MUSTERMEISZER: [u8; 32] = [
	2, 37, 26, 110, 1, 148, 242, 238, 151, 249, 136, 168, 242, 231, 121, 160, 106, 115, 178, 238,
	62, 26, 84, 244, 19, 163, 185, 246, 248, 208, 79, 108,
];

/// Converts bytes into AccountId type
fn as_account(bytes: [u8; 32]) -> AccountId {
	AccountId::new(bytes)
}

/// Implements the prefix for the patricia-trie to use
pub struct VestingPrefix;
impl StorageInstance for VestingPrefix {
	const STORAGE_PREFIX: &'static str = "Vesting";

	fn pallet_prefix() -> &'static str {
		"Vesting"
	}
}

type BalanceOf<T> = <<T as pallet_vesting::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

/// The actual map type for vestings.
///
/// We need to rebuild it here manually as the stored type
/// changed. So using the latest type does not work for older
/// blocks.
#[allow(type_alias_bounds)]
pub type Vesting<T: pallet_vesting::Config> = StorageMap<
	VestingPrefix,
	Blake2_128Concat,
	T::AccountId,
	VestingInfo<BalanceOf<T>, T::BlockNumber>,
>;

/// Retrieves Council and Prime
fn get_council_info() -> (Option<String>, Vec<String>) {
	let mut council = Vec::new();
	pallet_collective::Members::<polkadot_runtime::Runtime, pallet_collective::Instance1>::get()
		.into_iter()
		.for_each(|member| council.push(get_display_name(member)));

	let prime =
		pallet_collective::Pallet::<polkadot_runtime::Runtime, pallet_collective::Instance1>::prime().map(|prime| get_display_name(prime));

	(prime, council)
}

/// Retrieve the display name for an account, if possible, from the
/// identity pallet.
fn get_display_name(member: AccountId) -> String {
	pallet_identity::Pallet::<polkadot_runtime::Runtime>::identity(member.clone())
		.map(|registration| registration.info.display)
		.or_else(|| {
			pallet_identity::Pallet::<polkadot_runtime::Runtime>::super_of(member).map(
				|(super_acc, _)| {
					pallet_identity::Pallet::<polkadot_runtime::Runtime>::identity(super_acc)
						.unwrap()
						.info
						.display
				},
			)
		})
		.map(|display| match display {
			pallet_identity::Data::Raw(bytes) => {
				String::from_utf8(bytes.into()).unwrap_or(String::from("UNKNOWN"))
			}
			_ => String::from("UNKNOWN"),
		})
		.unwrap_or(String::from("UNKNOWN"))
}
