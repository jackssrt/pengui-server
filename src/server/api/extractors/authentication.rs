use anyhow::Error;
use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Query},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_client_ip::RightmostXForwardedFor;
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::Deserialize;
use strum::EnumIs;

use crate::{
    player::ids::PlayerUuid,
    server::{error::AppError, state::AppState},
};

pub struct Authentication {
    pub token: String,
    pub uuid: PlayerUuid,
}
#[derive(EnumIs)]
pub enum OptionalAuthentication {
    Authenticated(Authentication),
    Guest(PlayerUuid),
}
impl OptionalAuthentication {
    pub fn take_uuid(self) -> PlayerUuid {
        let (Self::Authenticated(Authentication { uuid, .. }) | Self::Guest(uuid)) = self;
        uuid
    }
}

type AuthenticationRejection = (StatusCode, Response);

fn anyhow_rejection(error: Error) -> AuthenticationRejection {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        AppError::from(error).into_response(),
    )
}
fn err_rejection(
    error: impl std::error::Error + Send + Sync + IntoResponse,
) -> AuthenticationRejection {
    (StatusCode::INTERNAL_SERVER_ERROR, error.into_response())
}
fn invalid_token_rejection() -> AuthenticationRejection {
    (StatusCode::UNAUTHORIZED, "invalid token".into_response())
}
fn token_not_specified_rejection() -> AuthenticationRejection {
    (
        StatusCode::UNAUTHORIZED,
        "token not specified".into_response(),
    )
}

pub struct HeaderAuthentication(pub Authentication);
impl FromRequestParts<&'static AppState> for HeaderAuthentication {
    type Rejection = AuthenticationRejection;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &&'static AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(header) = parts
            .extract::<Option<TypedHeader<Authorization<Bearer>>>>()
            .await
            .map_err(err_rejection)?
        else {
            return Err(token_not_specified_rejection());
        };
        let token = header.token();
        let uuid = PlayerUuid::fetch_for_token(state, token)
            .await
            .map_err(anyhow_rejection)?
            .ok_or_else(invalid_token_rejection)?;
        Ok(Self(Authentication {
            token: token.to_owned(),
            uuid,
        }))
    }
}
pub struct OptionalHeaderAuthentication(pub OptionalAuthentication);
impl FromRequestParts<&'static AppState> for OptionalHeaderAuthentication {
    type Rejection = AuthenticationRejection;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &&'static AppState,
    ) -> Result<Self, Self::Rejection> {
        let RightmostXForwardedFor(ip) = parts
            .extract::<RightmostXForwardedFor>()
            .await
            .map_err(err_rejection)?;
        let Some(header) = parts
            .extract::<Option<TypedHeader<Authorization<Bearer>>>>()
            .await
            .map_err(err_rejection)?
        else {
            return Ok(Self(OptionalAuthentication::Guest(
                PlayerUuid::fetch_for_ip(state, &ip)
                    .await
                    .map_err(anyhow_rejection)?,
            )));
        };

        let token = header.token();
        let uuid = PlayerUuid::fetch_for_token(state, token)
            .await
            .map_err(anyhow_rejection)?
            .ok_or_else(invalid_token_rejection)?;
        Ok(Self(OptionalAuthentication::Authenticated(
            Authentication {
                token: token.to_owned(),
                uuid,
            },
        )))
    }
}

#[derive(Deserialize)]
struct TokenQuery {
    pub token: Option<String>,
}

pub struct QueryAuthentication(pub Authentication);
impl FromRequestParts<&'static AppState> for QueryAuthentication {
    type Rejection = AuthenticationRejection;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &&'static AppState,
    ) -> Result<Self, Self::Rejection> {
        let query = parts
            .extract::<Query<TokenQuery>>()
            .await
            .map_err(err_rejection)?;
        let token = query.0.token.ok_or_else(token_not_specified_rejection)?;
        let uuid = PlayerUuid::fetch_for_token(state, &token)
            .await
            .map_err(anyhow_rejection)?
            .ok_or_else(invalid_token_rejection)?;

        Ok(Self(Authentication { token, uuid }))
    }
}

pub struct OptionalQueryAuthentication(pub OptionalAuthentication);
impl FromRequestParts<&'static AppState> for OptionalQueryAuthentication {
    type Rejection = AuthenticationRejection;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &&'static AppState,
    ) -> Result<Self, Self::Rejection> {
        let RightmostXForwardedFor(ip) = parts
            .extract::<RightmostXForwardedFor>()
            .await
            .map_err(err_rejection)?;
        let query = parts
            .extract::<Query<TokenQuery>>()
            .await
            .map_err(err_rejection)?;

        let Some(token) = query.0.token else {
            return Ok(Self(OptionalAuthentication::Guest(
                PlayerUuid::fetch_for_ip(state, &ip)
                    .await
                    .map_err(anyhow_rejection)?,
            )));
        };

        let uuid = PlayerUuid::fetch_for_token(state, &token)
            .await
            .map_err(anyhow_rejection)?
            .ok_or_else(invalid_token_rejection)?;

        Ok(Self(OptionalAuthentication::Authenticated(
            Authentication { token, uuid },
        )))
    }
}
