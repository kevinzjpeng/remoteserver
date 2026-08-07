# Server Access Manager - Design Specification

**Date:** 2026-08-07
**Status:** Draft
**Author:** Claude (with user input)

## Overview

A web-based server access management system that stores SSH credentials, provides GitHub OAuth authentication, includes an admin panel for credential management, and generates topology diagrams showing server relationships by environment.

## Key Requirements

1. **Single-user application** with GitHub OAuth login
2. **SSH credential storage** (host, port, user, keys, passwords) with metadata
3. **Admin panel** for managing all credentials (SSH, GitHub OAuth, API keys)
4. **Auto-generated topology diagrams** grouped by environment
5. **Encrypted sensitive fields** using master encryption key
6. **Single binary deployment**

## Architecture

### Approach: Monolithic with Server-Rendered Templates + HTMX

**Application Structure:**

```
┌─────────────────────────────────────────┐
│         Single Go Binary                │
├─────────────────────────────────────────┤
│  HTTP Server (Chi router)               │
├─────────────────────────────────────────┤
│  Template Engine (html/template)        │
├─────────────────────────────────────────┤
│  HTMX for dynamic interactions          │
├─────────────────────────────────────────┤
│  GitHub OAuth Authentication            │
├─────────────────────────────────────────┤
│  SQLite Database (encrypted fields)     │
├─────────────────────────────────────────┤
│  Cytoscape.js for topology viz          │
└─────────────────────────────────────────┘
```

### Key Components

1. **Web Server**
   - Go HTTP server using Chi router
   - Serves HTML pages with embedded templates
   - REST API endpoints for HTMX calls
   - Single port (8080)

2. **Template System**
   - Go's `html/template`
   - Base layout with navigation
   - Reusable components

3. **Frontend**
   - HTMX for AJAX interactions
   - Alpine.js for lightweight client-side state
   - Cytoscape.js for topology visualization
   - Pico CSS for styling

4. **Database Layer**
   - SQLite with migrations
   - Repository pattern for data access
   - Encrypted fields for sensitive data

5. **Authentication**
   - GitHub OAuth flow
   - Session management with secure cookies
   - Middleware to protect routes

## Data Model

### Core Entities

#### 1. Users Table
```sql
users (
    id INTEGER PRIMARY KEY,
    github_id TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    email TEXT,
    avatar_url TEXT,
    is_admin BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP,
    last_login TIMESTAMP
)
```

#### 2. Environments Table
```sql
environments (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,
    display_order INTEGER,
    created_at TIMESTAMP
)
```

#### 3. Servers Table
```sql
servers (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    hostname TEXT NOT NULL,
    port INTEGER DEFAULT 22,
    username TEXT NOT NULL,
    environment_id INTEGER,
    
    -- Encrypted fields
    auth_type TEXT,
    password_encrypted BLOB,
    private_key_encrypted BLOB,
    passphrase_encrypted BLOB,
    
    -- Metadata
    description TEXT,
    os_type TEXT,
    tags TEXT,
    notes TEXT,
    
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    last_accessed TIMESTAMP,
    
    FOREIGN KEY (environment_id) REFERENCES environments(id)
)
```

#### 4. Jump Hosts Table
```sql
jump_hosts (
    id INTEGER PRIMARY KEY,
    server_id INTEGER NOT NULL,
    jump_server_id INTEGER NOT NULL,
    jump_order INTEGER DEFAULT 1,
    
    FOREIGN KEY (server_id) REFERENCES servers(id),
    FOREIGN KEY (jump_server_id) REFERENCES servers(id)
)
```

#### 5. Service Credentials Table
```sql
service_credentials (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    service_type TEXT NOT NULL,
    credentials_encrypted BLOB,
    config_json TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
)
```

#### 6. Access Logs Table
```sql
access_logs (
    id INTEGER PRIMARY KEY,
    server_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    accessed_at TIMESTAMP,
    connection_type TEXT,
    success BOOLEAN,
    notes TEXT,
    
    FOREIGN KEY (server_id) REFERENCES servers(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
)
```

### Encryption Strategy

- Master encryption key from environment variable `ENCRYPTION_KEY`
- AES-256-GCM for field-level encryption
- Encrypted fields: `password_encrypted`, `private_key_encrypted`, `passphrase_encrypted`, `credentials_encrypted`
- Decryption in application layer

## Authentication & Authorization

### GitHub OAuth Flow

1. User clicks "Login with GitHub"
2. Redirect to GitHub OAuth consent screen
3. After approval, callback with authorization code
4. Exchange code for access token
5. Fetch user profile from GitHub API
6. Create/update user record
7. Set session cookie
8. Redirect to dashboard

### Session Management

- Secure, HTTP-only session cookies
- Session contains: user_id, github username, expiry
- Duration: configurable (default 24 hours)
- Cookie settings: `Secure`, `HttpOnly`, `SameSite=Strict`

### Authorization

- Single-user model: first user is admin
- `is_admin` flag in users table
- Admin routes: `/admin/*`, `/servers/*`, `/environments/*`
- Middleware checks authentication and admin status

### Security Measures

- CSRF protection on state-changing requests
- Rate limiting on login endpoint
- Secure cookie attributes
- Session invalidation on logout

## Admin Panel

### Sections

1. **GitHub OAuth Configuration**
   - Client ID, Client Secret (encrypted)
   - Test connection button
   - Status indicator

2. **Encryption Key Management**
   - Display key status
   - Generate new key helper
   - Re-encryption warning

3. **Environment Management**
   - CRUD operations for environments
   - Color picker for topology
   - Drag-and-drop reordering

4. **Server Management**
   - List view with filters and search
   - Server form with conditional fields
   - Import/export functionality
   - Test SSH connection

5. **Service Credentials Management**
   - List of configured services
   - Dynamic forms based on service type
   - Active/inactive toggle

6. **Access Logs Viewer**
   - Filterable log table
   - Date range, server, success filters
   - Export to CSV

7. **System Settings**
   - Session duration
   - Backup/restore database
   - Application logs
   - Health check

### Layout

```
┌─────────────────────────────────────────────┐
│  Admin Panel                    [User] [Logout] │
├─────────────────────────────────────────────┤
│  Sidebar          │  Main Content Area      │
│                   │                         │
│  • Dashboard      │  [Dynamic content]      │
│  • Environments   │                         │
│  • Servers        │                         │
│  • Credentials    │                         │
│  • Access Logs    │                         │
│  • Settings       │                         │
└─────────────────────────────────────────────┘
```

### UX Features

- HTMX for inline editing without page reloads
- Alpine.js for dropdowns, modals, conditional fields
- Toast notifications
- Confirmation modals
- Client-side and server-side validation
- SSH connection testing before saving

## Topology Visualization

### Library: Cytoscape.js

**Why Cytoscape.js:**
- Purpose-built for network/graph visualization
- Lightweight (~200KB)
- Automatic layout algorithms
- Interactive (click, zoom, pan, drag)
- Style nodes/edges based on data

### Topology Data Structure

```json
{
  "nodes": [
    {
      "id": "server-1",
      "name": "web-prod-01",
      "environment": "production",
      "type": "web",
      "status": "healthy"
    }
  ],
  "edges": [
    {
      "source": "server-1",
      "target": "server-2",
      "type": "jump_host",
      "label": "SSH"
    }
  ],
  "groups": [
    {
      "id": "production",
      "name": "Production",
      "color": "#ef4444",
      "node_ids": ["server-1", "server-2"]
    }
  ]
}
```

### Features

1. **Environment-Based Grouping**
   - Compound nodes for environments
   - Colored boundaries per environment
   - Configurable colors

2. **Auto-Layout**
   - Breadthfirst or CoSE layout
   - Automatic adjustment on changes
   - Smooth transitions

3. **Visual Indicators**
   - Node color by environment
   - Icon/shape by server type
   - Border for health status
   - Edge styles for connection types

4. **Interactivity**
   - Click node → details sidebar (HTMX)
   - Double-click → edit page
   - Hover → tooltip
   - Zoom/pan
   - Drag nodes
   - Filter by environment/tags

5. **Views**
   - Full topology (default)
   - Environment focus
   - Jump host chains
   - Filtered view

6. **API Endpoint**
   - `GET /api/topology`
   - Returns nodes, edges, groups
   - Auto-refresh every 30 seconds

### Page Layout

```
┌──────────────────────────────────────────────────────────┐
│  Topology                                    [Refresh]   │
├──────────────────────────────────────────────────────────┤
│  Filters: [Environment ▼] [Status ▼] [Tags ▼] [Search] │
├─────────────────────────────────────────┬────────────────┤
│                                         │                │
│         Topology Diagram                │  Details       │
│         (Cytoscape.js)                  │  Panel         │
│                                         │  (HTMX)        │
│                                         │                │
└─────────────────────────────────────────┴────────────────┘
```

### Export & Sharing

- Export as PNG/SVG
- Export data as JSON
- Shareable URLs with filters

## Technology Stack

### Backend
- **Language:** Go 1.21+
- **HTTP Router:** Chi
- **Database:** SQLite with go-sqlite3
- **Migrations:** golang-migrate
- **Templates:** html/template
- **Encryption:** Go crypto/aes, crypto/cipher

### Frontend
- **Interactivity:** HTMX 1.9+
- **Client State:** Alpine.js 3.x
- **Topology:** Cytoscape.js 3.x
- **Styling:** Pico CSS
- **Icons:** Font Awesome or Heroicons

### Configuration
- Environment variables for sensitive config
- YAML/JSON config file for non-sensitive settings
- `.env` file support for development

## Deployment

- Single binary deployment
- Runs as systemd service or Docker container
- SQLite database file in configurable location
- Static assets embedded in binary

## Configuration

```yaml
# Environment variables
GITHUB_CLIENT_ID=your_client_id
GITHUB_CLIENT_SECRET=your_encrypted_client_secret
SESSION_SECRET=random_32_byte_secret
ENCRYPTION_KEY=your_master_encryption_key
SESSION_DURATION=24h
DATABASE_PATH=/path/to/db.sqlite
PORT=8080
```

## Security Considerations

1. All sensitive data encrypted at rest
2. HTTPS required in production
3. Secure session cookies
4. CSRF protection
5. Rate limiting on authentication
6. Audit logging for all access
7. Input validation and sanitization
8. SQL injection prevention (parameterized queries)

## Future Enhancements (Out of Scope for MVP)

- Multi-user support with RBAC
- Team-based access control
- SSH session recording
- Automatic credential rotation
- Integration with cloud providers
- Mobile app
- API for external tools
- Webhook notifications

## Success Criteria

1. Can store and retrieve SSH credentials securely
2. GitHub OAuth login works smoothly
3. Admin can configure all credentials through UI
4. Topology diagram auto-generates and is interactive
5. Single binary deployment works
6. All sensitive data is encrypted

## Open Questions

- None at this time

## Appendix

### Encryption Implementation

```go
// Pseudocode for field encryption
func encrypt(plaintext []byte, key []byte) ([]byte, error) {
    block, err := aes.NewCipher(key)
    if err != nil {
        return nil, err
    }
    
    gcm, err := cipher.NewGCM(block)
    if err != nil {
        return nil, err
    }
    
    nonce := make([]byte, gcm.NonceSize())
    if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
        return nil, err
    }
    
    return gcm.Seal(nonce, nonce, plaintext, nil), nil
}

func decrypt(ciphertext []byte, key []byte) ([]byte, error) {
    block, err := aes.NewCipher(key)
    if err != nil {
        return nil, err
    }
    
    gcm, err := cipher.NewGCM(block)
    if err != nil {
        return nil, err
    }
    
    nonceSize := gcm.NonceSize()
    nonce, ciphertext := ciphertext[:nonceSize], ciphertext[nonceSize:]
    
    return gcm.Open(nil, nonce, ciphertext, nil)
}
```
