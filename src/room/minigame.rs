use anyhow::Result;
use serde::Serialize;

use crate::server::state::{config::Config, database::Database};

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Minigame {
    pub id: &'static str,
    pub var_id: u16,
    pub initial_var_sync: bool,
    pub switch_id: u16,
    pub switch_value: bool,
    pub dev: bool,
}
fn get_room_minigame(config: &Config, room_id: u16) -> Option<Minigame> {
    match (&*config.game_name, room_id) {
        ("yume", 155) => Some(Minigame {
            id: "nasu",
            var_id: 88,
            switch_id: 215,
            ..Default::default()
        }),
        ("2kki", 102) => Some(Minigame {
            id: "rby",
            var_id: 1010,
            initial_var_sync: true,
            ..Default::default()
        }),
        ("2kki", 618) => Some(Minigame {
            id: "rby_ex",
            var_id: 79,
            initial_var_sync: true,
            ..Default::default()
        }),
        ("2kki", 344) => Some(Minigame {
            id: "fuji_ex",
            var_id: 3218,
            switch_id: 3219,
            switch_value: true,
            ..Default::default()
        }),
        ("2kki", 1899) => Some(Minigame {
            id: "hozo",
            var_id: 4268,
            switch_id: 5019,
            switch_value: true,
            ..Default::default()
        }),
        ("mikan", 6) => Some(Minigame {
            id: "ta_be",
            var_id: 17,
            switch_id: 14,
            switch_value: true,
            ..Default::default()
        }),
        ("mikan", 86) => Some(Minigame {
            id: "ta_be_hardcore",
            var_id: 17,
            switch_id: 14,
            switch_value: true,
            ..Default::default()
        }),
        ("ultraviolet", 118) => Some(Minigame {
            id: "panerabbit",
            var_id: 152,
            switch_id: 302,
            switch_value: true,
            ..Default::default()
        }),
        _ => None,
    }
}

async fn get_player_minigame_score(
    database: &Database,
    player_uuid: &str,
    minigame_id: &str,
) -> Option<i32> {
    sqlx::query_scalar!(
        "SELECT score FROM playerMinigameScores WHERE uuid = ? AND minigameId = ?",
        player_uuid,
        minigame_id
    )
    .fetch_one(&database.pool)
    .await
    .ok()
}
async fn try_write_player_minigame_score(
    config: &Config,
    database: &Database,
    player_uuid: &str,
    minigame_id: &str,
    score: i32,
) -> Result<bool> {
    if score == 0 {
        return Ok(false);
    }
    let previous_score = get_player_minigame_score(database, player_uuid, minigame_id).await;
    if let Some(previous_score) = previous_score {
        let is_highscore = previous_score < score;
        if !is_highscore {
            return Ok(false);
        }
        sqlx::query!(
            "UPDATE playerMinigameScores SET score = ?, timestampCompleted = CURRENT_TIMESTAMP() WHERE uuid = ? AND game = ? AND minigameId = ?",
            score,
            player_uuid,
            &config.game_name,
            minigame_id,
        ).execute(&database.pool).await?;
    } else {
        sqlx::query!("INSERT INTO playerMinigameScores (uuid, game, minigameId, score, timestampCompleted) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP())", player_uuid,&config.game_name, minigame_id, score ).execute(&database.pool).await?;
    }

    Ok(true)
}
