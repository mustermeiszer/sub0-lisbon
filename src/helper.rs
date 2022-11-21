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

use std::{fmt::Debug, path::PathBuf, sync::Arc};

use fudge::{
	backend::MemDb,
	digest::{DigestCreator, DigestProvider, FudgeAuraDigest, FudgeBabeDigest},
	inherent::{
		FudgeDummyInherentRelayParachain, FudgeInherentParaParachain, FudgeInherentTimestamp,
	},
	BackendProvider, ParachainBuilder, RelaychainBuilder, StandaloneBuilder, TWasmExecutor,
};
use sc_service::{TFullBackend, TFullClient};
use sp_consensus_babe::SlotDuration;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::{generic::BlockId, testing::H256, BuildStorage};
use tokio::runtime::Handle;

/// Path to centrifuge db
pub fn centrifuge_db_path() -> PathBuf {
	let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("data");
	path.push("centrifuge");
	path
}

/// constant for polkadot db
pub fn polkadot_db_path() -> PathBuf {
	let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("data");
	path.push("polkadot");
	path
}

/// Get Standalone mem-db builder
///
/// This builder uses the Polkadot runtime
pub async fn default_in_mem(
	genesis: impl BuildStorage + 'static,
) -> StandaloneBuilder<
	polkadot_core_primitives::Block,
	polkadot_runtime::RuntimeApi,
	impl CreateInherentDataProviders<polkadot_core_primitives::Block, ()>,
	impl DigestCreator<polkadot_core_primitives::Block>,
> {
	let mut init = fudge::initiator::default_with(Handle::current(), MemDb::new());
	init.with_genesis(Box::new(genesis));

	StandaloneBuilder::new(init, cidp_and_dp_relay)
}

fn cidp_and_dp_relay(
	client: Arc<
		TFullClient<polkadot_core_primitives::Block, polkadot_runtime::RuntimeApi, TWasmExecutor>,
	>,
) -> (
	impl CreateInherentDataProviders<polkadot_core_primitives::Block, ()>,
	impl DigestCreator<polkadot_core_primitives::Block>,
) {
	// Init timestamp instance_id
	let instance_id =
		FudgeInherentTimestamp::create_instance(sp_std::time::Duration::from_secs(6), None);

	let cidp = move |clone_client: Arc<
		TFullClient<polkadot_core_primitives::Block, polkadot_runtime::RuntimeApi, TWasmExecutor>,
	>| {
		move |parent: H256, ()| {
			let client = clone_client.clone();
			let parent_header = client
				.header(&BlockId::Hash(parent.clone()))
				.unwrap()
				.unwrap();

			async move {
				let uncles =
					sc_consensus_uncles::create_uncles_inherent_data_provider(&*client, parent)?;

				let timestamp = FudgeInherentTimestamp::get_instance(instance_id)
					.expect("Instance is initialized. qed");

				let slot =
                    sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        timestamp.current_time(),
                        SlotDuration::from_millis(std::time::Duration::from_secs(6).as_millis() as u64),
                    );

				let relay_para_inherent = FudgeDummyInherentRelayParachain::new(parent_header);
				Ok((timestamp, uncles, slot, relay_para_inherent))
			}
		}
	};

	let dp = move |parent, inherents| async move {
		let mut digest = sp_runtime::Digest::default();

		let babe = FudgeBabeDigest::<polkadot_core_primitives::Block>::new();
		babe.append_digest(&mut digest, &parent, &inherents).await?;

		Ok(digest)
	};

	(cidp(client), dp)
}
/// Get Polkadot relay builder
pub async fn polkadot_builder(
	db: impl BackendProvider<
		polkadot_core_primitives::Block,
		Backend = TFullBackend<polkadot_core_primitives::Block>,
	>,
	genesis: Option<impl BuildStorage + 'static>,
) -> RelaychainBuilder<
	polkadot_core_primitives::Block,
	polkadot_runtime::RuntimeApi,
	polkadot_runtime::Runtime,
	impl CreateInherentDataProviders<polkadot_core_primitives::Block, ()>,
	impl DigestCreator<polkadot_core_primitives::Block>,
> {
	let mut init = fudge::initiator::default_with(Handle::current(), db);
	if let Some(genesis) = genesis {
		init.with_genesis(Box::new(genesis));
	}

	RelaychainBuilder::new(init, cidp_and_dp_relay)
}

/// Get Centrifuge para-builder
pub async fn centrifuge_builder(
	inherent_builder: fudge::InherentBuilder<
		TFullClient<polkadot_core_primitives::Block, polkadot_runtime::RuntimeApi, TWasmExecutor>,
		TFullBackend<polkadot_core_primitives::Block>,
	>,
	db: impl BackendProvider<
		centrifuge_runtime::Block,
		Backend = TFullBackend<centrifuge_runtime::Block>,
	>,
	genesis: Option<impl BuildStorage + 'static>,
) -> ParachainBuilder<
	centrifuge_runtime::Block,
	centrifuge_runtime::RuntimeApi,
	impl CreateInherentDataProviders<centrifuge_runtime::Block, ()>,
	impl DigestCreator<centrifuge_runtime::Block>,
> {
	let mut init = fudge::initiator::default_with(Handle::current(), db);
	if let Some(genesis) = genesis {
		init.with_genesis(Box::new(genesis));
	};

	// Init timestamp instance_id
	let instance_id_para =
		FudgeInherentTimestamp::create_instance(sp_std::time::Duration::from_secs(12), None);

	let cidp = move |_parent: H256, ()| {
		let inherent_builder_clone = inherent_builder.clone();
		async move {
			let timestamp = FudgeInherentTimestamp::get_instance(instance_id_para)
				.expect("Instance is initialized. qed");

			let slot =
				sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					timestamp.current_time(),
					SlotDuration::from_millis(std::time::Duration::from_secs(6).as_millis() as u64),
				);
			let inherent = inherent_builder_clone.parachain_inherent().await.unwrap();
			let relay_para_inherent = FudgeInherentParaParachain::new(inherent);
			Ok((timestamp, slot, relay_para_inherent))
		}
	};

	let dp = |clone_client: Arc<
		TFullClient<centrifuge_runtime::Block, centrifuge_runtime::RuntimeApi, TWasmExecutor>,
	>| {
		move |parent, inherents| {
			let client = clone_client.clone();

			async move {
				let aura = FudgeAuraDigest::<
					centrifuge_runtime::Block,
					TFullClient<
						centrifuge_runtime::Block,
						centrifuge_runtime::RuntimeApi,
						TWasmExecutor,
					>,
				>::new(&*client);

				let digest = aura.build_digest(&parent, &inherents).await?;
				Ok(digest)
			}
		}
	};

	ParachainBuilder::new(init, |client| (cidp, dp(client)))
}

/// Get Centrifuge para-builder
async fn dummy_builder(
	inherent_builder: fudge::InherentBuilder<
		TFullClient<polkadot_core_primitives::Block, polkadot_runtime::RuntimeApi, TWasmExecutor>,
		TFullBackend<polkadot_core_primitives::Block>,
	>,
	db: impl BackendProvider<test_parachain::Block, Backend = TFullBackend<test_parachain::Block>>,
	genesis: Option<impl BuildStorage + 'static>,
) -> ParachainBuilder<
	test_parachain::Block,
	test_parachain::RuntimeApi,
	impl CreateInherentDataProviders<test_parachain::Block, ()>,
	impl DigestCreator<test_parachain::Block>,
> {
	let mut init = fudge::initiator::default_with(Handle::current(), db);
	if let Some(genesis) = genesis {
		init.with_genesis(Box::new(genesis));
	};

	// Init timestamp instance_id
	let instance_id_para =
		FudgeInherentTimestamp::create_instance(sp_std::time::Duration::from_secs(12), None);

	let cidp = move |_parent: H256, ()| {
		let inherent_builder_clone = inherent_builder.clone();
		async move {
			let timestamp = FudgeInherentTimestamp::get_instance(instance_id_para)
				.expect("Instance is initialized. qed");

			let slot =
				sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					timestamp.current_time(),
					SlotDuration::from_millis(std::time::Duration::from_secs(6).as_millis() as u64),
				);
			let inherent = inherent_builder_clone.parachain_inherent().await.unwrap();
			let relay_para_inherent = FudgeInherentParaParachain::new(inherent);
			Ok((timestamp, slot, relay_para_inherent))
		}
	};

	let dp = |clone_client: Arc<
		TFullClient<test_parachain::Block, test_parachain::RuntimeApi, TWasmExecutor>,
	>| {
		move |parent, inherents| {
			let client = clone_client.clone();

			async move {
				let aura = FudgeAuraDigest::<
					test_parachain::Block,
					TFullClient<test_parachain::Block, test_parachain::RuntimeApi, TWasmExecutor>,
				>::new(&*client);

				let digest = aura.build_digest(&parent, &inherents).await?;
				Ok(digest)
			}
		}
	};

	ParachainBuilder::new(init, |client| (cidp, dp(client)))
}

// setup for a companion environment with centrifuge chain from databases
pub async fn test_env() -> TestEnv {
	todo!()
}

type RCidp = Box<
	dyn CreateInherentDataProviders<
		polkadot_core_primitives::Block,
		(),
		InherentDataProviders = (
			FudgeInherentTimestamp,
			sp_consensus_babe::inherents::InherentDataProvider,
			sp_authorship::InherentDataProvider<polkadot_runtime::Header>,
			FudgeDummyInherentRelayParachain<polkadot_runtime::Header>,
		),
	>,
>;
type CfgCidp = Box<
	dyn CreateInherentDataProviders<
		centrifuge_runtime::Block,
		(),
		InherentDataProviders = (
			FudgeInherentTimestamp,
			sp_consensus_aura::inherents::InherentDataProvider,
			FudgeInherentParaParachain,
		),
	>,
>;
type CfgDp = Box<dyn DigestCreator<centrifuge_runtime::Block> + Send + Sync>;
type DummyCidp = Box<
	dyn CreateInherentDataProviders<
		test_parachain::Block,
		(),
		InherentDataProviders = (
			FudgeInherentTimestamp,
			sp_consensus_aura::inherents::InherentDataProvider,
			FudgeInherentParaParachain,
		),
	>,
>;
type DummyDp = Box<dyn DigestCreator<test_parachain::Block> + Send + Sync>;
type RDp = Box<dyn DigestCreator<polkadot_core_primitives::Block> + Send + Sync>;

#[fudge::companion]
pub struct TestEnv {
	#[fudge::parachain(2999u32)]
	dummy: ParachainBuilder<test_parachain::Block, test_parachain::RuntimeApi, DummyCidp, DummyDp>,
	#[fudge::parachain(2031u32)]
	centrifuge:
		ParachainBuilder<centrifuge_runtime::Block, centrifuge_runtime::RuntimeApi, CfgCidp, CfgDp>,
	#[fudge::relaychain]
	polkadot: RelaychainBuilder<
		polkadot_core_primitives::Block,
		polkadot_runtime::RuntimeApi,
		polkadot_runtime::Runtime,
		RCidp,
		RDp,
	>,
}

// helper for logging stuff
pub fn log(log: impl Debug) {
	tracing::event!(tracing::Level::INFO, "Sub0 - Lisbon: DEBUGGING: {:?}", log);
}
