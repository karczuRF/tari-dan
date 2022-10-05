//   Copyright 2022. The Tari Project
//
//   Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//   following conditions are met:
//
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//   disclaimer.
//
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//   following disclaimer in the documentation and/or other materials provided with the distribution.
//
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//   products derived from this software without specific prior written permission.
//
//   THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//   INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//   DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//   SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//   SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//   WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//   USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use log::*;
use tari_dan_common_types::ShardId;
use tari_dan_core::{
    message::DanMessage,
    models::{Payload, TariDanPayload, TreeNodeHash},
    services::infrastructure_services::OutboundService,
};
use tari_dan_engine::instruction::Transaction;
use tokio::sync::{broadcast, mpsc};

use crate::p2p::services::messaging::OutboundMessaging;

const LOG_TARGET: &str = "dan::mempool::service";

pub struct MempoolService {
    // TODO: Should be a HashSet
    transactions: Vec<(Transaction, Option<TreeNodeHash>)>,
    new_transactions: mpsc::Receiver<Transaction>,
    outbound: OutboundMessaging,
    tx_valid_transactions: broadcast::Sender<(Transaction, ShardId)>,
}

impl MempoolService {
    pub(super) fn new(
        new_transactions: mpsc::Receiver<Transaction>,
        outbound: OutboundMessaging,
        tx_valid_transactions: broadcast::Sender<(Transaction, ShardId)>,
    ) -> Self {
        Self {
            transactions: Vec::new(),
            new_transactions,
            outbound,
            tx_valid_transactions,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(transaction) = self.new_transactions.recv() => {
                    self.handle_new_transaction(transaction).await;
                }

                else => {
                    info!(target: LOG_TARGET, "Mempool service shutting down");
                    break;
                }
            }
        }
    }

    async fn handle_new_transaction(&mut self, transaction: Transaction) {
        // TODO: validate transaction
        let payload = TariDanPayload::new(transaction.clone());
        for shard_id in payload.involved_shards() {
            self.tx_valid_transactions
                .send((transaction.clone(), shard_id))
                // TODO: handle, if channel is closed I would say we can ignore it since we're probably shutting down
                .unwrap();
        }
        self.transactions.push((transaction.clone(), None));
        let msg = DanMessage::NewTransaction(transaction);
        if let Err(err) = self.outbound.flood(Default::default(), msg).await {
            error!(target: LOG_TARGET, "Failed to broadcast new transaction: {}", err);
        }
    }
}