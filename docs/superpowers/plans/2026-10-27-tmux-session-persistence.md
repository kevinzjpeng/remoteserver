# Tmux-like SSH Session Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make SSH terminal sessions persist when the browser tab/window closes, so reconnecting shows the current terminal state (like tmux attach).

**Architecture:** The existing `SessionManager` already has the right structure (session map, broadcast channel, command sender) but `terminal.rs` ignores it. Refactor `handle_socket` to: (1) check for existing session, (2) attach or create, (3) send buffered output on reconnect, (4) on disconnect just decrement connections without killing SSH. Add a ring buffer to `Session` to store recent terminal output.

**Tech Stack:** Rust, axum (WebSocket), russh, tokio broadcast/mpsc channels

**Spec:** Conversational requirement — user wants tmux-like behavior: close browser → reopen → see same terminal state.

## Global Constraints

- Rust 2021 edition
- Use existing `russh` 0.40 for SSH
- Use existing `tokio` broadcast channels for multi-client fan-out
- Don't break existing auth/credential flow
- Keep the `SessionManager` API compatible (it's already wired into `AppState`)

## Review Focus

1. **Race condition on session creation**: Two WebSocket connects arriving simultaneously for the same server_id+session must not create two SSH connections. Lock the sessions map during check-and-insert.
2. **Buffer size**: Ring buffer must be bounded (e.g., 64KB) to avoid unbounded memory growth on long-running sessions.
3. **SSH task lifetime**: The SSH I/O task must outlive all WebSocket clients. It should only exit when the session is explicitly closed or the SSH connection dies.
4. **Reconnect output**: On reconnect, the client must receive the buffered terminal state so they see the current screen, not a blank terminal.
5. **Cleanup**: Sessions with 0 connections that have been idle beyond `idle_timeout` must still be cleaned up (existing cleanup task handles this).

---

### Task 1: Add terminal output buffer to Session

**Files:**
- Modify: `src/session/manager.rs`

**Interfaces:**
- Consumes: nothing new
- Produces: `Session` gains `output_buffer: Arc<Mutex<Vec<u8>>>` field; `SessionManager` gains `append_output(session_id, data)` and `get_buffer(session_id) -> Vec<u8>` methods

- [ ] **Step 1: Add output_buffer field to Session struct**

In `src/session/manager.rs`, add to the `Session` struct:

```rust
pub output_buffer: Arc<Mutex<Vec<u8>>>,
```

Initialize it in `store_session`:

```rust
output_buffer: Arc::new(Mutex::new(Vec::new())),
```

- [ ] **Step 2: Add BUFFER_SIZE constant and append/get methods**

Add to `SessionManager` impl:

```rust
const BUFFER_SIZE: usize = 65536; // 64KB

pub fn append_output(&self, server_id: i64, session_name: &str, data: &[u8]) {
    let session_id = format!("{}:{}", server_id, session_name);
    let sessions = self.sessions.lock().unwrap();
    if let Some(session) = sessions.get(&session_id) {
        let mut buf = session.output_buffer.lock().unwrap();
        buf.extend_from_slice(data);
        // Trim from front if over capacity
        if buf.len() > Self::BUFFER_SIZE {
            let excess = buf.len() - Self::BUFFER_SIZE;
            buf.drain(..excess);
        }
    }
}

pub fn get_buffer(&self, server_id: i64, session_name: &str) -> Vec<u8> {
    let session_id = format!("{}:{}", server_id, session_name);
    let sessions = self.sessions.lock().unwrap();
    if let Some(session) = sessions.get(&session_id) {
        session.output_buffer.lock().unwrap().clone()
    } else {
        Vec::new()
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add src/session/manager.rs
git commit -m "feat: add terminal output buffer to Session for reconnect replay"
```

---

### Task 2: Refactor terminal handler to reuse existing sessions

**Files:**
- Modify: `src/handlers/terminal.rs`

**Interfaces:**
- Consumes: `SessionManager::get_session`, `SessionManager::store_session`, `SessionManager::append_output`, `SessionManager::get_buffer`, `SessionManager::increment_connections`, `SessionManager::decrement_connections`, `Session::data_broadcaster`, `Session::command_sender`, `Session::output_buffer`
- Produces: `handle_socket` now attaches to existing sessions or creates new ones; on disconnect it detaches without killing SSH

- [ ] **Step 1: Rewrite handle_socket to check for existing session first**

Replace the entire `handle_socket` function body. The new flow:

1. Build `session_id` from `server_id` + `params.session`
2. Check `state.ssh_session_manager.get_session(...)` — if exists, attach
3. If not exists, create SSH connection (existing auth logic) and store in manager
4. Subscribe to `session.data_broadcaster`
5. Send buffered output to new client
6. Increment connections
7. Relay loop: forward broadcast → WebSocket, WebSocket → `command_sender`
8. On disconnect: decrement connections, unsubscribe — do NOT close SSH

Here is the new `handle_socket`:

```rust
async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    _session_data: SessionData,
    params: TerminalParams,
) {
    tracing::info!(
        "Terminal WebSocket connection started: server_id={}, session={}",
        params.server_id,
        params.session
    );

    let socket_tx = Arc::new(Mutex::new(socket));

    // Check if session already exists
    let session = if let Some(existing) = state.ssh_session_manager.get_session(params.server_id, &params.session) {
        tracing::info!("Attaching to existing session: {}", existing.id);
        existing
    } else {
        // Create new SSH connection
        tracing::info!("Creating new SSH session for server_id={}, session={}", params.server_id, params.session);

        let server_repo = ServerRepository::new(state.db.clone(), state.config.encryption_key.as_bytes().to_vec());
        let server = match server_repo.get_by_id(params.server_id) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to get server: {}", e);
                let mut socket = socket_tx.lock().await;
                let _ = socket.send(Message::Text(format!("Error: Server not found: {}", e))).await;
                return;
            }
        };

        let password = if server.auth_type == "password" {
            match server_repo.get_password(params.server_id) {
                Ok(p) => Some(p),
                Err(e) => {
                    tracing::error!("Failed to get password: {}", e);
                    let mut socket = socket_tx.lock().await;
                    let _ = socket.send(Message::Text(format!("Error: Failed to get credentials: {}", e))).await;
                    return;
                }
            }
        } else {
            None
        };

        let private_key = if server.auth_type == "key" {
            match server_repo.get_private_key(params.server_id) {
                Ok(k) if !k.is_empty() => Some(k),
                Err(e) => {
                    tracing::error!("Failed to get private key: {}", e);
                    let mut socket = socket_tx.lock().await;
                    let _ = socket.send(Message::Text(format!("Error: Failed to get credentials: {}", e))).await;
                    return;
                }
                _ => None,
            }
        } else {
            None
        };

        let passphrase = if server.auth_type == "key" {
            match server_repo.get_passphrase(params.server_id) {
                Ok(p) if !p.is_empty() => Some(p),
                _ => None,
            }
        } else {
            None
        };

        let config = Arc::new(Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(30)),
            ..Config::default()
        });

        let ssh_handler = ClientHandler;
        let addr = format!("{}:{}", server.hostname, server.port);
        tracing::info!("Connecting to SSH at: {}", addr);

        let mut ssh_client = match russh::client::connect(config, &addr, ssh_handler).await {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("SSH connection failed to {}: {}", addr, e);
                let mut socket = socket_tx.lock().await;
                let _ = socket.send(Message::Text(format!("Error: SSH connection failed: {}", e))).await;
                return;
            }
        };

        // Authenticate
        tracing::info!("SSH authentication attempt - Username: {}, Auth Type: {}", server.username, server.auth_type);
        let auth_result = if let Some(pwd) = password {
            ssh_client.authenticate_password(&server.username, &pwd).await
        } else if let Some(key_str) = private_key {
            let key_pair = if let Some(pass) = passphrase {
                russh_keys::load_secret_key(&key_str, Some(&pass))
            } else {
                russh_keys::load_secret_key(&key_str, None)
            };
            match key_pair {
                Ok(key) => ssh_client.authenticate_publickey(&server.username, Arc::new(key)).await,
                Err(e) => {
                    tracing::error!("Failed to parse private key: {}", e);
                    let mut socket = socket_tx.lock().await;
                    let _ = socket.send(Message::Text(format!("Error: Failed to parse private key: {}", e))).await;
                    return;
                }
            }
        } else {
            Err(russh::Error::NoAuthMethod)
        };

        match auth_result {
            Ok(true) => tracing::info!("SSH authentication successful"),
            Ok(false) => {
                tracing::error!("SSH authentication failed");
                let mut socket = socket_tx.lock().await;
                let _ = socket.send(Message::Text("Error: SSH authentication failed".to_string())).await;
                return;
            }
            Err(e) => {
                tracing::error!("SSH authentication error: {}", e);
                let mut socket = socket_tx.lock().await;
                let _ = socket.send(Message::Text(format!("Error: SSH authentication error: {}", e))).await;
                return;
            }
        }

        // Open channel
        let mut channel = match ssh_client.channel_open_session().await {
            Ok(ch) => ch,
            Err(e) => {
                tracing::error!("Failed to open SSH channel: {}", e);
                let mut socket = socket_tx.lock().await;
                let _ = socket.send(Message::Text(format!("Error: Failed to open session: {}", e))).await;
                return;
            }
        };

        if let Err(e) = channel.request_pty(false, "xterm-256color", 80, 40, 0, 0, &[]).await {
            tracing::error!("Failed to request PTY: {}", e);
            let mut socket = socket_tx.lock().await;
            let _ = socket.send(Message::Text(format!("Error: Failed to request PTY: {}", e))).await;
            return;
        }

        if let Err(e) = channel.request_shell(true).await {
            tracing::error!("Failed to start shell: {}", e);
            let mut socket = socket_tx.lock().await;
            let _ = socket.send(Message::Text(format!("Error: Failed to start shell: {}", e))).await;
            return;
        }

        // Create channels for SSH I/O
        let (ssh_cmd_tx, mut ssh_cmd_rx) = tokio::sync::mpsc::channel::<SshCommand>(100);
        let (ssh_data_tx, mut ssh_data_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
        let (broadcast_tx, _) = tokio::sync::broadcast::channel::<Vec<u8>>(256);

        // Spawn SSH I/O task — this outlives all WebSocket clients
        let broadcast_tx_clone = broadcast_tx.clone();
        let server_id = params.server_id;
        let session_name = params.session.clone();
        let session_mgr = state.ssh_session_manager.clone();
        tokio::spawn(async move {
            tracing::info!("SSH I/O task started");
            loop {
                tokio::select! {
                    msg = channel.wait() => {
                        match msg {
                            Some(ChannelMsg::Data { data }) => {
                                // Append to output buffer
                                session_mgr.append_output(server_id, &session_name, &data);
                                // Broadcast to all connected WebSocket clients
                                let _ = broadcast_tx_clone.send(data.to_vec());
                            }
                            Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => {
                                tracing::info!("SSH channel closed");
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(cmd) = ssh_cmd_rx.recv() => {
                        match cmd {
                            SshCommand::Data(data) => {
                                if channel.data(data.as_slice()).await.is_err() {
                                    break;
                                }
                            }
                            SshCommand::Resize { cols, rows } => {
                                if let Err(e) = channel.window_change(cols, rows, 0, 0).await {
                                    tracing::error!("Failed to resize terminal: {}", e);
                                }
                            }
                            SshCommand::Close => {
                                tracing::info!("Received close command");
                                break;
                            }
                        }
                    }
                }
            }
            tracing::info!("SSH I/O task ended");
        });

        // Store session in manager
        let session = state.ssh_session_manager.store_session(
            params.server_id,
            &params.session,
            ssh_client,
            ssh_cmd_tx,
            broadcast_tx,
        );

        // Send success message
        {
            let mut socket = socket_tx.lock().await;
            let _ = socket.send(Message::Text(format!("Connected to {}\r\n", server.name))).await;
        }

        session
    };

    // Subscribe to broadcast channel for this session
    let mut broadcast_rx = session.data_broadcaster.subscribe();

    // Send buffered output so reconnecting client sees current state
    let buffer = state.ssh_session_manager.get_buffer(params.server_id, &params.session);
    if !buffer.is_empty() {
        tracing::info!("Sending {} bytes of buffered output to reconnecting client", buffer.len());
        let mut socket = socket_tx.lock().await;
        let _ = socket.send(Message::Binary(buffer)).await;
    }

    // Increment connection count
    state.ssh_session_manager.increment_connections(params.server_id, &params.session);

    tracing::info!("Client attached to session: {}", session.id);

    // Relay loop: broadcast → WebSocket, WebSocket → command_sender
    loop {
        tokio::select! {
            // Forward SSH data from broadcast to this WebSocket
            result = broadcast_rx.recv() => {
                match result {
                    Ok(data) => {
                        let mut socket = socket_tx.lock().await;
                        if socket.send(Message::Binary(data)).await.is_err() {
                            tracing::info!("WebSocket send failed, client disconnected");
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("Client lagged, missed {} messages", n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Broadcast channel closed, SSH session ended");
                        break;
                    }
                }
            }

            // Read from WebSocket and forward to SSH
            ws_msg = {
                let mut socket = socket_tx.lock().await;
                match tokio::time::timeout(std::time::Duration::from_millis(10), socket.next()).await {
                    Ok(Some(Ok(msg))) => Some(msg),
                    Ok(Some(Err(_))) => None,
                    Ok(None) => None,
                    Err(_) => None,
                }
            } => {
                match ws_msg {
                    Some(Message::Text(text)) => {
                        if let Ok(control) = serde_json::from_str::<WsMessage>(&text) {
                            match control {
                                WsMessage::Resize { cols, rows } => {
                                    if session.command_sender.send(SshCommand::Resize { cols: cols as u32, rows: rows as u32 }).await.is_err() {
                                        break;
                                    }
                                }
                                WsMessage::Ping { t } => {
                                    let pong = serde_json::json!({"type": "pong", "t": t});
                                    let mut socket = socket_tx.lock().await;
                                    if socket.send(Message::Text(pong.to_string())).await.is_err() {
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            if session.command_sender.send(SshCommand::Data(text.into_bytes())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Message::Binary(data)) => {
                        if session.command_sender.send(SshCommand::Data(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Message::Close(_)) | None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Client disconnected — decrement connections but DON'T kill SSH
    state.ssh_session_manager.decrement_connections(params.server_id, &params.session);
    tracing::info!("Client detached from session: {}", session.id);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles without errors (may have warnings about unused imports — that's fine)

- [ ] **Step 3: Build and test manually**

Run: `cargo build --release`
Expected: builds successfully

- [ ] **Step 4: Test the behavior**

1. Start the server: `./target/release/server-access-manager`
2. Open terminal in browser, connect to a server
3. Run some commands (e.g., `ls`, `whoami`)
4. Close the browser tab
5. Reopen the terminal page for the same server
6. Expected: terminal shows the output from step 3, session is still alive

- [ ] **Step 5: Commit**

```bash
git add src/handlers/terminal.rs
git commit -m "feat: tmux-like session persistence — SSH sessions survive browser close"
```

---

### Task 3: Update frontend to show reconnect status

**Files:**
- Modify: `src/handlers/terminal.rs` (the inline HTML in `terminal_page`)

**Interfaces:**
- Consumes: nothing new
- Produces: terminal page shows "Reconnected to existing session" message when attaching to a running session

- [ ] **Step 1: Add reconnect detection to frontend JavaScript**

In the `terminal_page` function, after the WebSocket `onopen` handler, add logic to detect if we're reconnecting:

```javascript
ws.onopen = function() {
    debugLog("WebSocket connected successfully");
    startPing();
    if (term) term.focus();
    // Check if this is a reconnect by looking for existing session indicator
    fetch("/api/sessions?server_id=" + serverId + "&session=default", { credentials: "same-origin" })
        .then(function(r) { return r.json(); })
        .then(function(data) {
            if (data.sessions && data.sessions.length > 0) {
                var s = data.sessions[0];
                if (s.active) {
                    term.write("\r\n\x1b[33m[Reconnected to existing session]\x1b[0m\r\n");
                }
            }
        })
        .catch(function(e) { /* ignore */ });
};
```

Note: The `/api/sessions` endpoint already exists via `list_sessions` handler. Check if it's routed; if not, add it to the router.

- [ ] **Step 2: Add sessions API route if missing**

Check `src/handlers/mod.rs` for a route like `/api/sessions`. If missing, add:

```rust
.route("/api/sessions", get(terminal::list_sessions))
```

- [ ] **Step 3: Verify and commit**

Run: `cargo build --release`
Expected: builds successfully

```bash
git add src/handlers/terminal.rs src/handlers/mod.rs
git commit -m "feat: show reconnect status in terminal UI"
```

---

## Summary

After these 3 tasks:
- SSH sessions persist when browser closes (tmux-like)
- Reconnecting shows the current terminal state (buffered output)
- Multiple browser tabs can attach to the same session (broadcast channel)
- Idle sessions are still cleaned up after `idle_timeout` (existing cleanup task)
- Explicit disconnect (via close_session API) still kills the SSH connection
