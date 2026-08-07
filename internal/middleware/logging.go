package middleware

import (
	"log"
	"net/http"
	"time"
)

type responseWriter struct {
	http.ResponseWriter
	status int
}

func (rw *responseWriter) WriteHeader(code int) {
	rw.status = code
	rw.ResponseWriter.WriteHeader(code)
}

func Logging(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()

		// Wrap response writer to capture status code
		rw := &responseWriter{
			ResponseWriter: w,
			status:         http.StatusOK,
		}

		// Call next handler
		next.ServeHTTP(rw, r)

		// Log request
		duration := time.Since(start)
		user := GetUser(r)
		username := "anonymous"
		if user != nil {
			username = user.Username
		}

		log.Printf("%s %s %s %d %s %s",
			r.RemoteAddr,
			r.Method,
			r.URL.Path,
			rw.status,
			duration,
			username,
		)
	})
}

func Recovery(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			if err := recover(); err != nil {
				log.Printf("Panic recovered: %v", err)
				http.Error(w, "Internal Server Error", http.StatusInternalServerError)
			}
		}()

		next.ServeHTTP(w, r)
	})
}
