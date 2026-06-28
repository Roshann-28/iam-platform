// jwt.rs — JSON Web Tokens
//
// A JWT is a token we give to the user after they log in.
// The user sends this token with every request to prove who they are.
//
// A JWT has 3 parts separated by dots:
// header.payload.signature
//
// header   — says what algorithm we used
// payload  — contains the user's data (id, email, expiry)
// signature — proves the token wasn't tampered with
//
// Only our server knows the secret, so only we can create valid tokens.
// If someone changes the payload, the signature won't match and we reject it.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

// Claims — the data stored inside the JWT token
// When we verify a token, we get these fields back
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   // "subject" — the user's ID (standard JWT field name)
    pub email: String, // the user's email
    pub exp: usize,    // "expiry" — unix timestamp when token expires
    pub iat: usize,    // "issued at" — unix timestamp when token was created
}

// create_token() — builds and signs a JWT for a user
// Called after successful login or register
pub fn create_token(user_id: Uuid, email: &str, secret: &str) -> Result<String, AppError> {
    // Get current time as unix timestamp (seconds since 1970)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    // Token expires in 24 hours (24 * 60 * 60 = 86400 seconds)
    let exp = now + 86400;

    // Build the claims (payload of the JWT)
    let claims = Claims {
        sub: user_id.to_string(), // store user ID as string
        email: email.to_string(),
        iat: now,
        exp,
    };

    // Sign the token with our secret key and return it
    // Header::default() uses HS256 algorithm
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

// verify_token() — checks if a token is valid and returns its claims
// Called in our auth middleware on every protected request
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(), // checks signature + expiry automatically
    )
    .map(|data| data.claims) // extract just the claims from the result
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))
}
