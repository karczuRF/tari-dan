//   Copyright 2023 The Tari Project
//   SPDX-License-Identifier: BSD-3-Clause

use diesel::{Connection, SqliteConnection};
use rand::rngs::OsRng;
use tari_common_types::types::{FixedHash, PublicKey};
use tari_crypto::keys::PublicKey as _;
use tari_dan_common_types::{Epoch, NumPreshards, PeerAddress, ShardGroup, SubstateAddress};
use tari_dan_storage::global::{GlobalDb, ValidatorNodeDb};
use tari_dan_storage_sqlite::global::SqliteGlobalDbAdapter;
use tari_utilities::ByteArray;

fn create_db() -> GlobalDb<SqliteGlobalDbAdapter<PeerAddress>> {
    // std::fs::remove_file("/tmp/tmptmp.db").ok();
    // let conn = SqliteConnection::establish("file:///tmp/tmptmp.db").unwrap();
    let conn = SqliteConnection::establish(":memory:").unwrap();
    let db = GlobalDb::new(SqliteGlobalDbAdapter::new(conn));
    db.adapter().migrate().unwrap();
    db
}

fn new_public_key() -> PublicKey {
    PublicKey::random_keypair(&mut OsRng).1
}

fn derived_substate_address(public_key: &PublicKey) -> SubstateAddress {
    let hash = FixedHash::try_from(public_key.as_bytes()).unwrap();
    SubstateAddress::from_hash_and_version(hash, 0)
}

fn insert_vns(
    validator_nodes: &mut ValidatorNodeDb<'_, '_, SqliteGlobalDbAdapter<PeerAddress>>,
    num: usize,
    epoch: Epoch,
) {
    for _ in 0..num {
        let pk = new_public_key();
        insert_vn_with_public_key(validator_nodes, pk.clone(), epoch);
        set_committee_shard_group(validator_nodes, &pk, ShardGroup::all_shards(NumPreshards::P256), epoch);
    }
}

fn insert_vn_with_public_key(
    validator_nodes: &mut ValidatorNodeDb<'_, '_, SqliteGlobalDbAdapter<PeerAddress>>,
    public_key: PublicKey,
    start_epoch: Epoch,
) {
    validator_nodes
        .insert_validator_node(
            public_key.clone().into(),
            public_key.clone(),
            derived_substate_address(&public_key),
            start_epoch,
            public_key,
        )
        .unwrap()
}

fn set_committee_shard_group(
    validator_nodes: &mut ValidatorNodeDb<'_, '_, SqliteGlobalDbAdapter<PeerAddress>>,
    public_key: &PublicKey,
    shard_group: ShardGroup,
    epoch: Epoch,
) {
    validator_nodes
        .set_committee_shard(derived_substate_address(public_key), shard_group, epoch)
        .unwrap();
}

#[test]
fn insert_and_get_within_epoch() {
    let db = create_db();
    let mut tx = db.create_transaction().unwrap();
    let mut validator_nodes = db.validator_nodes(&mut tx);
    insert_vns(&mut validator_nodes, 3, Epoch(0));
    insert_vns(&mut validator_nodes, 2, Epoch(1));
    let vns = validator_nodes.get_all_registered_within_start_epoch(Epoch(0)).unwrap();
    assert_eq!(vns.len(), 3);
}

#[test]
fn change_committee_shard_group() {
    let db = create_db();
    let mut tx = db.create_transaction().unwrap();
    let mut validator_nodes = db.validator_nodes(&mut tx);
    let pk = new_public_key();
    insert_vn_with_public_key(&mut validator_nodes, pk.clone(), Epoch(0));
    set_committee_shard_group(&mut validator_nodes, &pk, ShardGroup::new(1, 2), Epoch(0));
    let count = validator_nodes.count(Epoch(0)).unwrap();
    assert_eq!(count, 1);
    set_committee_shard_group(&mut validator_nodes, &pk, ShardGroup::new(3, 4), Epoch(1));
    set_committee_shard_group(&mut validator_nodes, &pk, ShardGroup::new(7, 8), Epoch(2));
    set_committee_shard_group(&mut validator_nodes, &pk, ShardGroup::new(4, 5), Epoch(3));
    let pk2 = new_public_key();
    insert_vn_with_public_key(&mut validator_nodes, pk2.clone(), Epoch(3));
    set_committee_shard_group(&mut validator_nodes, &pk2, ShardGroup::new(4, 5), Epoch(3));
    let count = validator_nodes.count(Epoch(0)).unwrap();
    assert_eq!(count, 1);
    let count = validator_nodes.count(Epoch(3)).unwrap();
    assert_eq!(count, 2);
    let vns = validator_nodes
        .get_committee_for_shard_group(Epoch(3), ShardGroup::new(4, 5), false, 100)
        .unwrap();
    assert_eq!(vns.len(), 2);
}
