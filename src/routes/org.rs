// routes/org.rs — Organization handlers
//
// These handlers answer the question:
// "Which organization does the user belong to?"
//
// Routes:
// POST /orgs          — create a new organization
// GET  /orgs          — list all orgs the logged-in user belongs to
// GET  /orgs/:id      — get a specific org by ID
// DELETE /orgs/:id    — delete an org (only the owner can do this)

use axum::{
    extract::{Extension, Path, State},
    Json,
};
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::models::org::{CreateOrgRequest, Membership, Org, OrgResponse};
use crate::routes::AppState;

// create_org — POST /orgs
//
// Creates a new organization and automatically:
// 1. Sets the logged-in user as the owner
// 2. Creates a membership so the owner is also a member
//
// Extension(auth_user) — gets the logged-in user from middleware
// State(state) — gets the database pool and config
// Json(payload) — gets the request body
pub async fn create_org(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrgRequest>,
) -> Result<Json<OrgResponse>, AppError> {
    // Step 1: Validate input
    let name = payload.name.trim().to_string();
    let slug = payload.slug.trim().to_lowercase();

    if name.is_empty() || slug.is_empty() {
        return Err(AppError::BadRequest(
            "Name and slug are required".to_string(),
        ));
    }

    // Step 2: Check if slug is already taken
    // Slug must be unique across all organizations
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organizations WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&state.pool)
        .await?;

    if count > 0 {
        return Err(AppError::Conflict("Slug already taken".to_string()));
    }

    // Step 3: Insert the organization into the database
    // owner_id is the logged-in user's ID from the JWT token
    let org = sqlx::query_as::<_, Org>(
        "INSERT INTO organizations (name, slug, owner_id)
         VALUES ($1, $2, $3)
         RETURNING *",
    )
    .bind(&name)
    .bind(&slug)
    .bind(auth_user.user_id) // the creator becomes the owner
    .fetch_one(&state.pool)
    .await?;

    // Step 4: Create a membership for the owner
    // The owner is automatically a member of their own org
    sqlx::query(
        "INSERT INTO memberships (user_id, org_id)
         VALUES ($1, $2)",
    )
    .bind(auth_user.user_id)
    .bind(org.id)
    .execute(&state.pool)
    .await?;

    // Step 5: Return the created org
    Ok(Json(OrgResponse::from(org)))
}

// list_orgs — GET /orgs
//
// Returns all organizations the logged-in user belongs to
// A user can belong to multiple orgs (as owner or member)
pub async fn list_orgs(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<OrgResponse>>, AppError> {
    // Join organizations with memberships to find orgs this user belongs to
    // This answers: "which organizations does this user belong to?"
    let orgs = sqlx::query_as::<_, Org>(
        "SELECT o.* FROM organizations o
         INNER JOIN memberships m ON m.org_id = o.id
         WHERE m.user_id = $1",
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.pool)
    .await?;

    // Convert each Org into OrgResponse and collect into a Vec
    let response = orgs.into_iter().map(OrgResponse::from).collect();

    Ok(Json(response))
}

// get_org — GET /orgs/:id
//
// Returns a specific organization by ID
// The user must be a member of the org to view it
pub async fn get_org(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>, // extracts the :id from the URL
) -> Result<Json<OrgResponse>, AppError> {
    // Check if the user is a member of this org
    // This enforces that users can only see orgs they belong to
    let is_member = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM memberships
         WHERE user_id = $1 AND org_id = $2",
    )
    .bind(auth_user.user_id)
    .bind(org_id)
    .fetch_one(&state.pool)
    .await?;

    if is_member == 0 {
        // Return NotFound instead of Forbidden
        // We don't want to reveal that the org exists to non-members
        return Err(AppError::NotFound("Organization not found".to_string()));
    }

    // Fetch the organization
    let org = sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

    Ok(Json(OrgResponse::from(org)))
}

// delete_org — DELETE /orgs/:id
//
// Deletes an organization
// ONLY the owner can delete an org
pub async fn delete_org(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check if the logged-in user is the OWNER of this org
    // Only owners can delete organizations
    let org = sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

    // Compare owner_id with the logged-in user's ID
    if org.owner_id != auth_user.user_id {
        return Err(AppError::Unauthorized(
            "Only the owner can delete this organization".to_string(),
        ));
    }

    // Delete the org — memberships are deleted automatically
    // because we used ON DELETE CASCADE in the migration
    sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(org_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Organization deleted successfully"
    })))
}
