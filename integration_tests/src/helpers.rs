//   Copyright 2023 The Tari Project
//   SPDX-License-Identifier: BSD-3-Clause

use std::{
    fmt::{Debug, Display},
    net::TcpListener,
    time::Duration,
};

use tari_dan_common_types::SubstateRequirement;
use tari_engine_types::substate::SubstateId;
use tokio::{io::AsyncWriteExt, task::JoinHandle};

use crate::TariWorld;

pub fn get_os_assigned_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

pub fn get_os_assigned_ports() -> (u16, u16) {
    (get_os_assigned_port(), get_os_assigned_port())
}
pub async fn wait_listener_on_local_port_os_thread<T, E: Debug>(
    handle: std::thread::JoinHandle<Result<T, E>>,
    port: u16,
) -> std::thread::JoinHandle<Result<T, E>> {
    let mut i = 0;
    while let Err(e) = tokio::net::TcpSocket::new_v4()
        .unwrap()
        .connect(([127u8, 0, 0, 1], port).into())
        .await
    {
        if handle.is_finished() {
            handle
                .join()
                .expect("Node exited panicked")
                .expect("Node exited unexpectedly");
            panic!("Node exited cleanly unexpectedly");
        }
        // println!("Waiting for base node to start listening on port {}. {}", port, e);
        if i >= 20 {
            // println!("Node failed to start listening on port {} within 10s", port);
            panic!(
                "Node failed to start listening on port {} within 20s (err: {})",
                port, e
            );
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        i += 1;
    }
    handle
}

pub async fn wait_listener_on_local_port<T, E: Debug>(
    handle: JoinHandle<Result<T, E>>,
    port: u16,
) -> JoinHandle<Result<T, E>> {
    let mut i = 0;
    loop {
        match tokio::net::TcpSocket::new_v4()
            .unwrap()
            .connect(([127u8, 0, 0, 1], port).into())
            .await
        {
            Ok(mut sock) => {
                sock.shutdown().await.unwrap();
                break;
            },
            Err(e) => {
                if handle.is_finished() {
                    match handle.await {
                        Ok(Ok(_)) => panic!("Node exited cleanly unexpectedly"),
                        Ok(Err(e)) => panic!("Node exited with error: {:?}", e),
                        Err(e) => {
                            let panic = e.into_panic();
                            panic!(
                                "Node panicked {:?}",
                                panic
                                    .downcast_ref::<&str>()
                                    .copied()
                                    .or_else(|| panic.downcast_ref::<String>().map(|s| s.as_str()))
                                    .unwrap()
                            );
                        },
                    }
                }
                // println!("Waiting for base node to start listening on port {}. {}", port, e);
                if i >= 20 {
                    // println!("Node failed to start listening on port {} within 10s", port);
                    panic!(
                        "Node failed to start listening on port {} within 20s (err: {})",
                        port, e
                    );
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                i += 1;
            },
        }
    }
    handle
}

pub async fn check_join_handle<E: Display>(
    name: &str,
    handle: tokio::task::JoinHandle<Result<(), E>>,
) -> tokio::task::JoinHandle<Result<(), E>> {
    if !handle.is_finished() {
        return handle;
    }

    match handle.await {
        Ok(Ok(_)) => {
            panic!("Node {} exited unexpectedly", name);
        },
        Ok(Err(e)) => {
            panic!("Node {} exited unexpectedly with error: {}", name, e);
        },
        Err(e) => {
            panic!("Node {} panicked: {:?}", name, e.try_into_panic());
        },
    }
}

pub fn get_address_from_output(world: &TariWorld, output_ref: String) -> &SubstateId {
    world
        .outputs
        .iter()
        .find_map(|(parent_name, outputs)| {
            outputs
                .iter()
                .find(|(child_name, _)| {
                    let fqn = format!("{}/{}", parent_name, child_name);
                    fqn == output_ref
                })
                .map(|(_, addr)| &addr.substate_id)
        })
        .unwrap_or_else(|| panic!("Output not found: {}", output_ref))
}

pub fn get_component_from_namespace(world: &TariWorld, fq_component_name: String) -> SubstateRequirement {
    let (input_group, component_name) = fq_component_name.split_once('/').unwrap_or_else(|| {
        panic!(
            "Component name must be in the format '{{group}}/components/{{template_name}}', got {}",
            fq_component_name
        )
    });

    world
        .outputs
        .get(input_group)
        .unwrap_or_else(|| panic!("No outputs found with name {}", input_group))
        .iter()
        .find(|(name, _)| **name == component_name)
        .map(|(_, data)| data.clone())
        .unwrap_or_else(|| panic!("No component named {}", component_name))
}
