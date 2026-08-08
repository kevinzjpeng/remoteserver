package handlers

import (
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
	servers *repository.ServerRepository
}

func NewTerminalHandler(servers *repository.ServerRepository) *TerminalHandler {
	return &TerminalHandler{servers: servers}
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

	// Get server details
	server, err := h.servers.GetByID(serverID)
	if err != nil {
		http.Error(w, "Server not found", http.StatusNotFound)
		return
	}

	// Get password if auth type is password
	var password string
	if server.AuthType == "password" {
		password, err = h.servers.GetPassword(serverID)
		if err != nil {
			log.Printf("Failed to get password: %v", err)
			http.Error(w, "Failed to get credentials", http.StatusInternalServerError)
			return
		}
	}

	// Get private key if auth type is key
	var privateKey string
	if server.AuthType == "key" {
		privateKey, err = h.servers.GetPrivateKey(serverID)
		if err != nil {
			log.Printf("Failed to get private key: %v", err)
			http.Error(w, "Failed to get credentials", http.StatusInternalServerError)
			return
		}
	}

	// Upgrade to WebSocket
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("WebSocket upgrade failed: %v", err)
		return
	}
	defer conn.Close()

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
			log.Printf("Failed to parse private key: %v", err)
			conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("Error: Failed to parse private key: %v", err)))
			return
		}
		config.Auth = append(config.Auth, ssh.PublicKeys(signer))
	}

	// Connect to SSH server
	addr := fmt.Sprintf("%s:%d", server.Hostname, server.Port)
	sshClient, err := ssh.Dial("tcp", addr, config)
	if err != nil {
		log.Printf("SSH connection failed: %v", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("Error: SSH connection failed: %v", err)))
		return
	}
	defer sshClient.Close()

	// Create SSH session
	session, err := sshClient.NewSession()
	if err != nil {
		log.Printf("Failed to create SSH session: %v", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("Error: Failed to create session: %v", err)))
		return
	}
	defer session.Close()

	// Set up terminal modes
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}

	// Request pseudo terminal
	if err := session.RequestPty("xterm", 80, 40, modes); err != nil {
		log.Printf("Failed to request PTY: %v", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("Error: Failed to request PTY: %v", err)))
		return
	}

	// Set up pipes
	stdin, err := session.StdinPipe()
	if err != nil {
		log.Printf("Failed to create stdin pipe: %v", err)
		return
	}

	stdout, err := session.StdoutPipe()
	if err != nil {
		log.Printf("Failed to create stdout pipe: %v", err)
		return
	}

	stderr, err := session.StderrPipe()
	if err != nil {
		log.Printf("Failed to create stderr pipe: %v", err)
		return
	}

	// Start shell
	if err := session.Shell(); err != nil {
		log.Printf("Failed to start shell: %v", err)
		conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("Error: Failed to start shell: %v", err)))
		return
	}

	// Send success message
	conn.WriteMessage(websocket.TextMessage, []byte("Connected to "+server.Name+"\r\n"))

	// Goroutine to read from SSH and send to WebSocket
	var wg sync.WaitGroup
	wg.Add(2)

	go func() {
		defer wg.Done()
		io.Copy(&websocketWriter{conn: conn}, stdout)
	}()

	go func() {
		defer wg.Done()
		io.Copy(&websocketWriter{conn: conn}, stderr)
	}()

	// Goroutine to read from WebSocket and send to SSH
	go func() {
		for {
			_, message, err := conn.ReadMessage()
			if err != nil {
				log.Printf("WebSocket read error: %v", err)
				break
			}
			stdin.Write(message)
		}
	}()

	// Wait for SSH session to complete
	session.Wait()
	wg.Wait()
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
