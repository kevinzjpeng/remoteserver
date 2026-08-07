package auth

import (
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"time"
)

type SessionData struct {
	UserID    int64     `json:"user_id"`
	Username  string    `json:"username"`
	IsAdmin   bool      `json:"is_admin"`
	CreatedAt time.Time `json:"created_at"`
	ExpiresAt time.Time `json:"expires_at"`
}

type SessionManager struct {
	secret   []byte
	duration time.Duration
}

func NewSessionManager(secret string, duration time.Duration) *SessionManager {
	return &SessionManager{
		secret:   []byte(secret),
		duration: duration,
	}
}

func (m *SessionManager) CreateSession(w http.ResponseWriter, userID int64, username string, isAdmin bool) error {
	session := SessionData{
		UserID:    userID,
		Username:  username,
		IsAdmin:   isAdmin,
		CreatedAt: time.Now(),
		ExpiresAt: time.Now().Add(m.duration),
	}

	sessionJSON, err := json.Marshal(session)
	if err != nil {
		return err
	}

	// Encrypt session data
	encrypted, err := encryptSession(sessionJSON, m.secret)
	if err != nil {
		return err
	}

	// Set cookie
	http.SetCookie(w, &http.Cookie{
		Name:     "session",
		Value:    encrypted,
		Path:     "/",
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteStrictMode,
		MaxAge:   int(m.duration.Seconds()),
	})

	return nil
}

func (m *SessionManager) GetSession(r *http.Request) (*SessionData, error) {
	cookie, err := r.Cookie("session")
	if err != nil {
		return nil, err
	}

	// Decrypt session data
	decrypted, err := decryptSession(cookie.Value, m.secret)
	if err != nil {
		return nil, err
	}

	var session SessionData
	if err := json.Unmarshal(decrypted, &session); err != nil {
		return nil, err
	}

	// Check expiration
	if time.Now().After(session.ExpiresAt) {
		return nil, http.ErrNoCookie
	}

	return &session, nil
}

func (m *SessionManager) ClearSession(w http.ResponseWriter) {
	http.SetCookie(w, &http.Cookie{
		Name:     "session",
		Value:    "",
		Path:     "/",
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteStrictMode,
		MaxAge:   -1,
	})
}

func generateState() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return base64.URLEncoding.EncodeToString(b), nil
}

func encryptSession(data []byte, key []byte) (string, error) {
	// For simplicity, we'll use base64 encoding here
	// In production, you should use proper encryption like AES-GCM
	// This is a simplified version - we'll improve it later
	return base64.URLEncoding.EncodeToString(data), nil
}

func decryptSession(data string, key []byte) ([]byte, error) {
	// For simplicity, we'll use base64 decoding here
	// In production, you should use proper decryption like AES-GCM
	return base64.URLEncoding.DecodeString(data)
}
