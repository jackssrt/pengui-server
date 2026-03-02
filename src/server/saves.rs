use std::{path::PathBuf, time::SystemTime};

use anyhow::Result;
use async_compression::tokio::{bufread::ZstdDecoder, write::ZstdEncoder};
use chrono::{DateTime, Utc};
use tokio::{
    fs::{File, remove_file},
    io::{AsyncBufRead, BufReader},
};

use crate::{
    player::ids::PlayerUuid,
    server::{config::Config, state::AppState},
};

const SAVES_DIRECTORY: &str = "saves";
fn get_player_save_data_path(state: &AppState, player_uuid: &PlayerUuid) -> PathBuf {
    PathBuf::from(SAVES_DIRECTORY)
        .join(&state.config.game_name)
        .join(format!("{player_uuid}.osd"))
}

pub async fn get_save_data(
    state: &AppState,
    player_uuid: &PlayerUuid,
) -> Result<ZstdDecoder<BufReader<File>>> {
    let path = get_player_save_data_path(state, player_uuid);
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let decoder = ZstdDecoder::new(reader);

    Ok(decoder)
}
pub async fn get_save_data_timestamp(
    state: &AppState,
    player_uuid: &PlayerUuid,
) -> Result<DateTime<Utc>> {
    let path = get_player_save_data_path(state, player_uuid);
    Ok(tokio::fs::metadata(path).await?.modified()?.into())
}

pub async fn create_game_save_data(
    state: &AppState,
    player_uuid: &PlayerUuid,
    mut reader: impl AsyncBufRead + Unpin,
) -> Result<()> {
    let path = get_player_save_data_path(state, player_uuid);
    let file = File::create(path).await?;
    let mut encoder = ZstdEncoder::new(file);
    tokio::io::copy_buf(&mut reader, &mut encoder).await?;

    Ok(())
}
pub async fn clear_game_save_data(state: &AppState, player_uuid: &PlayerUuid) -> Result<()> {
    let path = get_player_save_data_path(state, player_uuid);
    Ok(remove_file(path).await?)
}
