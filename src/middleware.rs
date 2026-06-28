use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::errors::AppError;
use crate::jwt::verify_token;
use crate::routes::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok());

    let auth_header = match auth_header {
        Some(header) => header,
        None => {
            return Err(AppError::Unauthorized(
                "Missing Authorization header".to_string(),
            ))
        }
    };

    let token = match auth_header.strip_prefix("Bearer ") {
        Some(token) => token,
        None => {
            return Err(AppError::Unauthorized(
                "Authorization header must start with 'Bearer '".to_string(),
            ))
        }
    };

    let claims = verify_token(token, &state.config.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    req.extensions_mut().insert(AuthUser {
        user_id,
        email: claims.email,
    });

    Ok(next.run(req).await)
}
