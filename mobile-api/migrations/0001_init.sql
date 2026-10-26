CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE users (
    id uuid PRIMARY KEY,
    username varchar(100) NOT NULL,
    password_hash varchar(500) NOT NULL,
    display_name varchar(200) NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL,
    CONSTRAINT users_username_uidx UNIQUE (username)
);

CREATE TABLE clients (
    id uuid PRIMARY KEY,
    code varchar(100) NOT NULL,
    name varchar(200) NOT NULL,
    api_key_hash varchar(64) NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL,
    CONSTRAINT clients_code_uidx UNIQUE (code),
    CONSTRAINT clients_api_key_hash_uidx UNIQUE (api_key_hash)
);

CREATE TABLE request_logs (
    id uuid NOT NULL,
    request_id uuid NOT NULL,
    requested_at timestamptz NOT NULL,
    duration_ms integer NOT NULL,
    http_method varchar(16) NOT NULL,
    path varchar(500) NOT NULL,
    query_string varchar(2000),
    client_id uuid,
    client_code varchar(100),
    ip_address varchar(100),
    request_headers text,
    request_body text,
    status_code integer NOT NULL,
    response_body text,
    is_error boolean NOT NULL,
    error_message varchar(2000),
    exception_details text,
    iso_response_code varchar(8),
    account_number varchar(64),
    src_account varchar(64),
    dest_account varchar(64),
    member_id varchar(64),
    tran_code varchar(64),
    amount numeric(18, 2),
    upstream_path varchar(500),
    upstream_status integer,
    upstream_duration_ms integer,
    upstream_body text,
    search_text text,
    PRIMARY KEY (id, requested_at),
    CONSTRAINT request_logs_client_id_fkey
        FOREIGN KEY (client_id) REFERENCES clients (id) ON DELETE SET NULL
) PARTITION BY RANGE (requested_at);

CREATE INDEX request_logs_requested_at_idx ON request_logs (requested_at DESC);
CREATE INDEX request_logs_path_idx ON request_logs (path);
CREATE INDEX request_logs_client_code_idx ON request_logs (client_code);
CREATE INDEX request_logs_status_code_idx ON request_logs (status_code);
CREATE INDEX request_logs_is_error_idx ON request_logs (is_error);
CREATE INDEX request_logs_iso_response_code_idx ON request_logs (iso_response_code);
CREATE INDEX request_logs_tran_code_idx ON request_logs (tran_code);
CREATE INDEX request_logs_account_number_idx ON request_logs (account_number);
CREATE INDEX request_logs_id_idx ON request_logs (id);
CREATE INDEX request_logs_search_trgm ON request_logs USING gin (search_text gin_trgm_ops);

CREATE TABLE request_logs_default PARTITION OF request_logs DEFAULT;

CREATE OR REPLACE FUNCTION ensure_request_log_partition(month_start date)
RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    part_name text := 'request_logs_' || to_char(month_start, 'YYYY_MM');
    start_ts timestamptz := month_start::timestamp AT TIME ZONE 'UTC';
    end_ts timestamptz := (month_start + INTERVAL '1 month')::timestamp AT TIME ZONE 'UTC';
BEGIN
    IF to_regclass(part_name) IS NULL THEN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF request_logs FOR VALUES FROM (%L) TO (%L)',
            part_name,
            start_ts,
            end_ts
        );
    END IF;
END;
$$;

SELECT ensure_request_log_partition((date_trunc('month', (now() AT TIME ZONE 'UTC'))::date - INTERVAL '1 month')::date);
SELECT ensure_request_log_partition((date_trunc('month', (now() AT TIME ZONE 'UTC'))::date));
SELECT ensure_request_log_partition((date_trunc('month', (now() AT TIME ZONE 'UTC'))::date + INTERVAL '1 month')::date);
