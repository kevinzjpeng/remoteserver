package handlers

import (
	"io"
	"log"
	"sync"
	"time"

	"golang.org/x/crypto/ssh"
)

type TerminalSession struct {
	ID          string
	ServerID    int64
	SSHClient   *ssh.Client
	Session     *ssh.Session
	Stdin       io.WriteCloser
	Stdout      io.Reader
	Stderr      io.Reader
	CreatedAt   time.Time
	LastActive  time.Time
	mu          sync.RWMutex
	closed      bool
	closeChan   chan struct{}
}

type SessionManager struct {
	sessions map[string]*TerminalSession
	mu       sync.RWMutex
}

func NewSessionManager() *SessionManager {
	sm := &SessionManager{
		sessions: make(map[string]*TerminalSession),
	}
	// Start cleanup goroutine
	go sm.cleanupInactiveSessions()
	return sm
}

func (sm *SessionManager) GetSession(sessionID string) (*TerminalSession, bool) {
	sm.mu.RLock()
	defer sm.mu.RUnlock()
	session, exists := sm.sessions[sessionID]
	return session, exists
}

func (sm *SessionManager) CreateSession(sessionID string, serverID int64, sshClient *ssh.Client, session *ssh.Session, stdin io.WriteCloser, stdout io.Reader, stderr io.Reader) *TerminalSession {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	ts := &TerminalSession{
		ID:         sessionID,
		ServerID:   serverID,
		SSHClient:  sshClient,
		Session:    session,
		Stdin:      stdin,
		Stdout:     stdout,
		Stderr:     stderr,
		CreatedAt:  time.Now(),
		LastActive: time.Now(),
		closeChan:  make(chan struct{}),
	}

	sm.sessions[sessionID] = ts
	log.Printf("Created terminal session: %s for server %d", sessionID, serverID)
	return ts
}

func (sm *SessionManager) RemoveSession(sessionID string) {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	if session, exists := sm.sessions[sessionID]; exists {
		session.Close()
		delete(sm.sessions, sessionID)
		log.Printf("Removed terminal session: %s", sessionID)
	}
}

func (sm *SessionManager) UpdateLastActive(sessionID string) {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	if session, exists := sm.sessions[sessionID]; exists {
		session.mu.Lock()
		session.LastActive = time.Now()
		session.mu.Unlock()
	}
}

func (sm *SessionManager) cleanupInactiveSessions() {
	ticker := time.NewTicker(1 * time.Minute)
	defer ticker.Stop()

	for range ticker.C {
		sm.mu.Lock()
		now := time.Now()
		for sessionID, session := range sm.sessions {
			session.mu.RLock()
			inactive := now.Sub(session.LastActive) > 30*time.Minute // 30 minutes timeout
			session.mu.RUnlock()

			if inactive {
				log.Printf("Cleaning up inactive session: %s (inactive for %v)", sessionID, inactive)
				session.Close()
				delete(sm.sessions, sessionID)
			}
		}
		sm.mu.Unlock()
	}
}

func (ts *TerminalSession) Close() {
	ts.mu.Lock()
	defer ts.mu.Unlock()

	if ts.closed {
		return
	}

	ts.closed = true
	close(ts.closeChan)

	if ts.Session != nil {
		ts.Session.Close()
	}
	if ts.SSHClient != nil {
		ts.SSHClient.Close()
	}
	log.Printf("Closed terminal session: %s", ts.ID)
}

func (ts *TerminalSession) IsClosed() bool {
	ts.mu.RLock()
	defer ts.mu.RUnlock()
	return ts.closed
}

func (ts *TerminalSession) Wait() error {
	return ts.Session.Wait()
}

func (ts *TerminalSession) WriteStdin(data []byte) (int, error) {
	ts.mu.Lock()
	ts.LastActive = time.Now()
	ts.mu.Unlock()
	return ts.Stdin.Write(data)
}
