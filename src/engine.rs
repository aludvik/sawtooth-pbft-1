/*
 * Copyright 2018 Bitwise IO, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

//! Entry point for the consensus algorithm, including the main event loop

use std::sync::mpsc::{Receiver, RecvTimeoutError};

use sawtooth_sdk::consensus::{engine::*, service::Service};

use node::PbftNode;

use config;
use timing;

use error::PbftError;
use state::PbftState;
use storage::{DiskStorage, Storage};

#[derive(Default)]
pub struct PbftEngine {}

impl PbftEngine {
    pub fn new() -> Self {
        PbftEngine {}
    }
}

impl Engine for PbftEngine {
    fn start(
        &mut self,
        updates: Receiver<Update>,
        mut service: Box<Service>,
        startup_state: StartupState,
    ) {
        let StartupState {
            chain_head,
            peers: _peers,
            local_peer_info,
        } = startup_state;

        // Load on-chain settings
        let config = config::load_pbft_config(chain_head.block_id, &mut *service);

        let node_id = config
            .peers
            .iter()
            .position(|ref id| id == &&local_peer_info.peer_id)
            .expect("This node is not in the peers list, which is necessary")
            as u64;

        let mut pbft_state =
            DiskStorage::new_with_default("test.json", || PbftState::new(node_id, &config))
                .unwrap();

        let mut working_ticker = timing::Ticker::new(config.block_duration);
        let mut backlog_ticker = timing::Ticker::new(config.message_timeout);

        let mut node = PbftNode::new(&config, service, pbft_state.read().is_primary());

        debug!("Starting state: {:#?}", **pbft_state.read());

        // Event loop. Keep going until we receive a shutdown message.
        loop {
            let incoming_message = updates.recv_timeout(config.message_timeout);
            let state = &mut **pbft_state.write();

            let res = match incoming_message {
                Ok(Update::BlockNew(block)) => node.on_block_new(block, state),
                Ok(Update::BlockValid(block_id)) => node.on_block_valid(block_id, state),
                Ok(Update::BlockInvalid(_)) => {
                    warn!("{}: BlockInvalid received, starting view change", state);
                    node.start_view_change(state)
                }
                Ok(Update::BlockCommit(block_id)) => node.on_block_commit(block_id, state),
                Ok(Update::PeerMessage(message, _sender_id)) => {
                    node.on_peer_message(&message, state)
                }
                Ok(Update::Shutdown) => break,
                Ok(Update::PeerConnected(_)) | Ok(Update::PeerDisconnected(_)) => {
                    error!("PBFT currently only supports static networks");
                    Ok(())
                }
                Err(RecvTimeoutError::Timeout) => Err(PbftError::Timeout),
                Err(RecvTimeoutError::Disconnected) => {
                    error!("Disconnected from validator");
                    break;
                }
            };
            handle_pbft_result(res);

            working_ticker.tick(|| {
                if let Err(e) = node.try_publish(state) {
                    error!("{}", e);
                }

                // Every so often, check to see if timeout has expired; initiate ViewChange if necessary
                if node.check_timeout_expired(state) {
                    handle_pbft_result(node.start_view_change(state));
                }
            });

            backlog_ticker.tick(|| {
                handle_pbft_result(node.retry_backlog(state));
            })
        }
    }

    fn version(&self) -> String {
        String::from(env!("CARGO_PKG_VERSION"))
    }

    fn name(&self) -> String {
        String::from(env!("CARGO_PKG_NAME"))
    }
}

fn handle_pbft_result(res: Result<(), PbftError>) {
    if let Err(e) = res {
        match e {
            PbftError::Timeout => (),
            PbftError::WrongNumMessages(_, _, _) | PbftError::NotReadyForMessage => trace!("{}", e),
            _ => error!("{}", e),
        }
    }
}
