ALTER TABLE users ADD COLUMN role varchar(20) NOT NULL DEFAULT 'admin';
UPDATE users SET role = 'admin';
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'user';
ALTER TABLE users ADD CONSTRAINT users_role_check CHECK (role IN ('admin', 'user'));

ALTER TABLE clients ADD COLUMN api_key varchar(80);
