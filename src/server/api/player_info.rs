use axum::{Json, extract::State};
use axum_client_ip::RightmostXForwardedFor;
use serde::Serialize;

use crate::{
    locations::Locations,
    player::{
        badge::BadgeName, badge_slots::BadgeSlots, ids::PlayerUuid, medal::Medals,
        name::PlayerName, rank::Rank, screenshot_limit::ScreenshotLimit,
    },
    server::{
        api::extractors::authentication::OptionalHeaderAuthentication, error::AppError,
        state::AppState,
    },
    traits::{FetchForPlayerUuid, MaybeFetchForPlayerUuid},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfo {
    uuid: PlayerUuid,
    registered: bool,
    name: PlayerName,
    rank: Rank,
    badge: BadgeName,
    #[serde(flatten)]
    badge_slots: BadgeSlots,
    screenshot_limit: ScreenshotLimit,
    medals: Medals,
    location_ids: Option<Locations>,
}
async fn helper(state: &AppState, token: Option<&str>) {}

#[axum::debug_handler]
pub async fn handle_player_info(
    State(state): State<&'static AppState>,
    OptionalHeaderAuthentication(auth): OptionalHeaderAuthentication,
    RightmostXForwardedFor(ip): RightmostXForwardedFor,
) -> Result<Json<PlayerInfo>, AppError> {
    let is_authenticated = auth.is_authenticated();
    let uuid = auth.take_uuid();
    Ok(PlayerInfo {
        name: PlayerName::fetch_for_player_uuid(state, &uuid)
            .await?
            .unwrap_or_default(),
        rank: Rank::fetch_for_player_uuid(state, &uuid).await?,
        badge: BadgeName::fetch_for_player_uuid(state, &uuid)
            .await?
            .unwrap_or_default(),
        location_ids: {
            let locations = Locations::fetch_for_player_uuid(state, &uuid).await?;
            (!locations.0.is_empty()).then_some(locations)
        },
        medals: Medals::fetch_for_player_uuid(state, &uuid).await?,
        badge_slots: BadgeSlots::fetch_for_player_uuid(state, &uuid).await?,
        screenshot_limit: ScreenshotLimit::fetch_for_player_uuid(state, &uuid).await?,
        registered: is_authenticated,
        uuid,
    }
    .into())
}
