# IAM Service — Identity & Access Management API

A backend service built in **Rust** using **Axum**, **SQLx**, and **PostgreSQL**.
It handles user authentication, organizations, memberships, and role-based access control (RBAC).

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| Web framework | Axum |
| Database | PostgreSQL (via Docker) |
| ORM / Query | SQLx |
| Password hashing | Argon2 |
| Auth tokens | JWT (jsonwebtoken) |
| Runtime | Tokio (async) |

---

## Project Structure

```
src/
├── main.rs          — entry point, starts the server
├── config.rs        — app config (DB URL, JWT secret)
├── db.rs            — database connection pool
├── errors.rs        — custom error types → HTTP responses
├── jwt.rs           — create and verify JWT tokens
├── middleware.rs     — auth middleware (protects routes)
├── models/
│   ├── mod.rs       — registers model modules
│   ├── user.rs      — User, RegisterRequest, LoginRequest, AuthResponse
│   └── org.rs       — Org, Membership, CreateOrgRequest, OrgResponse
└── routes/
    ├── mod.rs       — router wiring (all routes defined here)
    ├── auth.rs      — register + login handlers
    └── org.rs       — org CRUD handlers
```

---

## Database Schema

### Migration 1 — Users
```sql
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email         VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name     VARCHAR(255) NOT NULL,
    is_active     BOOLEAN NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Migration 2 — Organizations + Memberships
```sql
CREATE TABLE organizations (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name       VARCHAR(255) NOT NULL,
    slug       VARCHAR(255) NOT NULL UNIQUE,
    owner_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE memberships (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, org_id)
);
```

### Migration 3 — Roles + Permissions (RBAC)
```sql
CREATE TABLE roles (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id      UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, name)
);

CREATE TABLE permissions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(255) NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE role_permissions (
    role_id       UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE member_roles (
    membership_id UUID NOT NULL REFERENCES memberships(id) ON DELETE CASCADE,
    role_id       UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (membership_id, role_id)
);
```

### Migration 4 — Sessions, API Keys, Audit Logs
```sql
CREATE TABLE sessions (...);
CREATE TABLE api_keys (...);
CREATE TABLE audit_logs (...);
```
> These tables are **created in the DB but not yet wired up** in Rust — see [What's Not Done Yet](#whats-not-done-yet).

---

## API Routes

### Public Routes
| Method | Path | Description |
|---|---|---|
| GET | `/health` | Health check |
| POST | `/auth/register` | Create a new account |
| POST | `/auth/login` | Login, receive JWT |

### Protected Routes (require `Authorization: Bearer <token>`)
| Method | Path | Description |
|---|---|---|
| GET | `/me` | Get logged-in user info |
| POST | `/orgs` | Create a new organization |
| GET | `/orgs` | List all orgs user belongs to |
| GET | `/orgs/:id` | Get a specific org (members only) |
| DELETE | `/orgs/:id` | Delete an org (owner only) |

---

## How Auth Works

1. User registers or logs in → server returns a **JWT access token**
2. Client stores the token and sends it on every request:
   ```
   Authorization: Bearer <token>
   ```
3. `auth_middleware` intercepts protected routes, verifies the token, and injects `AuthUser` (user_id + email) into the request
4. Handlers extract `AuthUser` via `Extension(auth_user)`

JWT payload (claims):
```json
{
  "sub": "<user_uuid>",
  "email": "user@example.com",
  "iat": 1234567890,
  "exp": 1234654290
}
```
Tokens expire after **24 hours**.

---

## Running Locally

### Prerequisites
- Rust (stable)
- Docker

### 1. Start Postgres
```bash
docker run --name iam-db \
  -e POSTGRES_PASSWORD=pass123 \
  -e POSTGRES_DB=iam_db \
  -p 5432:5432 \
  -d postgres
```

### 2. Run migrations
Connect to Postgres and run the 4 migration SQL files in order.

### 3. Start the server
```bash
cargo run
```

Server starts at `http://localhost:3000`

---

## Example Requests

### Register
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"you@example.com","password":"secret123","full_name":"Your Name"}'
```

### Login
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"you@example.com","password":"secret123"}'
```

### Create Org
```bash
curl -X POST http://localhost:3000/orgs \
  -H "Authorization: Bearer <your_token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"Acme Corp","slug":"acme-corp"}'
```

---

## Design Decisions

**Why Argon2 for passwords?**
Argon2 is the winner of the Password Hashing Competition (2015) and is resistant to GPU brute-force attacks. bcrypt would also work but Argon2 is the current best practice.

**Why JWT over sessions?**
JWTs are stateless — no DB lookup needed on every request. The tradeoff is you can't invalidate a token early (until the `sessions` table is wired up for refresh tokens).

**Why `owner_id` on the `organizations` table?**
Ownership is a single, clear fact. Deriving it from memberships would require a special "owner" role and a more complex query on every ownership check.

**Why return 404 instead of 403 for non-member org access?**
Security through obscurity — we don't want to reveal that an org exists to someone who isn't a member of it.

**Why `ON DELETE CASCADE` on memberships?**
Orphaned membership rows with no parent org or user are worse than silently cleaning up. Cascade keeps the DB consistent automatically.

---

## What's Not Done Yet

### Day 4 — Roles + Permissions (RBAC engine)
- [ ] `models/role.rs` — Role, Permission, RolePermission structs
- [ ] `routes/role.rs` — create role, assign permission, assign role to member
- [ ] `has_permission(user_id, org_id, permission)` helper function
- [ ] Permission checks inside org handlers (e.g. only members with `member:invite` can invite)

### Day 5 — Tests
- [ ] Integration tests for `/auth/register` and `/auth/login`
- [ ] Integration tests for org CRUD
- [ ] Permission check tests (assert 403 when permission missing)
- [ ] Duplicate slug / duplicate email conflict tests

### Day 6 — Not wired up yet
- [ ] `sessions` table — refresh token flow (login currently only issues access tokens)
- [ ] `api_keys` table — key generation, hashing, and verification
- [ ] `audit_logs` table — log every state-changing action (create org, delete member, etc.)
- [ ] Real environment variable loading (currently config is hardcoded in `config.rs`)
- [ ] `sqlx::migrate!` macro — replace manual SQL runs with tracked migrations
- [ ] Logging / tracing (currently just `println!`)
- [ ] Input validation on slug format (only allow `a-z`, `0-9`, `-`)
- [ ] Member invite endpoint (`POST /orgs/:id/members`)
- [ ] Member remove endpoint (`DELETE /orgs/:id/members/:user_id`)
- [ ] Token revocation (blacklist or short expiry + refresh)
