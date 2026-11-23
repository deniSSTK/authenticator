# Authenticator

Authenticator is a B2B2C two-factor authentication (2FA) project based on RFC 6238. The planned web client and B2B API let third-party services connect user accounts through one-time links and verify time-based one-time passwords (TOTP).

## Project status as of November 23, 2026

- **Backend** - The Axum server reads configuration from environment variables, opens a PostgreSQL connection pool at startup, and serves `GET /healthz`. The health endpoint returns `{"status":"ok"}` as a liveness check.
- **Authentication** - The `register`, `login`, `refresh`, and `me` routes are wired with typed response DTOs and shared JSON errors. They return `501 Not Implemented` after any initial input checks. An SQL insert for users exists, and the registration service has draft types. Password hashing, token issuance, and session management are pending.
- **Database** - Docker Compose configures PostgreSQL 18. The initial migration defines `users`, `user_sessions`, `applications`, `api_keys`, and `totp_enrollments`. The `connect_sessions` table and `applications.webhook_secret` are planned.
- **Integrations and frontend** - TOTP verification, enrollment, webhooks, and the Angular client are planned.

## Technology stack

- **Backend** - Rust (2024 edition), Axum 0.8, and Tokio.
- **Database** - PostgreSQL 18 and SQLx 0.9.1.
- **Cryptography** - Argon2id for password hashing, AES-256-GCM for envelope encryption, HMAC-SHA256 for webhook signatures, and `totp-rs` for TOTP are planned.
- **Frontend** - An Angular single-page application is planned.

## Repository structure

```text
authenticator/
├── .env.example              # Local PostgreSQL settings for Docker Compose
├── docker-compose.yml        # Local PostgreSQL service
├── backend/
│   ├── .env.example          # Backend environment settings
│   ├── Cargo.toml
│   ├── migrations/           # PostgreSQL schema migrations
│   └── src/
│       ├── api.rs             # Shared HTTP response and error exports
│       ├── api/               # JSON errors and response helpers
│       ├── auth.rs            # Authentication module and router export
│       ├── auth/
│       │   ├── http.rs        # HTTP module declarations
│       │   ├── http/          # Auth routes and request and response DTOs
│       │   ├── repo.rs        # SQL queries for authentication
│       │   └── service.rs     # Draft registration service types
│       ├── config.rs         # Environment configuration
│       ├── state.rs          # Shared PostgreSQL pool
│       └── main.rs           # HTTP server entry point
└── README.md
```

## Planned architecture and security

1. **User plane (`/v1/auth/*`, `/v1/applications/*`, `/v1/vault/*`)** - The Angular client uses short-lived JWT bearer access tokens and rotating refresh tokens.
2. **B2B runtime plane (`/v1/totp/*`)** - Third-party services authenticate server-to-server requests with `X-API-Key`. The API checks the key's scopes for each operation.

### Envelope encryption

- **Key encryption key (KEK)** - A 32-byte master key is loaded from the environment at startup and held in process memory.
- **Data encryption key (DEK)** - Each TOTP enrollment gets its own 32-byte key. AES-256-GCM encrypts the TOTP secret with the DEK, and the KEK encrypts the DEK.
- **Additional authenticated data (AAD)** - The user ID and enrollment ID bind the ciphertext to its enrollment record.

### TOTP replay protection

TOTP codes use a 30-second time step by default. After verifying a code, the backend uses an atomic PostgreSQL update to accept the matching step only if it is newer than `last_verified_step` for that enrollment.

```sql
UPDATE totp_enrollments
SET last_verified_step = $1, updated_at = NOW()
WHERE id = $2
  AND status = 1
  AND deleted_at IS NULL
  AND (last_verified_step IS NULL OR last_verified_step < $1)
RETURNING id;
```

## Planned redirect connect flow

The third-party service identifies the user by its own `external_user_id`. The authenticator can link the account without sharing the user's authenticator email or login.

1. The third-party backend calls `POST /v1/totp/connect-sessions` with an `X-API-Key`, the `external_user_id`, a display label in `account_label`, and a `redirect_url`.
2. The service redirects the user's browser to the returned one-time link, `https://<frontend_url>/connect/{connect_id}?token={connect_token}`.
3. The Angular client fetches the session with `GET /v1/vault/connect-sessions/{connect_id}?token=...`. When the user approves, it calls `POST /v1/vault/connect-sessions/{connect_id}/approve` with `{"token":"..."}`.
4. In one transaction, the backend consumes the session, generates and encrypts the TOTP secret, and creates an active enrollment. The backend sends the third-party service a `totp.connected` webhook signed with `X-Signature-HMAC-SHA256` and returns the user to `redirect_url`.
5. On subsequent sign-ins, the third-party service checks the six-digit code with `POST /v1/totp/verify`.

## Database schema

- **`users`** - Authenticator accounts (`id`, `email`, `password_hash`, `status`).
- **`user_sessions`** - Client sessions (`id`, `user_id`, `refresh_token_hash`, `device_name`, `ip_address`, `expires_at`).
- **`applications`** - Registered third-party services (`id`, `owner_user_id`, `issuer_name`, `webhook_url`, `is_active`). The `webhook_secret` field is planned.
- **`api_keys`** - Scoped B2B API keys (`id`, `application_id`, `key_prefix`, `key_hash`, `scopes`, `expires_at`, `revoked_at`, `last_used_at`).
- **`connect_sessions` (planned)** - One-time enrollment sessions (`id`, `application_id`, `external_user_id`, `account_label`, `token_hash`, `redirect_url`, `expires_at`, `consumed_at`).
- **`totp_enrollments`** - Linked accounts and encrypted TOTP secrets (`id`, `user_id`, `application_id`, `external_user_id`, `issuer`, `account_label`, `encrypted_secret`, `secret_nonce`, `encrypted_dek`, `dek_nonce`, `key_version`, `algorithm`, `digits`, `period`, `status`, `last_verified_step`, `revision`, `deleted_at`). Both `application_id` and `external_user_id` are required. Status `1` means active and `2` means disabled.

## API contract

### System (public)

- [x] `GET /healthz` - Check whether the HTTP server is responding.

### Authentication (public, refresh token, or user JWT)

- [ ] `POST /v1/auth/register` - Register a new user.
- [ ] `POST /v1/auth/login` - Sign in and receive an access token and a refresh token.
- [ ] `POST /v1/auth/refresh` - Rotate tokens using a refresh token.
- [ ] `POST /v1/auth/logout` - End the current session.
- [ ] `GET /v1/auth/me` - Get the authenticated user's profile with a bearer JWT.

### Applications (user JWT)

- [ ] `POST /v1/applications` - Create a B2B application.
- [ ] `GET /v1/applications` - List the user's applications.
- [ ] `PATCH /v1/applications/{app_id}` - Update `issuer_name`, `webhook_url`, or `is_active`.
- [ ] `DELETE /v1/applications/{app_id}` - Delete an application.
- [ ] `POST /v1/applications/{app_id}/webhook-secret/rotate` - Rotate the webhook signing secret.
- [ ] `POST /v1/applications/{app_id}/api-keys` - Create an API key with a list of scopes.
- [ ] `GET /v1/applications/{app_id}/api-keys` - List API key metadata (`id`, `key_prefix`, `scopes`, `last_used_at`).
- [ ] `DELETE /v1/applications/{app_id}/api-keys/{key_id}` - Revoke an API key by setting `revoked_at`.

### Vault (user JWT)

- [ ] `GET /v1/vault/connect-sessions/{connect_id}?token=...` - Get the enrollment request for the approval screen.
- [ ] `POST /v1/vault/connect-sessions/{connect_id}/approve` - Approve the enrollment with `{"token":"..."}`.
- [ ] `GET /v1/vault/items` - List linked services and their current TOTP codes.
- [ ] `DELETE /v1/vault/items/{item_id}` - Unlink a service and send a `totp.disconnected` webhook.

### B2B TOTP runtime (`X-API-Key`)

- [ ] `POST /v1/totp/connect-sessions` - Create a one-time enrollment link for an `external_user_id`.
- [ ] `POST /v1/totp/verify` - Verify a six-digit code for an `external_user_id`.
- [ ] `GET /v1/totp/enrollments/{external_user_id}` - Check a user's enrollment status.
- [ ] `DELETE /v1/totp/enrollments/{external_user_id}` - Disable 2FA for the user in the application.

## Local development

Use Rust and Cargo, Docker Compose, and the SQLx CLI with PostgreSQL support. From the repository root, create the local environment files and start PostgreSQL. If the files already exist, keep your local settings and make sure the credentials and port match `DATABASE_URL`.

```bash
test -f .env || cp .env.example .env
test -f backend/.env || cp backend/.env.example backend/.env
docker compose up -d --wait postgres
```

From `backend/`, export the backend settings, apply the migrations, and start the server.

```bash
cd backend
set -a
. ./.env
set +a
sqlx migrate run
cargo run
```

With the default bind address, check the server from another terminal.

```bash
curl http://127.0.0.1:3000/healthz
```
