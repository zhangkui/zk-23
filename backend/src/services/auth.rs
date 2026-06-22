use crate::{AppState, AppError, models::*};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use crate::models::user::JwtClaims;

pub async fn login(
    state: AppState,
    req: &user::LoginRequest,
) -> Result<user::LoginResponse, AppError> {
    info!("Login attempt for user: {}", req.username);

    let (user, password_hash) = state.clickhouse_client.get_user_by_username(&req.username).await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("用户已被禁用".to_string()));
    }

    let password_valid = bcrypt::verify(&req.password, &password_hash)
        .unwrap_or(false);

    if !password_valid {
        warn!("Invalid password for user: {}", req.username);
        return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
    }

    let now = Utc::now();
    let expires_at = now + Duration::hours(state.config.jwt.expires_hours as i64);

    let claims = JwtClaims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.to_string(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: state.config.jwt.issuer.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt.secret.as_bytes()),
    ).map_err(|e| AppError::Internal(format!("JWT encode error: {}", e)))?;

    let updated_user = user::User {
        last_login: Some(now),
        updated_at: now,
        ..user.clone()
    };

    let activity_log = user::UserActivityLog {
        id: Uuid::new_v4(),
        user_id: user.id,
        activity_type: user::ActivityType::Login,
        description: "用户登录".to_string(),
        ip_address: None,
        user_agent: None,
        success: true,
        details: None,
        timestamp: now,
    };

    info!("User {} logged in successfully", req.username);

    Ok(user::LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt.expires_hours * 3600,
        expires_at,
        user: updated_user,
    })
}

pub async fn verify_token(
    state: AppState,
    token: &str,
) -> Result<JwtClaims, AppError> {
    let claims = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(state.config.jwt.secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| AppError::Unauthorized(format!("无效的令牌: {}", e)))?;

    Ok(claims.claims)
}

pub async fn get_current_user(
    state: AppState,
    user_id: Uuid,
) -> Result<user::User, AppError> {
    let (user, _) = state.clickhouse_client.get_user_by_id(user_id).await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    Ok(user)
}

pub async fn change_password(
    state: AppState,
    user_id: Uuid,
    old_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    info!("Changing password for user: {}", user_id);

    let (user, password_hash) = state.clickhouse_client.get_user_by_id(user_id).await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    let password_valid = bcrypt::verify(old_password, &password_hash)
        .unwrap_or(false);

    if !password_valid {
        return Err(AppError::BadRequest("原密码错误".to_string()));
    }

    let new_password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)?;

    state.clickhouse_client.update_user_password(user_id, &new_password_hash).await?;

    info!("Password changed successfully for user: {}", user_id);

    Ok(())
}

pub async fn create_user(
    state: AppState,
    req: &user::CreateUserRequest,
    created_by: Uuid,
) -> Result<Uuid, AppError> {
    info!("Creating user: {} by {}", req.username, created_by);

    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;

    let user = user::User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        email: req.email.clone(),
        full_name: req.full_name.clone(),
        role: req.role.clone(),
        department: req.department.clone(),
        phone: req.phone.clone(),
        is_active: true,
        last_login: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state.clickhouse_client.insert_user(&user, &password_hash).await?;

    Ok(user.id)
}

pub async fn list_users(
    state: AppState,
    role: Option<user::UserRole>,
    is_active: Option<bool>,
    page: u32,
    page_size: u32,
) -> Result<Vec<user::User>, AppError> {
    info!("Listing users");
    Ok(vec![])
}

pub fn extract_user_id_from_claims(claims: &JwtClaims) -> Result<Uuid, AppError> {
    Ok(claims.sub)
}
