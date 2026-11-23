-- Add migration script here

CREATE TABLE users
(
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1, -- 1: active, 2: suspended
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user_sessions
(
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash BYTEA NOT NULL UNIQUE,
    device_name TEXT,
    ip_address INET,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);

CREATE TABLE applications
(
    id UUID PRIMARY KEY,
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    issuer_name VARCHAR(64) NOT NULL,
    webhook_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE api_keys
(
    id UUID PRIMARY KEY,
    application_id UUID NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    key_prefix VARCHAR(12) NOT NULL,
    key_hash BYTEA NOT NULL UNIQUE,
    scopes TEXT[] NOT NULL DEFAULT ARRAY['totp:enroll', 'totp:verify'],
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix) WHERE revoked_at IS NULL;

CREATE TABLE totp_enrollments
(
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,

    -- identity user inside external service
    external_user_id VARCHAR(255) NOT NULL,

    -- metadata
    issuer VARCHAR(120) NOT NULL,
    account_label VARCHAR(255) NOT NULL,

    -- encryption
    encrypted_secret BYTEA NOT NULL,
    secret_nonce BYTEA NOT NULL,
    encrypted_dek BYTEA NOT NULL,
    dek_nonce BYTEA NOT NULL,
    key_version INT NOT NULL DEFAULT 1,

    -- RFC 6238 params
    algorithm SMALLINT NOT NULL DEFAULT 1, -- 1: SHA1, 2: SHA256, 3: SHA512
    digits SMALLINT NOT NULL DEFAULT 6,
    period SMALLINT NOT NULL DEFAULT 30,

    status SMALLINT NOT NULL DEFAULT 1, -- 1: active, 2: disabled
    last_verified_step BIGINT,

    revision BIGINT NOT NULL DEFAULT 1,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX uq_enrollment_app_ext_user
    ON totp_enrollments (application_id, external_user_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_enrollments_user_sync
    ON totp_enrollments (user_id, updated_at);