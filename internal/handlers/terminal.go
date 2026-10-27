package handlers

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"io"
	"log"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"

	"server-access-manager/internal/repository"
)

type TerminalHandler struct {
	servers        *repository.ServerRepository
	sessionManager *SessionManager
}

func NewTerminalHandler(servers *repository.ServerRepository) *TerminalHandler {
	return &TerminalHandler{
		servers:        servers,
		sessionManager: NewSessionManager(),
	}
}

func generateSessionID() string {
	bytes := make([]byte, 16)
	rand.Read(bytes)
	return hex.EncodeToString(bytes)
}

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true // Allow all origins for development
	},
}

func (h *TerminalHandler) Terminal(w http.ResponseWriter, r *http.Request) {
	serverIDStr := r.URL.Query().Get("server_id")
	serverID, err := strconv.ParseInt(serverIDStr, 10, 64)
	if err != nil {
		http.Error(w, "Invalid server ID", http.StatusBadRequest)
		return
	}

	// Get or generate session ID
	sessionID := r.URL.Query().Get("session_id")
	if sessionID == "" {
		sessionID = generateSessionID()
	}

	// Get server details
	server, err := h.servers.GetByID(serverID)
	if err != nil {
		http.Error(w, "Server not found", http.StatusNotFound)
		return
	}

	log.Printf("Terminal connection attempt: server=%s, session=%s, auth_type=%s, username=%s",
		server.Name, sessionID, server.AuthType, server.Username)

	// Upgrade to WebSocket
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("WebSocket upgrade failed: %v", err)
		return
	}
	defer conn.Close()

	// Send session ID to client
	conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("__SESSION_ID__:%s\n", sessionID)))

	// Check if session already exists
	termSession, exists := h.sessionManager.GetSession(sessionID)
	if exists && !termSession.IsClosed() {
		log.Printf("Reattaching to existing session: %s", sessionID)
		conn.WriteMessage(websocket.TextMessage, []byte("Reconnected to existing session\r\n"))
		h.reattachToSession(conn, termSession)
		return
	}

	// Create new SSH session
	termSession, err = h.createSSHSession(serverID, server)
	if err != nil {
		log.Printf("Failed to create SSH session: %v", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("Error: %v", err)))
		return
	}

	// Store session
	h.sessionManager.CreateSession(sessionID, serverID, termSession.SSHClient, termSession.Session,
		termSession.Stdin, termSession.Stdout, termSession.Stderr)

	// Send success message
	conn.WriteMessage(websocket.TextMessage, []byte("Connected to "+server.Name+"\r\n"))

	// Handle the session
	h.handleSession(conn, termSession)
}

func (h *TerminalHandler) createSSHSession(serverID int64, server *repository.Server) (*TerminalSession, error) {
	// Get password if auth type is password
	var password string
	if server.AuthType == "password" {
		var err error
		password, err = h.servers.GetPassword(serverID)
		if err != nil {
			return nil, fmt.Errorf("failed to get credentials: %v", err)
		}
	}

	// Get private key if auth type is key
	var privateKey string
	if server.AuthType == "key" {
		var err error
		privateKey, err = h.servers.GetPrivateKey(serverID)
		if err != nil {
			return nil, fmt.Errorf("failed to get credentials: %v", err)
		}
	}

	// Build SSH config
	config := &ssh.ClientConfig{
		User: server.Username,
		Auth: []ssh.AuthMethod{},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         10 * time.Second,
	}

	// Add authentication method
	if server.AuthType == "password" && password != "" {
		config.Auth = append(config.Auth, ssh.Password(password))
	} else if server.AuthType == "key" && privateKey != "" {
		signer, err := ssh.ParsePrivateKey([]byte(privateKey))
		if err != nil {
			return nil, fmt.Errorf("failed to parse private key: %v", err)
		}
		config.Auth = append(config.Auth, ssh.PublicKeys(signer))
	}

	// Connect to SSH server
	addr := fmt.Sprintf("%s:%d", server.Hostname, server.Port)
	sshClient, err := ssh.Dial("tcp", addr, config)
	if err != nil {
		return nil, fmt.Errorf("SSH connection failed: %v", err)
	}

	// Create SSH session
	session, err := sshClient.NewSession()
	if err != nil {
		sshClient.Close()
		return nil, fmt.Errorf("failed to create session: %v", err)
	}

	// Set up terminal modes
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}

	// Request pseudo terminal
	if err := session.RequestPty("xterm", 80, 40, modes); err != nil {
		session.Close()
		sshClient.Close()
		return nil, fmt.Errorf("failed to request PTY: %v", err)
	}

	// Set up pipes
	stdin, err := session.StdinPipe()
	if err != nil {
		session.Close()
		sshClient.Close()
		return nil, fmt.Errorf("failed to create stdin pipe: %v", err)
	}

	stdout, err := session.StdoutPipe()
	if err != nil {
		session.Close()
		sshClient.Close()
		return nil, fmt.Errorf("failed to create stdout pipe: %v", err)
	}

	stderr, err := session.StderrPipe()
	if err != nil {
		session.Close()
		sshClient.Close()
		return nil, fmt.Errorf("failed to create stderr pipe: %v", err)
	}

	// Start shell
	if err := session.Shell(); err != nil {
		session.Close()
		sshClient.Close()
		return nil, fmt.Errorf("failed to start shell: %v", err)
	}

	return &TerminalSession{
		SSHClient: sshClient,
		Session:   session,
		Stdin:     stdin,
		Stdout:    stdout,
		Stderr:    stderr,
	}, nil
}

func (h *TerminalHandler) handleSession(conn *websocket.Conn, termSession *TerminalSession) {
	var wg sync.WaitGroup
	wg.Add(2)

	// Goroutine to read from SSH stdout and send to WebSocket
	go func() {
		defer wg.Done()
		io.Copy(&websocketWriter{conn: conn}, termSession.Stdout)
	}()

	// Goroutine to read from SSH stderr and send to WebSocket
	go func() {
		defer wg.Done()
		io.Copy(&websocketWriter{conn: conn}, termSession.Stderr)
	}()

	// Goroutine to read from WebSocket and send to SSH
	go func() {
		for {
			_, message, err := conn.ReadMessage()
			if err != nil {
				log.Printf("WebSocket read error (session stays alive): %v", err)
				break
			}
			termSession.WriteStdin(message)
		}
	}()

	// Wait for SSH session to complete (not WebSocket close)
	termSession.Wait()
	log.Printf("SSH session ended: %s", termSession.ID)
	h.sessionManager.RemoveSession(termSession.ID)
}

func (h *TerminalHandler) reattachToSession(conn *websocket.Conn, termSession *TerminalSession) {
	var wg sync.WaitGroup
	wg.Add(2)

	// Goroutine to read from SSH stdout and send to WebSocket
	go func() {
		defer wg.Done()
		io.Copy(&websocketWriter{conn: conn}, termSession.Stdout)
	}()

	// Goroutine to read from SSH stderr and send to WebSocket
	go func() {
		defer wg.Done()
		io.Copy(&websocketWriter{conn: conn}, termSession.Stderr)
	}()

	// Goroutine to read from WebSocket and send to SSH
	go func() {
		for {
			_, message, err := conn.ReadMessage()
			if err != nil {
				log.Printf("WebSocket read error (session stays alive): %v", err)
				break
			}
			termSession.WriteStdin(message)
		}
	}()

	// Wait for SSH session to complete (not WebSocket close)
	termSession.Wait()
	log.Printf("SSH session ended: %s", termSession.ID)
	h.sessionManager.RemoveSession(termSession.ID)
}

type websocketWriter struct {
	conn *websocket.Conn
}

func (w *websocketWriter) Write(p []byte) (n int, err error) {
	err = w.conn.WriteMessage(websocket.TextMessage, p)
	if err != nil {
		return 0, err
	}
	return len(p), nil
}
