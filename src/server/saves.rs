use std::{path::PathBuf, time::SystemTime};

use anyhow::Result;
use async_compression::tokio::{bufread::ZstdDecoder, write::ZstdEncoder};
use tokio::{
    fs::{File, remove_file},
    io::{AsyncBufRead, BufReader},
};

use crate::server::config::Config;

const SAVES_DIRECTORY: &str = "saves";
fn get_player_save_data_path(config: &Config, player_uuid: &str) -> PathBuf {
    PathBuf::from(SAVES_DIRECTORY)
        .join(&config.game_name)
        .join(format!("{player_uuid}.osd"))
}

async fn get_save_data(config: &Config, player_uuid: &str) -> Result<ZstdDecoder<BufReader<File>>> {
    let path = get_player_save_data_path(config, player_uuid);
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let decoder = ZstdDecoder::new(reader);

    Ok(decoder)
}
async fn get_save_data_timestamp(config: &Config, player_uuid: &str) -> Result<SystemTime> {
    let path = get_player_save_data_path(config, player_uuid);
    Ok(tokio::fs::metadata(path).await?.modified()?)
}

async fn create_game_save_data(
    config: &Config,
    player_uuid: &str,
    mut reader: impl AsyncBufRead + Unpin,
) -> Result<()> {
    let path = get_player_save_data_path(config, player_uuid);
    let file = File::create(path).await?;
    let mut encoder = ZstdEncoder::new(file);
    tokio::io::copy_buf(&mut reader, &mut encoder).await?;

    Ok(())
}
async fn clear_game_save_data(config: &Config, player_uuid: &str) -> Result<()> {
    let path = get_player_save_data_path(config, player_uuid);
    Ok(remove_file(path).await?)
}
