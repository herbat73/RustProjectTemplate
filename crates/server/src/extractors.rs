//! Extractors that authenticate a request from its `Authorization: Bearer` header. A handler
//! that takes one of them is only reachable with a valid token of the matching kind.

use application::AppError;
use application::ports::AccessClaims;
use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use crate::{error::ApiError, state::AppState};

async fn bearer_token(parts: &mut Parts) -> Result<String, ApiError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| ApiError(AppError::InvalidToken))?;
    Ok(bearer.token().to_string())
}

/// The caller, authenticated by a short-lived access token.
#[derive(Debug, Clone)]
pub struct AccessUser(pub AccessClaims);

impl FromRequestParts<AppState> for AccessUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).await?;
        Ok(Self(state.auth.verify_access_token(&token)?))
    }
}

/// The uuid of the caller, authenticated by a long-lived requester token.
#[derive(Debug, Clone)]
pub struct RequesterUser(pub String);

impl FromRequestParts<AppState> for RequesterUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).await?;
        Ok(Self(state.auth.verify_requester_token(&token)?))
    }
}
