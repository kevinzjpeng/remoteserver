use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension, Json,
};
use futures::{SinkExt, StreamExt};
use russh::{
    client::{Config, Handler},
    ChannelMsg,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::auth::session::SessionData;
use crate::repository::ServerRepository;
use crate::session::manager::ClientHandler;
use crate::AppState;

#[derive(Deserialize)]
pub struct TerminalParams {
    server_id: i64,
    session: String,
}

#[derive(Deserialize)]
pub struct TerminalPageParams {
    server_id: i64,
    server_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsMessage {
    #[serde(rename = "resize")]
    Resize { cols: u16, rows: u16 },
    #[serde(rename = "ping")]
    Ping { t: i64 },
    #[serde(rename = "pong")]
    Pong { t: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SshCommand {
    Data(Vec<u8>),
    Resize { cols: u32, rows: u32 },
    Close,
}

#[derive(Serialize)]
pub struct SessionListResponse {
    sessions: Vec<SessionInfo>,
}

#[derive(Serialize)]
pub struct SessionInfo {
    id: String,
    name: String,
    created_at: String,
    last_active: String,
    client_count: usize,
    active: bool,
}

pub async fn terminal_page(
    Extension(_session_data): Extension<SessionData>,
    Query(params): Query<TerminalPageParams>,
) -> impl IntoResponse {
    tracing::info!("Terminal page requested: server_id={}, server_name={:?}", params.server_id, params.server_name);
    let server_name = params.server_name.unwrap_or_else(|| "SSH Terminal".to_string());
    let server_id = params.server_id;

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SSH Terminal - Server Access Manager</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/xterm@5.3.0/css/xterm.css">
    <script src="https://cdn.jsdelivr.net/npm/xterm@5.3.0/lib/xterm.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/xterm-addon-fit@0.8.0/lib/xterm-addon-fit.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/xterm-addon-web-links@0.9.0/lib/xterm-addon-web-links.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/xterm-addon-unicode-11@0.5.0/lib/xterm-addon-unicode-11.js"></script>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/tonsky/FiraCode@6.2/distr/fira_code.css">
    <style>
        body {{
            margin: 0;
            padding: 0;
            background: #1e1e1e;
            font-family: 'Fira Code', 'JetBrains Mono', 'Cascadia Code', 'SF Mono', Monaco, Menlo, Consolas, 'Ubuntu Mono', monospace;
        }}
        #header {{
            background: #2d2d2d;
            color: #fff;
            padding: 10px 20px;
            border-bottom: 1px solid #444;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        #header h1 {{
            margin: 0;
            font-size: 18px;
        }}
        #header a {{
            color: #58a6ff;
            text-decoration: none;
        }}
        #header a:hover {{
            text-decoration: underline;
        }}
        #disconnect-btn {{
            background: #dc3545;
            color: #fff;
            border: none;
            padding: 6px 10px;
            border-radius: 4px;
            cursor: pointer;
            margin-left: 10px;
            z-index: 1001;
            position: relative;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        #disconnect-btn:hover {{
            background: #c82333;
        }}
        #disconnect-btn:active {{
            background: #a02622;
        }}
        #terminal-container {{
            height: calc(100vh - 50px);
            background: #000;
            cursor: text;
            position: relative;
        }}
        #terminal-container:focus {{
            outline: none;
            box-shadow: 0 0 0 2px #28a745;
        }}
        #mobile-input {{
            position: absolute;
            top: 0;
            left: 0;
            width: 1px;
            height: 1px;
            opacity: 0;
            pointer-events: none;
        }}
        .xterm {{
            font-feature-settings: "liga" 0;
        }}
        .xterm xterm-screen {{
            font-family: 'Fira Code', 'JetBrains Mono', 'Cascadia Code', 'SF Mono', Monaco, Menlo, Consolas, 'Ubuntu Mono', monospace !important;
        }}
    </style>
</head>
<body>
    <div id="header">
        <div style="display: flex; align-items: center; gap: 10px;">
            <h1 id="server-name">{}</h1>
            <div id="ping-status" style="display: flex; align-items: center; gap: 5px; font-size: 12px; color: #888;">
                <span id="ping-dot" style="width: 8px; height: 8px; border-radius: 50%; background: #28a745;"></span>
                <span id="ping-value">--ms</span>
            </div>
        </div>
        <div style="display: flex; align-items: center;">
            <a href="/topology" title="返回拓扑图">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#58a6ff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="15 18 9 12 15 6"/>
                </svg>
            </a>
            <button id="disconnect-btn" onclick="event.stopPropagation(); disconnect()" title="Detach from session (session continues running)">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
                    <polyline points="16 17 21 12 16 7"/>
                    <line x1="21" y1="12" x2="9" y2="12"/>
                </svg>
            </button>
            <button id="close-session-btn" onclick="event.stopPropagation(); closeSession()" title="Terminate SSH session" style="background:#fd7e14;color:#fff;border:none;padding:6px 10px;border-radius:4px;cursor:pointer;margin-left:5px;z-index:1001;position:relative;display:flex;align-items:center;justify-content:center;">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"/>
                    <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
            </button>
        </div>
    </div>
    <div id="debug-panel" style="display:none; position:fixed; bottom:0; left:0; right:0; background:#1a1a1a; color:#0f0; font-family:monospace; font-size:11px; max-height:30vh; overflow-y:auto; padding:10px; z-index:9999; border-top:2px solid #333;">
        <div style="display:flex; justify-content:space-between; margin-bottom:5px;">
            <strong>Debug Console</strong>
            <button onclick="document.getElementById(&quot;debug-panel&quot;).style.display=&quot;none&quot;" style="background:#333; color:#fff; border:none; padding:2px 8px; cursor:pointer;">Close</button>
        </div>
        <div id="debug-log"></div>
    </div>
    <button onclick="document.getElementById(&quot;debug-panel&quot;).style.display=&quot;block&quot;" style="position:fixed; bottom:10px; right:10px; background:#333; color:#fff; border:none; padding:5px 10px; cursor:pointer; z-index:9998; border-radius:4px;">Show Debug</button>
    <div id="terminal-container" tabindex="0">
        <textarea id="mobile-input" autocomplete="off" autocapitalize="off" spellcheck="false"></textarea>
    </div>

    <script>
        var serverId = {};
        var serverName = "{}";

        function debugLog(msg, isError) {{
            var timestamp = new Date().toISOString().substr(11, 12);
            var prefix = isError ? "X " : "OK ";
            var logDiv = document.getElementById("debug-log");
            if (logDiv) {{
                var entry = document.createElement("div");
                entry.style.color = isError ? "#f88" : "#8f8";
                entry.textContent = "[" + timestamp + "] " + prefix + msg;
                logDiv.appendChild(entry);
                logDiv.scrollTop = logDiv.scrollHeight;
            }}
            console.log((isError ? "ERROR" : "LOG") + ": " + msg);
        }}

        function showDebug(msg, isError) {{
            debugLog(msg, isError);
            var panel = document.getElementById("debug-panel");
            if (panel) panel.style.display = "block";
        }}

        debugLog("Script started");

        // Check connectivity
        (function() {{
            debugLog("Page loaded, checking server connectivity...");
            fetch("/topology", {{ method: "HEAD", credentials: "same-origin" }})
                .then(function(r) {{
                    if (r.status === 200) {{
                        debugLog("Server connection OK, session valid");
                    }} else if (r.status === 401 || r.status === 302) {{
                        debugLog("Session invalid or expired - please login", true);
                        showDebug("Please login first!", true);
                    }} else {{
                        debugLog("Server responded with status: " + r.status);
                    }}
                }})
                .catch(function(e) {{
                    debugLog("Server connection failed: " + e.message, true);
                    showDebug("Cannot reach server!", true);
                }});
        }})();

        // Initialize terminal
        var term;
        var fitAddon;
        try {{
            debugLog("Initializing terminal...");
            term = new Terminal({{
                cursorBlink: true,
                fontSize: 14,
                fontFamily: "'Fira Code', 'JetBrains Mono', 'Cascadia Code', 'SF Mono', Monaco, Menlo, Consolas, 'Ubuntu Mono', monospace",
                theme: {{
                    background: "#000000",
                    foreground: "#ffffff",
                    cursor: "#ffffff",
                    cursorAccent: "#000000",
                    selection: "#444444"
                }},
                allowTransparency: true,
                drawBoldTextInBrightColors: false,
                minimumContrastRatio: 1
            }});

            fitAddon = new FitAddon.FitAddon();
            var webLinksAddon = new WebLinksAddon.WebLinksAddon();

            term.loadAddon(fitAddon);
            term.loadAddon(webLinksAddon);

            // Try to load Unicode11Addon if available
            try {{
                if (typeof Unicode11Addon !== "undefined") {{
                    var unicode11Addon = new Unicode11Addon.Unicode11Addon();
                    term.loadAddon(unicode11Addon);
                    term.unicode.activeVersion = "11";
                    debugLog("Unicode11 addon loaded");
                }} else {{
                    debugLog("Unicode11 addon not available, skipping");
                }}
            }} catch (unicodeErr) {{
                debugLog("Unicode11 addon failed to load: " + unicodeErr.message);
            }}

            term.open(document.getElementById("terminal-container"));
            fitAddon.fit();
            debugLog("Terminal initialized successfully");
        }} catch (e) {{
            debugLog("Failed to initialize terminal: " + e.message, true);
            showDebug("Terminal init error: " + e.message, true);
        }}

        // Disconnect function (detach - session continues running)
        function disconnect() {{
            debugLog("Detach button clicked");
            try {{
                if (window.ws && window.ws.readyState !== undefined) {{
                    if (window.ws.readyState === WebSocket.OPEN) {{
                        window.ws.close();
                    }}
                }}
                if (term) {{
                    term.write("\r\n\x1b[33mDetached from session. Session continues running.\x1b[0m\r\n");
                    term.write("\x1b[33mRefresh this page to reconnect.\x1b[0m\r\n");
                }}
                if (typeof stopPing !== "undefined") stopPing();
            }} catch (e) {{
                debugLog("Error in disconnect: " + e.message, true);
            }}
        }}

        // Close session function (terminate SSH session)
        function closeSession() {{
            if (!confirm("Terminate this SSH session? This will kill all running processes.")) {{
                return;
            }}
            debugLog("Close session button clicked");
            try {{
                // First close the WebSocket
                if (window.ws && window.ws.readyState !== undefined) {{
                    if (window.ws.readyState === WebSocket.OPEN) {{
                        window.ws.close();
                    }}
                }}
                // Then call the close session API
                fetch("/api/sessions/close?server_id=" + serverId + "&session=default", {{
                    method: "POST",
                    credentials: "same-origin"
                }})
                .then(function(r) {{
                    if (r.ok) {{
                        if (term) {{
                            term.write("\r\n\x1b[31mSession terminated.\x1b[0m\r\n");
                        }}
                    }} else {{
                        if (term) {{
                            term.write("\r\n\x1b[31mFailed to terminate session.\x1b[0m\r\n");
                        }}
                    }}
                }})
                .catch(function(e) {{
                    debugLog("Error closing session: " + e.message, true);
                }});
                if (typeof stopPing !== "undefined") stopPing();
            }} catch (e) {{
                debugLog("Error in closeSession: " + e.message, true);
            }}
        }}

        // Mobile detection
        var isMobile = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);

        // Mobile input handling
        var mobileInput = document.getElementById("mobile-input");
        var lastValue = "";

        // Focus mobile input on touch/click (mobile only)
        document.getElementById("terminal-container").addEventListener("click", function(e) {{
            if (isMobile) {{
                mobileInput.focus();
            }} else {{
                term.focus();
            }}
        }});

        // Handle input from mobile textarea
        mobileInput.addEventListener("input", function(e) {{
            var newValue = mobileInput.value;
            debugLog("Mobile input: newValue.length=" + newValue.length + ", lastValue.length=" + lastValue.length);
            if (newValue.length > lastValue.length) {{
                var added = newValue.substring(lastValue.length);
                debugLog("Mobile input added: " + JSON.stringify(added));
                if (ws && ws.readyState === WebSocket.OPEN) {{
                    ws.send(added);
                    debugLog("Sent mobile input via WebSocket");
                }} else {{
                    debugLog("WebSocket not ready for mobile input, readyState=" + (ws ? ws.readyState : "null"), true);
                }}
            }} else if (newValue.length < lastValue.length) {{
                var deleted = lastValue.length - newValue.length;
                debugLog("Mobile input deleted: " + deleted + " chars");
                for (var i = 0; i < deleted; i++) {{
                    if (ws && ws.readyState === WebSocket.OPEN) {{
                        ws.send("\x08");
                    }}
                }}
            }}
            lastValue = newValue;
        }});

        // Handle Enter key on mobile
        mobileInput.addEventListener("keydown", function(e) {{
            if (e.key === "Enter") {{
                e.preventDefault();
                debugLog("Mobile Enter key pressed");
                if (ws && ws.readyState === WebSocket.OPEN) {{
                    ws.send("\r");
                    debugLog("Sent Enter via WebSocket");
                }} else {{
                    debugLog("WebSocket not ready for Enter, readyState=" + (ws ? ws.readyState : "null"), true);
                }}
                mobileInput.value = "";
                lastValue = "";
            }}
        }});

        // Force focus after a short delay
        setTimeout(function() {{
            if (isMobile) {{
                mobileInput.focus();
            }} else if (term) {{
                term.focus();
            }}
        }}, 100);

        // Connect to WebSocket
        debugLog("About to connect to WebSocket...");
        var wsProtocol = window.location.protocol === "https:" ? "wss:" : "ws:";
        var wsUrl = wsProtocol + "//" + window.location.host + "/ws/terminal?server_id=" + serverId + "&session=default";
        debugLog("Connecting to WebSocket: " + wsUrl);

        var ws;
        try {{
            ws = new WebSocket(wsUrl);
            debugLog("WebSocket object created");
        }} catch (e) {{
            debugLog("Failed to create WebSocket: " + e.message, true);
            showDebug("WebSocket creation failed: " + e.message, true);
            throw e;
        }}

        window.ws = ws;

        var pingDot = document.getElementById("ping-dot");
        var pingValue = document.getElementById("ping-value");
        var pingInterval;

        function updatePing(connected) {{
            if (connected) {{
                pingDot.style.background = "#28a745";
                var startTime = Date.now();
                ws.send(JSON.stringify({{ type: "ping", t: startTime }}));
            }} else {{
                pingDot.style.background = "#dc3545";
                pingValue.textContent = "--ms";
            }}
        }}

        function startPing() {{
            updatePing(true);
            pingInterval = setInterval(function() {{ updatePing(true); }}, 5000);
        }}

        function stopPing() {{
            if (pingInterval) clearInterval(pingInterval);
            pingDot.style.background = "#dc3545";
            pingValue.textContent = "--ms";
        }}

        ws.onopen = function() {{
            debugLog("WebSocket connected successfully, readyState: " + ws.readyState);
            startPing();
            if (term) {{
                debugLog("Focusing terminal");
                term.focus();
            }}
            // Check if this is a reconnect to an existing session
            debugLog("Checking for existing session...");
            fetch("/api/sessions?server_id=" + serverId + "&session=default", {{ credentials: "same-origin" }})
                .then(function(r) {{
                    debugLog("Session API response status: " + r.status);
                    return r.json();
                }})
                .then(function(data) {{
                    debugLog("Session API response: " + JSON.stringify(data));
                    if (data.sessions && data.sessions.length > 0) {{
                        var s = data.sessions[0];
                        debugLog("Found session: " + JSON.stringify(s));
                        if (s.client_count > 1 || (s.client_count === 1 && s.active)) {{
                            debugLog("This is a reconnect, showing message");
                            term.write("\r\n\x1b[33m[Reconnected to existing session]\x1b[0m\r\n");
                        }} else {{
                            debugLog("Session exists but not active or only 1 client");
                        }}
                    }} else {{
                        debugLog("No sessions found");
                    }}
                }})
                .catch(function(e) {{
                    debugLog("Session check failed: " + e.message, true);
                }});
        }};

        ws.onmessage = function(event) {{
            debugLog("WebSocket message received, type: " + (event.data instanceof Blob ? "Blob" : typeof event.data) + ", size: " + (event.data instanceof Blob ? event.data.size : event.data.length));
            if (event.data instanceof Blob) {{
                event.data.text().then(function(text) {{
                    debugLog("Blob converted to text, length: " + text.length);
                    // Check if it is a pong response
                    if (text.indexOf("pong") !== -1) {{
                        try {{
                            var msg = JSON.parse(text);
                            if (msg.type === "pong" && msg.t) {{
                                var ping = Date.now() - msg.t;
                                pingValue.textContent = ping + "ms";
                                if (ping < 100) {{
                                    pingDot.style.background = "#28a745";
                                }} else if (ping < 300) {{
                                    pingDot.style.background = "#ffc107";
                                }} else {{
                                    pingDot.style.background = "#dc3545";
                                }}
                            }}
                        }} catch (e) {{}}
                        return;
                    }}
                    if (term) {{
                        debugLog("Writing " + text.length + " chars to terminal");
                        term.write(text);
                    }}
                }});
            }} else {{
                // Check if it is a pong response
                if (event.data.indexOf("pong") !== -1) {{
                    try {{
                        var msg = JSON.parse(event.data);
                        if (msg.type === "pong" && msg.t) {{
                            var ping = Date.now() - msg.t;
                            pingValue.textContent = ping + "ms";
                            if (ping < 100) {{
                                pingDot.style.background = "#28a745";
                            }} else if (ping < 300) {{
                                pingDot.style.background = "#ffc107";
                            }} else {{
                                pingDot.style.background = "#dc3545";
                            }}
                        }}
                    }} catch (e) {{}}
                    return;
                }}
                if (term) {{
                    debugLog("Writing text to terminal, length: " + event.data.length);
                    term.write(event.data);
                }}
            }}
        }};

        ws.onerror = function(error) {{
            debugLog("WebSocket error occurred", true);
            stopPing();
            if (term) term.write("\r\n\x1b[31mWebSocket error occurred\x1b[0m\r\n");
        }};

        ws.onclose = function(event) {{
            debugLog("WebSocket closed, code: " + event.code + ", reason: " + event.reason);
            stopPing();
            if (term) term.write("\r\n\x1b[31mConnection closed\x1b[0m\r\n");
        }};

        // Send terminal input to WebSocket
        if (term) {{
            debugLog("Setting up term.onData handler");
            term.onData(function(data) {{
                debugLog("term.onData called, data: " + JSON.stringify(data) + ", ws.readyState: " + (ws ? ws.readyState : "null"));
                if (ws && ws.readyState === WebSocket.OPEN) {{
                    ws.send(data);
                    debugLog("Sent data via WebSocket");
                }} else {{
                    debugLog("WebSocket not ready for term.onData, readyState=" + (ws ? ws.readyState : "null"), true);
                }}
            }});
        }}

        // Handle window resize
        window.addEventListener("resize", function() {{
            if (fitAddon) fitAddon.fit();
            if (ws && ws.readyState === WebSocket.OPEN && term) {{
                ws.send(JSON.stringify({{
                    type: "resize",
                    cols: term.cols,
                    rows: term.rows
                }}));
                debugLog("Sent resize: " + term.cols + "x" + term.rows);
            }}
        }});
    </script>
</body>
</html>"##,
        server_name,
        server_id,
        server_name,
    );

    Html(html)
}

pub async fn terminal_ws(
    State(state): State<AppState>,
    Extension(session_data): Extension<SessionData>,
    ws: WebSocketUpgrade,
    Query(params): Query<TerminalParams>,
) -> impl IntoResponse {
    let state = state.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, state, session_data, params))
}

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

        // Send initial status message
        {
            let mut socket = socket_tx.lock().await;
            let _ = socket.send(Message::Text("WebSocket connected, initializing...".to_string())).await;
        }

        // Get server details
        tracing::info!("Getting server details for ID: {}", params.server_id);
        let server_repo = ServerRepository::new(state.db.clone(), state.config.encryption_key.as_bytes().to_vec());
        let server = match server_repo.get_by_id(params.server_id) {
            Ok(s) => {
                tracing::info!("Server found: {} ({}:{})", s.name, s.hostname, s.port);
                s
            },
            Err(e) => {
                tracing::error!("Failed to get server: {}", e);
                let mut socket = socket_tx.lock().await;
                let _ = socket.send(Message::Text(format!("Error: Server not found: {}", e))).await;
                return;
            }
        };

        tracing::info!("Connecting to server: {} ({}:{})", server.name, server.hostname, server.port);

        // Get credentials
        tracing::info!("Getting credentials for server {} (auth_type: {})", params.server_id, server.auth_type);
        let password = if server.auth_type == "password" {
            tracing::info!("Retrieving password for server {}", params.server_id);
            match server_repo.get_password(params.server_id) {
                Ok(p) => {
                    tracing::info!("Password retrieved successfully (length: {})", p.len());
                    Some(p)
                },
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

        // Create SSH config
        let config = Arc::new(Config {
            inactivity_timeout: None, // No timeout - keep connection alive for session persistence
            ..Config::default()
        });

        // Create SSH client handler
        let ssh_handler = ClientHandler;

        // Connect to SSH server
        let addr = format!("{}:{}", server.hostname, server.port);
        tracing::info!("Connecting to SSH at: {}", addr);
        let mut ssh_client = match russh::client::connect(config, &addr, ssh_handler).await {
            Ok(client) => {
                tracing::info!("SSH TCP connection established to {}", addr);
                client
            },
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
            Ok(true) => {
                tracing::info!("SSH authentication successful");
            }
            Ok(false) => {
                tracing::error!("SSH authentication failed - credentials rejected");
                let mut socket = socket_tx.lock().await;
                let _ = socket.send(Message::Text("Error: SSH authentication failed - credentials rejected".to_string())).await;
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
        tracing::info!("Opening SSH channel");
        let mut channel = match ssh_client.channel_open_session().await {
            Ok(ch) => {
                tracing::info!("SSH channel opened successfully");
                ch
            },
            Err(e) => {
                tracing::error!("Failed to open SSH channel: {}", e);
                let mut socket = socket_tx.lock().await;
                let _ = socket.send(Message::Text(format!("Error: Failed to open session: {}", e))).await;
                return;
            }
        };

        // Request PTY
        tracing::info!("Requesting PTY");
        if let Err(e) = channel.request_pty(false, "xterm-256color", 80, 40, 0, 0, &[]).await {
            tracing::error!("Failed to request PTY: {}", e);
            let mut socket = socket_tx.lock().await;
            let _ = socket.send(Message::Text(format!("Error: Failed to request PTY: {}", e))).await;
            return;
        }
        tracing::info!("PTY requested successfully");

        // Request shell
        tracing::info!("Requesting shell");
        if let Err(e) = channel.request_shell(true).await {
            tracing::error!("Failed to start shell: {}", e);
            let mut socket = socket_tx.lock().await;
            let _ = socket.send(Message::Text(format!("Error: Failed to start shell: {}", e))).await;
            return;
        }
        tracing::info!("Shell requested successfully");

        // Create channels for SSH I/O
        let (ssh_cmd_tx, mut ssh_cmd_rx) = tokio::sync::mpsc::channel::<SshCommand>(100);
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
                                tracing::debug!("Received {} bytes from SSH", data.len());
                                // Append to output buffer
                                session_mgr.append_output(server_id, &session_name, &data);
                                // Broadcast to all connected WebSocket clients
                                let _ = broadcast_tx_clone.send(data.to_vec());
                            }
                            Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => {
                                tracing::info!("SSH channel closed, removing session from manager");
                                // Remove session from manager so reconnecting clients create a new session
                                let _ = session_mgr.close_session(server_id, &session_name);
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(cmd) = ssh_cmd_rx.recv() => {
                        match cmd {
                            SshCommand::Data(data) => {
                                tracing::debug!("Sending {} bytes to SSH", data.len());
                                if channel.data(data.as_slice()).await.is_err() {
                                    tracing::error!("Failed to send data to SSH");
                                    break;
                                }
                            }
                            SshCommand::Resize { cols, rows } => {
                                tracing::info!("Resizing terminal to {}x{}", cols, rows);
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
        // Handle race condition: if another task created this session concurrently,
        // use the existing session and clean up our unused SSH connection
        let ssh_cmd_tx_clone = ssh_cmd_tx.clone();
        let session = match state.ssh_session_manager.store_session(
            params.server_id,
            &params.session,
            ssh_client,
            ssh_cmd_tx,
            broadcast_tx,
        ) {
            Ok(new_session) => {
                tracing::info!("Created new session: {}", new_session.id);
                new_session
            }
            Err(existing_session) => {
                // Race condition: another task created this session first.
                // Clean up our unused SSH connection by sending Close command.
                tracing::warn!(
                    "Session {} already exists (race condition), using existing session",
                    existing_session.id
                );
                // The SSH I/O task we spawned will exit when it receives Close
                let _ = ssh_cmd_tx_clone.try_send(SshCommand::Close);
                existing_session
            }
        };

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
        match socket.send(Message::Binary(buffer)).await {
            Ok(_) => tracing::info!("Buffer sent successfully"),
            Err(e) => tracing::error!("Failed to send buffer: {:?}", e),
        }
    }

    // Increment connection count
    state.ssh_session_manager.increment_connections(params.server_id, &params.session);

    tracing::info!("Client attached to session: {}, command_sender available: {}", session.id, session.command_sender.capacity());

    // Relay loop: broadcast → WebSocket, WebSocket → command_sender
    // Use the same polling pattern as the original code but with broadcast channel
    tracing::info!("Starting relay loop for session {}", session.id);
    let mut loop_count = 0;
    loop {
        loop_count += 1;
        if loop_count % 1000 == 0 {
            tracing::info!("Relay loop iteration {} for session {}", loop_count, session.id);
        }

        // 1. Forward SSH data from broadcast to WebSocket (non-blocking)
        while let Ok(data) = broadcast_rx.try_recv() {
            tracing::info!("Forwarding {} bytes from broadcast to WebSocket", data.len());
            let mut socket = socket_tx.lock().await;
            if socket.send(Message::Binary(data)).await.is_err() {
                tracing::info!("WebSocket send failed, client disconnected");
                break;
            }
        }

        // 2. Read from WebSocket (non-blocking with short timeout)
        let ws_msg = {
            let mut socket = socket_tx.lock().await;
            match tokio::time::timeout(std::time::Duration::from_millis(10), socket.next()).await {
                Ok(Some(Ok(msg))) => {
                    tracing::info!("Received WebSocket message: {:?}", msg);
                    Some(msg)
                }
                Ok(Some(Err(e))) => {
                    tracing::info!("WebSocket read error: {:?}, breaking", e);
                    break;
                }
                Ok(None) => {
                    tracing::info!("WebSocket closed (None), breaking");
                    break;
                }
                Err(_) => None,
            }
        };

        if let Some(msg) = ws_msg {
            match msg {
                Message::Text(text) => {
                    tracing::info!("Received Text message: {} bytes", text.len());
                    if let Ok(control) = serde_json::from_str::<WsMessage>(&text) {
                        match control {
                            WsMessage::Resize { cols, rows } => {
                                tracing::info!("Resize to {}x{}", cols, rows);
                                if session.command_sender.send(SshCommand::Resize { cols: cols as u32, rows: rows as u32 }).await.is_err() {
                                    tracing::error!("Failed to send resize command");
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
                        tracing::info!("Sending {} bytes of text to SSH", text.len());
                        if session.command_sender.send(SshCommand::Data(text.into_bytes())).await.is_err() {
                            tracing::error!("Failed to send text data to SSH");
                            break;
                        }
                    }
                }
                Message::Binary(data) => {
                    tracing::info!("Sending {} bytes of binary data to SSH", data.len());
                    if session.command_sender.send(SshCommand::Data(data)).await.is_err() {
                        tracing::error!("Failed to send binary data to SSH");
                        break;
                    }
                }
                Message::Close(_) => {
                    tracing::info!("Received WebSocket close message");
                    break;
                }
                _ => {}
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }

    // Client disconnected — decrement connections but DON'T kill SSH
    state.ssh_session_manager.decrement_connections(params.server_id, &params.session);
    tracing::info!("Client detached from session: {}", session.id);
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(_session_data): Extension<SessionData>,
    Query(params): Query<TerminalParams>,
) -> impl IntoResponse {
    let sessions = state.ssh_session_manager.list_sessions(params.server_id);

    let response = SessionListResponse {
        sessions: sessions
            .into_iter()
            .map(|s| SessionInfo {
                id: s.id,
                name: s.name,
                created_at: s.created_at.to_rfc3339(),
                last_active: s.last_active.to_rfc3339(),
                client_count: s.client_count,
                active: s.active,
            })
            .collect(),
    };

    Json(response)
}

pub async fn close_session(
    State(state): State<AppState>,
    Extension(_session_data): Extension<SessionData>,
    Query(params): Query<TerminalParams>,
) -> Result<Response, StatusCode> {
    state
        .ssh_session_manager
        .close_session(params.server_id, &params.session)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK.into_response())
}
