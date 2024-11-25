use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zellij_tile::{prelude::PluginMessage, shim::post_message_to_plugin, ZellijWorker};

use crate::FavSessionInfo;

#[derive(Default, Serialize, Deserialize)]
pub struct FavSynchronizer {}

#[derive(Serialize, Deserialize)]
pub struct SyncMessage {
    pub favs: Vec<FavSessionInfo>,
    pub flush: Vec<FavSessionInfo>,
    pub sender_id: Uuid,
}

pub const FAV_SYNCHRONIZER_NAME: &str = "fav";
pub const FAV_SYNCHRONIZER_MESSAGE: &str = "Sync";

impl ZellijWorker<'_> for FavSynchronizer {
    fn on_message(&mut self, message: String, payload: String) {
        eprintln!("zellij-favs-worker-payload: {}", payload);

        post_message_to_plugin(PluginMessage {
            name: message,
            payload,
            worker_name: Some(FAV_SYNCHRONIZER_NAME.to_string()),
        });
    }
}
