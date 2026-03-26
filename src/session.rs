use std::time::Duration;

use crate::{server::state::AppState, session::client::packet::OutgoingPacket};

pub mod client;

pub fn init_session(state: &'static AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        let mut last_player_count = None;
        loop {
            interval.tick().await;
            let player_count = { state.players.players.lock().len() };

            // don't send the same player count twice
            if let Some(last_player_count) = last_player_count
                && last_player_count != player_count
            {
                state
                    .players
                    .broadcast_session_packet(OutgoingPacket::PlayerCount(player_count))
                    .await;
            }
            last_player_count = Some(player_count);
        }
    });
}
