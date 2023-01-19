use std::{sync::Arc, time::Duration};

use rspc::Type;
use sd_p2p::{Event, Manager};
use sd_sync::CRDTOperation;
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, time::sleep};
use tracing::info;
use uuid::Uuid;

use crate::{library::LibraryManager, node::NodeConfigManager};

use self::{
	peer_metadata::{OperatingSystem, PeerMetadata},
	proto::{Request, Response},
};

mod peer_metadata;
mod proto;

const SPACEDRIVE_APP_ID: &'static str = "spacedrive";

/// TODO
#[derive(Default, Debug, Clone, Serialize, Deserialize, Type)]
pub struct PeerBootstrapProgress {
	completed: u8, // u8 is plenty for a percentage
}

pub struct P2PManager;

impl P2PManager {
	pub async fn new(
		node_config: Arc<NodeConfigManager>,
		library_manager: Arc<LibraryManager>,
		mut p2p_rx: mpsc::Receiver<(Uuid, CRDTOperation)>,
	) -> Arc<Self> {
		let config = Arc::new(node_config.get().await); // TODO: Update this throughout the application lifecycle

		let this = Arc::new(Self {});

		let manager = Manager::new(
			SPACEDRIVE_APP_ID,
			&config
				.keypair
				.as_ref()
				.expect("Keypair not found. This should be unreachable code!"),
			move || async move {
				PeerMetadata {
					name: "123".to_string(), // config.name.clone(), // TODO
					operating_system: Some(OperatingSystem::get_os()),
					version: Some(env!("CARGO_PKG_VERSION").to_string()),
				}
			},
			|manager, event| async move {
				// TODO: Send all these events to frontend through rspc
				match event {
					Event::PeerDiscovered(event) => {
						println!(
							"Discovered peer by id '{}' with address '{:?}' and metadata: {:?}",
							event.peer_id(),
							event.addresses(),
							event.metadata()
						);

						// TODO: Tie this into Spacedrive
						event.dial(&manager).await;
					}
					event => println!("{:?}", event),
				}
			},
			// This closure it run to handle a single incoming request. It's return type is then sent back to the client.
			// TODO: Why can't it infer the second param here???
			{
				let library_manager = library_manager.clone();
				move |_manager, data: Vec<u8>| {
					let library_manager = library_manager.clone(); // This makes sure this function is `Fn` not `FnOnce`.
					async move {
						let req = rmp_serde::from_slice::<Request>(&data).unwrap();
						match req.handle(&library_manager).await.unwrap() {
							Response::None => Ok(vec![]),
							resp => Ok(rmp_serde::to_vec(&resp).unwrap()),
						}
					}
				}
			},
		)
		.await
		.unwrap();

		tokio::spawn({
			let manager = manager.clone();
			async move {
				while let Some(op) = p2p_rx.recv().await {
					// TODO: Only seen to peers in the current library and deal with library signing here.
					// TODO: Put protocol above broadcast feature.
					manager
						.broadcast(rmp_serde::to_vec_named(&op).unwrap())
						.await
						.unwrap();
				}
			}
		});

		tokio::spawn(async move {
			sleep(Duration::from_millis(500)).await;
			info!(
				"Node '{}' is now online listening at addresses: {:?}",
				manager.peer_id(),
				manager.listen_addrs().await
			);

			// TODO: Remove this without the connections timing out????
			loop {
				sleep(Duration::from_secs(3)).await;
				manager
					.clone()
					.broadcast(rmp_serde::to_vec(&Request::Ping).unwrap())
					.await
					.unwrap();
				// println!("Sent broadcast!");
			}
		});

		// TODO: proper shutdown
		// https://docs.rs/ctrlc/latest/ctrlc/
		// https://docs.rs/system_shutdown/latest/system_shutdown/

		this
	}
}