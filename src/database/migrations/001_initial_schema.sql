-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_id TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    email TEXT,
    avatar_url TEXT,
    is_admin BOOLEAN DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_login DATETIME
);

-- Environments table
CREATE TABLE IF NOT EXISTS environments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    color TEXT DEFAULT '#6b7280',
    display_order INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Servers table
CREATE TABLE IF NOT EXISTS servers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    hostname TEXT NOT NULL,
    port INTEGER DEFAULT 22,
    username TEXT NOT NULL,
    environment_id INTEGER,

    -- Authentication (encrypted)
    auth_type TEXT DEFAULT 'password',
    password_encrypted BLOB,
    private_key_encrypted BLOB,
    passphrase_encrypted BLOB,

    -- Metadata
    description TEXT,
    os_type TEXT DEFAULT 'linux',
    tags TEXT,
    notes TEXT,

    -- Status
    status TEXT DEFAULT 'unknown',
    is_intranet BOOLEAN DEFAULT 0,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_accessed DATETIME,

    FOREIGN KEY (environment_id) REFERENCES environments(id) ON DELETE SET NULL
);

-- Jump hosts table
CREATE TABLE IF NOT EXISTS jump_hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id INTEGER NOT NULL,
    jump_server_id INTEGER NOT NULL,
    jump_order INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    FOREIGN KEY (jump_server_id) REFERENCES servers(id) ON DELETE CASCADE,
    UNIQUE(server_id, jump_server_id, jump_order)
);

-- Service credentials table
CREATE TABLE IF NOT EXISTS service_credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    service_type TEXT NOT NULL,
    credentials_encrypted BLOB,
    config_json TEXT,
    is_active BOOLEAN DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Access logs table
CREATE TABLE IF NOT EXISTS access_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    accessed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    connection_type TEXT DEFAULT 'ssh',
    success BOOLEAN DEFAULT 1,
    notes TEXT,

    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_servers_environment ON servers(environment_id);
CREATE INDEX IF NOT EXISTS idx_servers_hostname ON servers(hostname);
CREATE INDEX IF NOT EXISTS idx_access_logs_server ON access_logs(server_id);
CREATE INDEX IF NOT EXISTS idx_access_logs_user ON access_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_access_logs_accessed_at ON access_logs(accessed_at);
CREATE INDEX IF NOT EXISTS idx_service_credentials_type ON service_credentials(service_type);

-- Insert default environments
INSERT OR IGNORE INTO environments (name, description, color, display_order) VALUES
    ('production', 'Production servers', '#ef4444', 1),
    ('staging', 'Staging servers', '#f59e0b', 2),
    ('development', 'Development servers', '#3b82f6', 3);
