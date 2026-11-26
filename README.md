# Authenticator

Authenticator is a B2B2C two-factor authentication (2FA) project based on RFC 6238. The planned web client and B2B API let third-party services connect user accounts through one-time links and verify time-based one-time passwords (TOTP).

## Project status

- **Backend** - The Axum server reads configuration from environment variables, opens a PostgreSQL connection pool at startup, and serves `GET /healthz`. The health endpoint returns `{"status":"ok"}` as a liveness check.
- **Authentication** - The `register`, `login`, `refresh`, and `me` routes are wired with typed response DTOs and shared JSON errors. They return `501 Not Implemented` after any initial input checks. An SQL insert for users exists, and the registration service has draft types. Password hashing, token issuance, and session management are pending.
- **Database** - Docker Compose configures PostgreSQL 18. The initial migration defines `users`, `user_sessions`, `applications`, `api_keys`, and `totp_enrollments`. The `connect_sessions` table and `applications.webhook_secret` are planned.
- **Frontend** - `frontend/` is a placeholder for the planned Angular client. The DTOs below are intended for frontend mocks.
- **Integration demo** - `integration-demo/` is a Git submodule for a planned example application. Its pinned revision currently contains a license only.
- **Integrations** - TOTP verification, enrollment, and webhooks are planned.

## Technology stack

- **Backend** - Rust (2024 edition), Axum 0.8, and Tokio.
- **Database** - PostgreSQL 18 and SQLx 0.9.1.
- **Cryptography** - Argon2id for password hashing, AES-256-GCM for envelope encryption, HMAC-SHA256 for webhook signatures, and `totp-rs` for TOTP are planned.
- **Frontend** - Angular is planned.

## Planned frontend

1. **Application dashboard** - Manage the applications you own, their settings, and their API keys.
2. **Personal code vault** - View your own current TOTP codes for linked services.
3. **Authentication** - Create an account and sign in to access the dashboard and your codes.

## Repository structure

```text
authenticator/
├── .env.example              # Docker Compose database settings
├── .gitignore
├── .gitmodules               # Integration demo submodule configuration
├── docker-compose.yml        # Local PostgreSQL service
├── LICENSE
├── README.md
├── backend/
│   ├── .env.example          # Backend environment settings
│   ├── Cargo.toml
│   ├── Cargo.lock
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
│       ├── config.rs          # Environment configuration
│       ├── state.rs           # Shared PostgreSQL pool
│       └── main.rs            # HTTP server entry point
├── frontend/
│   └── .gitkeep              # Placeholder for the Angular client
└── integration-demo/         # Git submodule for an example integration
```

Initialize the integration demo after cloning, or clone with `git clone --recurse-submodules <repository-url>`.

```bash
git submodule update --init --recursive
```

The demo is maintained in [authenticator-integration-demo](https://github.com/deniSSTK/authenticator-integration-demo). Its implementation and startup instructions will live in that repository.

## Planned architecture and security

The following design describes planned behavior. Authentication, encryption, TOTP, and webhook delivery are not yet implemented.

1. **User plane (`/v1/auth/*`, `/v1/applications/*`, `/v1/vault/*`)** - The browser sends a short-lived JWT access token and a rotating refresh token in `HttpOnly` cookies scoped to their respective routes.
2. **B2B runtime plane (`/v1/totp/*`)** - Third-party services authenticate server-to-server requests with `X-API-Key`. The API checks the key's scopes for each operation.

### Envelope encryption

- **Key encryption key (KEK)** - A 32-byte master key is loaded from the environment at startup and held in process memory.
- **Data encryption key (DEK)** - Each TOTP enrollment gets its own 32-byte key. AES-256-GCM encrypts the TOTP secret with the DEK, and the KEK encrypts the DEK.
- **Additional authenticated data (AAD)** - The user ID and enrollment ID bind the ciphertext to its enrollment record.

### TOTP replay protection

TOTP codes will use a 30-second time step by default. After verifying a code, the backend can use an atomic PostgreSQL update to accept the matching step only if it is newer than `last_verified_step` for that enrollment.

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
4. In one transaction, the backend consumes the session, generates and encrypts the TOTP secret, and creates an active enrollment. The backend sends the third-party service a `totp.connected` webhook signed with `X-Signature-HMAC-SHA256` and returns `redirect_url` to the frontend for navigation.
5. On subsequent sign-ins, the third-party service checks the six-digit code with `POST /v1/totp/verify`.

## Database schema

- **`users`** - Authenticator accounts (`id`, `email`, `password_hash`, `status`).
- **`user_sessions`** - Client sessions (`id`, `user_id`, `refresh_token_hash`, `device_name`, `ip_address`, `expires_at`).
- **`applications`** - Registered third-party services (`id`, `owner_user_id`, `issuer_name`, `webhook_url`, `is_active`). The `webhook_secret` field is planned.
- **`api_keys`** - Scoped B2B API keys (`id`, `application_id`, `key_prefix`, `key_hash`, `scopes`, `expires_at`, `revoked_at`, `last_used_at`).
- **`connect_sessions` (planned)** - One-time enrollment sessions (`id`, `application_id`, `external_user_id`, `account_label`, `token_hash`, `redirect_url`, `expires_at`, `consumed_at`).
- **`totp_enrollments`** - Linked accounts and encrypted TOTP secrets (`id`, `user_id`, `application_id`, `external_user_id`, `issuer`, `account_label`, `encrypted_secret`, `secret_nonce`, `encrypted_dek`, `dek_nonce`, `key_version`, `algorithm`, `digits`, `period`, `status`, `last_verified_step`, `revision`, `deleted_at`). Both `application_id` and `external_user_id` are required. Status `1` means active and `2` means disabled.

## API contract for frontend mocks

Use the DTOs below for Angular types and mock responses. The Rust server currently implements only `GET /healthz`. The wired auth routes are stubs and do not yet implement these DTOs, cookies, or error codes. Valid auth requests return `501` with `{ "code": "not_implemented" }`; some initial validation failures return `422` with `{ "code": "invalid_input" }`.

### Shared formats

- Use JSON with `snake_case` field names. Send `Content-Type: application/json` with JSON request bodies.
- A successful response has `success: true` and `data`. Use `data: []` for an empty list and `data: null` for a successful operation without a payload.
- An error response has `success: false`, `code`, and an optional `target` from the unified `ApiErrorTarget` type identifying the affected field or resource. Omit `target` when it does not apply. The frontend can map the combination to display copy.
- IDs generated by Authenticator are UUID strings. `external_user_id` is an opaque string supplied by the third-party service.
- Timestamps are ISO 8601 UTC strings such as `2026-11-26T12:00:30Z`.
- A field typed as `T | null` is present in the response and explicitly null when unset. A request field marked with `?` may be omitted.

```ts
type Uuid = string;
type UtcTimestamp = string;
type ApiSuccess<T> = { success: true; data: T };
type ApiError = { success: false; code: ApiErrorCode; target?: ApiErrorTarget };
type ApiResponse<T> = ApiSuccess<T> | ApiError;
```

The tables show the HTTP status and the type of `data` in the **Success** column. In the planned contract, `GET /healthz` returns HTTP 200 with `{ "success": true, "data": { "status": "ok" } }`.

### Authentication

Register and login are public. `/v1/auth/me`, Applications, and Vault use the `access_token` cookie. Refresh and logout use the `refresh_token` cookie.

| Method and path | JSON body | Success |
| --- | --- | --- |
| `POST /v1/auth/register` | `Credentials` | `201 AuthSession` |
| `POST /v1/auth/login` | `Credentials` | `200 AuthSession` |
| `GET /v1/auth/me` | — | `200 null` |
| `POST /v1/auth/refresh` | — | `200 null` |
| `POST /v1/auth/logout` | — | `200 null` |

```ts
type Credentials = {
  email: string;
  password: string;
};

type AuthUser = {
  id: Uuid;
  email: string;
};

type AuthSession = {
  user: AuthUser;
};
```

`GET /v1/auth/me` checks whether the browser has a valid access token. A valid token gets HTTP 200 with `{ "success": true, "data": null }`. A missing, invalid, or expired token gets HTTP 401 with `{ "success": false, "code": "UNAUTHORIZED" }`.

### Session cookies

- Registration and login set both cookies via `Set-Cookie`. Refresh replaces both cookies and invalidates the previous refresh token. Logout revokes the session and clears both cookies, including when the refresh token is missing or already expired or revoked.
- In production, both cookies use `HttpOnly`, `Secure`, and `SameSite=Lax`. The API leaves `Domain` unset. `access_token` uses `Path=/v1` and `refresh_token` uses `Path=/v1/auth`. Logout clears them using the same paths.
- The browser handles the token values. Frontend code works with the JSON DTOs and leaves `localStorage` and `sessionStorage` free of tokens. For separate frontend and API origins within the same site, Angular uses `withCredentials: true`, and the API allows credentials for the configured frontend origin.
- The API checks the `Origin` of browser `POST`, `PATCH`, and `DELETE` requests on user routes against the configured frontend origin and returns `403 FORBIDDEN` for a missing or disallowed origin. Authentication responses use `Cache-Control: no-store`.
- On startup, the client calls `/v1/auth/me`. On `401 UNAUTHORIZED` from a protected route, it tries refresh once and retries the original request after success. Concurrent requests share one refresh attempt. If refresh returns `401 UNAUTHORIZED` with `target: "REFRESH_TOKEN"`, the client returns to sign-in.

### Applications

All routes are scoped to applications owned by the signed-in user.

| Method and path | JSON body | Success |
| --- | --- | --- |
| `POST /v1/applications` | `CreateApplicationRequest` | `201 Application` |
| `GET /v1/applications` | — | `200 Application[]` |
| `PATCH /v1/applications/{app_id}` | `UpdateApplicationRequest` | `200 Application` |
| `DELETE /v1/applications/{app_id}` | — | `200 null` |
| `POST /v1/applications/{app_id}/webhook-secret/rotate` | — | `200 RotateWebhookSecretResponse` |
| `POST /v1/applications/{app_id}/api-keys` | `CreateApiKeyRequest` | `201 CreatedApiKey` |
| `GET /v1/applications/{app_id}/api-keys` | — | `200 ApiKey[]` |
| `DELETE /v1/applications/{app_id}/api-keys/{key_id}` | — | `200 null` |

```ts
type CreateApplicationRequest = {
  issuer_name: string;
  webhook_url?: string | null;
};

type UpdateApplicationRequest = {
  issuer_name?: string;
  webhook_url?: string | null;
  is_active?: boolean;
};

type Application = {
  id: Uuid;
  issuer_name: string;
  webhook_url: string | null;
  is_active: boolean;
  created_at: UtcTimestamp;
  updated_at: UtcTimestamp;
};

type RotateWebhookSecretResponse = {
  webhook_secret: string;
};

type ApiKeyScope = "totp:enroll" | "totp:verify";

type CreateApiKeyRequest = {
  scopes: ApiKeyScope[];
  expires_at?: UtcTimestamp | null;
};

type ApiKey = {
  id: Uuid;
  key_prefix: string;
  scopes: ApiKeyScope[];
  created_at: UtcTimestamp;
  expires_at: UtcTimestamp | null;
  last_used_at: UtcTimestamp | null;
  revoked_at: UtcTimestamp | null;
};

type CreatedApiKey = ApiKey & {
  api_key: string;
};
```

- `issuer_name` is a display name of up to 64 characters. New applications default to `is_active: true` and `webhook_url: null` when the URL is omitted.
- For updates, omitted fields keep their current values. Send `webhook_url: null` to clear it.
- API key scopes must contain at least one value. Omitting `expires_at` creates a key with no expiration. Revoked keys stay in the list with `revoked_at` populated. Deleting an already revoked key returns `200` with `data: null`.
- The full key is returned only on creation. The webhook signing secret is returned only by the rotation endpoint. Show them to the owner on those successful responses. A disabled application rejects B2B runtime requests.

### Vault and connect approval

| Method and path | JSON body | Success |
| --- | --- | --- |
| `GET /v1/vault/connect-sessions/{connect_id}?token=...` | — | `200 ConnectSessionPreview` |
| `POST /v1/vault/connect-sessions/{connect_id}/approve` | `ApproveConnectRequest` | `200 ApproveConnectResponse` |
| `GET /v1/vault/items` | — | `200 VaultItem[]` |
| `DELETE /v1/vault/items/{item_id}` | — | `200 null` |

```ts
type ConnectSessionPreview = {
  id: Uuid;
  issuer_name: string;
  account_label: string;
  expires_at: UtcTimestamp;
};

type ApproveConnectRequest = {
  token: string;
};

type ApproveConnectResponse = {
  item_id: Uuid;
  redirect_url: string;
};

type VaultItem = {
  id: Uuid;
  application_id: Uuid;
  issuer: string;
  account_label: string;
  status: "active" | "disabled";
  code: string | null;
  code_expires_at: UtcTimestamp | null;
};
```

- The frontend opens `/connect/{connect_id}?token=...`. After sign-in, it uses the same ID and token to load the preview. Approving consumes the link and returns the destination in JSON. The frontend then navigates to that `redirect_url`.
- A vault code is a six-character string so values like `"042317"` keep the leading zero. Use `code_expires_at` for the countdown. After expiry, fetch the list again. Disabled items have both code fields set to `null`.
- Approval adds the item to the list. Unlinking removes it and triggers the `totp.disconnected` webhook.

Example vault response.

```json
{
  "success": true,
  "data": [
    {
      "id": "6c071584-657c-4228-84ed-5f8508f1cd86",
      "application_id": "dfd5f696-a68a-48f9-b9d2-aa9787c63fc2",
      "issuer": "Example Shop",
      "account_label": "Personal account",
      "status": "active",
      "code": "042317",
      "code_expires_at": "2026-11-26T12:00:30Z"
    }
  ]
}
```

### B2B TOTP runtime

These are server-to-server routes. The integration sends `X-API-Key` and URL-encodes `external_user_id` when using it as a path segment. The Angular client uses the resulting connect link and the Vault routes.

| Method and path | Required scope | JSON body | Success |
| --- | --- | --- | --- |
| `POST /v1/totp/connect-sessions` | `totp:enroll` | `CreateConnectSessionRequest` | `201 CreatedConnectSession` |
| `POST /v1/totp/verify` | `totp:verify` | `VerifyTotpRequest` | `200 VerifyTotpResponse` |
| `GET /v1/totp/enrollments/{external_user_id}` | `totp:enroll` | — | `200 EnrollmentState` |
| `DELETE /v1/totp/enrollments/{external_user_id}` | `totp:enroll` | — | `200 null` |

```ts
type CreateConnectSessionRequest = {
  external_user_id: string;
  account_label: string;
  redirect_url: string;
};

type CreatedConnectSession = {
  id: Uuid;
  connect_url: string;
  expires_at: UtcTimestamp;
};

type VerifyTotpRequest = {
  external_user_id: string;
  code: string;
};

type VerifyTotpResponse = {
  valid: boolean;
};

type EnrollmentState = {
  external_user_id: string;
  status: "active" | "disabled" | "not_enrolled";
};
```

`external_user_id` and `account_label` are required and limited to 255 characters each. `connect_url` includes the one-time token. A well-formed but incorrect, expired, or replayed code returns `{ "success": true, "data": { "valid": false } }`, as does verification without an active enrollment. Malformed codes use `400 BAD_REQUEST`. The B2B delete route disables an enrollment and returns `200` with `data: null` even when it is already disabled or absent.

### Error codes for mocks

```ts
type ApiErrorCode =
  | "BAD_REQUEST"
  | "UNAUTHORIZED"
  | "FORBIDDEN"
  | "NOT_FOUND"
  | "ALREADY_EXISTS"
  | "ALREADY_CONSUMED"
  | "EXPIRED"
  | "INTERNAL_ERROR";

type ApiErrorTarget =
  | "EMAIL"
  | "CREDENTIALS"
  | "ACCESS_TOKEN"
  | "REFRESH_TOKEN"
  | "API_KEY"
  | "ORIGIN"
  | "APPLICATION"
  | "ENROLLMENT"
  | "CONNECT_SESSION"
  | "TOTP_CODE";
```

The following pairs define the mock responses. For input errors, use a target when the affected field is represented in `ApiErrorTarget`. Omit it for malformed JSON and errors that cannot be attributed to a single target. Login returns the same credentials error for an unknown email and an incorrect password.

| HTTP status | `code` | `target` | When to return it |
| --- | --- | --- | --- |
| `400` | `BAD_REQUEST` | optional | Malformed JSON, missing required input, or invalid input values |
| `401` | `UNAUTHORIZED` | `ACCESS_TOKEN` or omitted | Missing, invalid, or expired access token |
| `401` | `UNAUTHORIZED` | `API_KEY` | Missing, invalid, or expired API key |
| `401` | `UNAUTHORIZED` | `CREDENTIALS` | Login credentials are incorrect |
| `401` | `UNAUTHORIZED` | `REFRESH_TOKEN` | Refresh token is missing, invalid, revoked, or expired |
| `403` | `FORBIDDEN` | `ORIGIN` | Browser origin is missing or disallowed |
| `403` | `FORBIDDEN` | `API_KEY` | API key lacks the required scope |
| `403` | `FORBIDDEN` | `APPLICATION` | The API key's application is disabled |
| `404` | `NOT_FOUND` | optional | Resource is missing, belongs to another user, or the connect ID and token do not match |
| `409` | `ALREADY_EXISTS` | `EMAIL` | Registration email is already in use |
| `409` | `ALREADY_EXISTS` | `ENROLLMENT` | An active enrollment already exists for this application and external user |
| `409` | `ALREADY_CONSUMED` | `CONNECT_SESSION` | The connect session was already approved |
| `410` | `EXPIRED` | `CONNECT_SESSION` | The connect link has expired |
| `500` | `INTERNAL_ERROR` | omitted | Unexpected server error |

For example, an email lookup that cannot find an account may return `{ "success": false, "code": "NOT_FOUND", "target": "EMAIL" }`. The login endpoint uses `UNAUTHORIZED` and `CREDENTIALS` for either incorrect credential.

For frontend development, the mock can simulate the session in memory. Include `/v1/auth/me` with and without a valid session, an expired access token followed by successful refresh, and logout followed by `401 UNAUTHORIZED` from `/v1/auth/me`. Include empty and populated lists and connect links in pending, expired, and consumed states. Generate code expiry timestamps relative to the mock clock so the vault countdown works during development.

## Local development

Use Rust and Cargo, Docker Compose, and the SQLx CLI with PostgreSQL support. The server requires PostgreSQL at startup and reads `DATABASE_URL` and `KEY_ENCRYPTION_KEY` from the environment. `BIND_ADDR` defaults to `127.0.0.1:3000`. The example key is a placeholder for the planned encryption feature. From the repository root, create the local environment files and start PostgreSQL. If the files already exist, keep your local settings and make sure the credentials and port match `DATABASE_URL`.

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

With the default bind address, check the server from another terminal. The current response is `{ "status": "ok" }`; the response envelope in the planned contract has not been implemented.

```bash
curl http://127.0.0.1:3000/healthz
```

The Compose file currently reads the optional host port from `DATABASE_POST`, defaulting to `5432`. If you set it, update the port in `backend/.env` as well.
