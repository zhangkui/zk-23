use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, FromRequestParts},
    http::{request::Parts, StatusCode, Request, HeaderMap},
    middleware::Next,
    response::{Response, IntoResponse, Json},
    RequestPartsExt,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::user::JwtClaims;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<user::LoginRequest>,
) -> Result<Json<user::LoginResponse>, AppError> {
    let response = services::auth::login(state, &req).await?;
    Ok(Json(response))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    claims: JwtClaims,
) -> Result<Json<user::User>, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    let user = services::auth::get_current_user(state, user_id).await?;
    Ok(Json(user))
}

pub async fn change_password(
    State(state): State<AppState>,
    claims: JwtClaims,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    services::auth::change_password(state, user_id, &req.old_password, &req.new_password).await?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

impl FromRequestParts<AppState> for JwtClaims {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("缺少认证令牌".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("认证令牌格式错误".to_string()))?;

        let claims = services::auth::verify_token(state.clone(), token).await?;
        Ok(claims)
    }
}

pub async fn auth_middleware<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("缺少认证令牌".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("认证令牌格式错误".to_string()))?;

    let claims = services::auth::verify_token(state, token).await?;

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

pub async fn require_role(
    required_role: user::UserRole,
    claims: JwtClaims,
) -> Result<(), AppError> {
    let user_role = user::UserRole::from_str(&claims.role)
        .map_err(|_| AppError::Unauthorized("无效的用户角色".to_string()))?;

    if user_role < required_role {
        return Err(AppError::Forbidden("权限不足".to_string()));
    }

    Ok(())
}
