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

mod helper;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();
	// code snippet for
	// * build a block with polkadot mem
	// * build block with TestEnv
	// * open database of polkadot/centrifuge
	// * showing gavins verified account
	// * showing my verified account
	// * making me council instead of gavin
	// * submitting root origin
	// * submitting extrinsic with real signing
	// * querying the gensis state - maybe balances
}

/// Gavin's verified polkadot account - sub for council:
/// 13RDY9nrJpyTDBSUdBw12dGwhk19sGwsrVZ2bxkzYHBSagP2
/// 0x6af08f6bb841825b168ddf79837e70d88d75e1c5b290b74fa97cedfd668dd22c
const GAVIN: [u8; 32] = [0u8; 32];

/// My verified polkadot account:
/// 13p7q4N8aQqnGJRGnevq8e8k8rsXkF8S7u1E5694XtRviJc
/// 0x02251a6e0194f2ee97f988a8f2e779a06a73b2ee3e1a54f413a3b9f6f8d04f6c
const MUSTERMEISZER: [u8; 32] = [0u8; 32];

/// Switches the council prime account on polkadot
fn switch_council_prime() {}
