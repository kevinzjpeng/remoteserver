package handlers

import (
	"crypto/rand"
	"encoding/base64"
	"log"
	"net/http"

	"server-access-manager/internal/auth"
	"server-access-manager/internal/repository"
)

type AuthHandler struct {
	githubOAuth *auth.GitHubOAuthClient
	sessions    *auth.SessionManager
	users       *repository.UserRepository
}

func NewAuthHandler(githubOAuth *auth.GitHubOAuthClient, sessions *auth.SessionManager, users *repository.UserRepository) *AuthHandler {
	return &AuthHandler{
		githubOAuth: githubOAuth,
		sessions:    sessions,
		users:       users,
	}
}

func (h *AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	// Generate state for CSRF protection
	state, err := generateState()
	if err != nil {
		http.Error(w, "Failed to generate state", http.StatusInternalServerError)
		return
	}

	// Store state in cookie
	http.SetCookie(w, &http.Cookie{
		Name:     "oauth_state",
		Value:    state,
		Path:     "/",
		HttpOnly: true,
		Secure:   false, // Set to true in production with HTTPS
		SameSite: http.SameSiteStrictMode,
		MaxAge:   600, // 10 minutes
	})

	// Redirect to GitHub
	authURL := h.githubOAuth.GetAuthURL(state)
	http.Redirect(w, r, authURL, http.StatusFound)
}

func (h *AuthHandler) Callback(w http.ResponseWriter, r *http.Request) {
	// Verify state
	stateCookie, err := r.Cookie("oauth_state")
	if err != nil {
		http.Error(w, "Missing state cookie", http.StatusBadRequest)
		return
	}

	state := r.URL.Query().Get("state")
	if state != stateCookie.Value {
		http.Error(w, "Invalid state", http.StatusBadRequest)
		return
	}

	// Clear state cookie
	http.SetCookie(w, &http.Cookie{
		Name:     "oauth_state",
		Value:    "",
		Path:     "/",
		HttpOnly: true,
		Secure:   false, // Set to true in production with HTTPS
		SameSite: http.SameSiteStrictMode,
		MaxAge:   -1,
	})

	// Get code from query
	code := r.URL.Query().Get("code")
	if code == "" {
		http.Error(w, "Missing code", http.StatusBadRequest)
		return
	}

	// Exchange code for access token
	accessToken, err := h.githubOAuth.ExchangeCode(code)
	if err != nil {
		log.Printf("Failed to exchange code: %v", err)
		http.Error(w, "Failed to authenticate with GitHub", http.StatusInternalServerError)
		return
	}

	// Get user info from GitHub
	githubUser, err := h.githubOAuth.GetUser(accessToken)
	if err != nil {
		log.Printf("Failed to get GitHub user: %v", err)
		http.Error(w, "Failed to get user info", http.StatusInternalServerError)
		return
	}

	// Create or get user
	user, err := h.users.CreateByGithub(
		string(rune(githubUser.ID)),
		githubUser.Login,
		githubUser.Email,
		githubUser.AvatarURL,
	)
	if err != nil {
		log.Printf("Failed to create/get user: %v", err)
		http.Error(w, "Failed to create user", http.StatusInternalServerError)
		return
	}

	// Update last login
	if err := h.users.UpdateLastLogin(user.ID); err != nil {
		log.Printf("Failed to update last login: %v", err)
	}

	// Create session
	if err := h.sessions.CreateSession(w, user.ID, user.Username, user.IsAdmin); err != nil {
		log.Printf("Failed to create session: %v", err)
		http.Error(w, "Failed to create session", http.StatusInternalServerError)
		return
	}

	// Redirect to dashboard
	http.Redirect(w, r, "/", http.StatusFound)
}

func (h *AuthHandler) Logout(w http.ResponseWriter, r *http.Request) {
	h.sessions.ClearSession(w)
	http.Redirect(w, r, "/login", http.StatusFound)
}

func generateState() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return base64.URLEncoding.EncodeToString(b), nil
}
