package middleware

import (
	"context"
	"net/http"

	"server-access-manager/internal/auth"
)

type contextKey string

const userContextKey contextKey = "user"

func Auth(sessions *auth.SessionManager) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			session, err := sessions.GetSession(r)
			if err != nil {
				// No valid session, redirect to login
				http.Redirect(w, r, "/login", http.StatusFound)
				return
			}

			// Add user to context
			ctx := context.WithValue(r.Context(), userContextKey, session)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

func AuthAPI(sessions *auth.SessionManager) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			session, err := sessions.GetSession(r)
			if err != nil {
				// No valid session, return 401
				http.Error(w, "Unauthorized", http.StatusUnauthorized)
				return
			}

			// Add user to context
			ctx := context.WithValue(r.Context(), userContextKey, session)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

func AdminRequired(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		session, ok := r.Context().Value(userContextKey).(*auth.SessionData)
		if !ok || !session.IsAdmin {
			http.Error(w, "Forbidden", http.StatusForbidden)
			return
		}

		next.ServeHTTP(w, r)
	})
}

func GetUser(r *http.Request) *auth.SessionData {
	session, ok := r.Context().Value(userContextKey).(*auth.SessionData)
	if !ok {
		return nil
	}
	return session
}
