// models/org.rs — Organization data structures
//
// An organization is a workspace that users belong to.
// Think of it like a GitHub organization or a Slack workspace.
//
// Key concepts:
// - A user creates an org and becomes the OWNER
// - Other users can be invited to join as MEMBERS
// - Each member has ROLES which define what they can do

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Org — mirrors the organizations table in the database
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Org {
    pub id: Uuid,       // unique ID for this org
    pub name: String,   // display name e.g. "Acme Corp"
    pub slug: String,   // URL-friendly name e.g. "acme-corp"
    pub owner_id: Uuid, // the user who created this org
}

// CreateOrgRequest — what the client sends to POST /orgs
#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String, // e.g. "Acme Corp"
    pub slug: String, // e.g. "acme-corp" — must be unique
}

// OrgResponse — what we send back to the client
#[derive(Debug, Serialize)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub owner_id: Uuid,
}

// Convert Org into OrgResponse
impl From<Org> for OrgResponse {
    fn from(org: Org) -> Self {
        Self {
            id: org.id,
            name: org.name,
            slug: org.slug,
            owner_id: org.owner_id,
        }
    }
}

// Membership — mirrors the memberships table
// Represents a user belonging to an organization
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Membership {
    pub id: Uuid,
    pub user_id: Uuid, // which user
    pub org_id: Uuid,  // which organization
}
