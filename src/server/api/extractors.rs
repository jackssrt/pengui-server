use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Query},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::Deserialize;

use crate::server::error::AppError;

pub struct OptionalAuthorizationToken(pub Option<String>);
impl<S> FromRequestParts<S> for OptionalAuthorizationToken
where
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let header = parts.extract::<TypedHeader<Authorization<Bearer>>>().await;
        match header {
            Ok(header) => Ok(Self(Some(header.token().to_owned()))),
            Err(rejection) if rejection.is_missing() => Ok(Self(None)),
            Err(rejection) => Err(rejection.into()),
        }
    }
}
pub struct AuthorizationToken(pub String);
impl<S> FromRequestParts<S> for AuthorizationToken
where
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let header = parts.extract::<TypedHeader<Authorization<Bearer>>>().await;
        Ok(Self(header?.token().to_owned()))
    }
}
#[derive(Deserialize)]
struct QueryWithToken {
    pub token: String,
}

pub struct QueryToken(pub String);
impl<S> FromRequestParts<S> for QueryToken
where
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = parts.extract::<Query<QueryWithToken>>().await?;
        let token = query.token.clone();
        Ok(Self(token))
    }
}
#[derive(Deserialize)]
struct OptionalQueryWithToken {
    pub token: Option<String>,
}

pub struct OptionalQueryToken(pub Option<String>);
impl<S> FromRequestParts<S> for OptionalQueryToken
where
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = parts.extract::<Query<OptionalQueryWithToken>>().await?;
        let token = query.token.clone();
        Ok(Self(token))
    }
}
