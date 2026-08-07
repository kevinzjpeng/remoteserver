package middleware

import (
	"crypto/rand"
	"encoding/base64"
	"net/http"
	"strings"
)

const csrfTokenCookie = "csrf_token"
const csrfTokenHeader = "X-CSRF-Token"

func CSRFProtect(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Generate CSRF token if not present
		cookie, err := r.Cookie(csrfTokenCookie)
		if err != nil {
			token, err := generateCSRFToken()
			if err != nil {
				http.Error(w, "Failed to generate CSRF token", http.StatusInternalServerError)
				return
			}

			http.SetCookie(w, &http.Cookie{
				Name:     csrfTokenCookie,
				Value:    token,
				Path:     "/",
				HttpOnly: false, // JavaScript needs to read this
				Secure:   true,
				SameSite: http.SameSiteStrictMode,
			})
		}

		// For state-changing methods, validate CSRF token
		if r.Method == "POST" || r.Method == "PUT" || r.Method == "DELETE" || r.Method == "PATCH" {
			// Get token from header or form
			token := r.Header.Get(csrfTokenHeader)
			if token == "" {
				token = r.FormValue("csrf_token")
			}

			// Validate token
			if token == "" || (cookie != nil && token != cookie.Value) {
				http.Error(w, "Invalid CSRF token", http.StatusForbidden)
				return
			}
		}

		next.ServeHTTP(w, r)
	})
}

func CSRFToken(r *http.Request) string {
	cookie, err := r.Cookie(csrfTokenCookie)
	if err != nil {
		return ""
	}
	return cookie.Value
}

func generateCSRFToken() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return base64.URLEncoding.EncodeToString(b), nil
}

func SkipCSRF(paths []string) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			for _, path := range paths {
				if strings.HasPrefix(r.URL.Path, path) {
					next.ServeHTTP(w, r)
					return
				}
			}
			CSRFProtect(next).ServeHTTP(w, r)
		})
	}
}
