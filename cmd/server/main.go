package main

import (
	"database/sql"
	"log"
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"

	"server-access-manager/internal/auth"
	"server-access-manager/internal/config"
	"server-access-manager/internal/database"
	"server-access-manager/internal/handlers"
	appmiddleware "server-access-manager/internal/middleware"
	"server-access-manager/internal/repository"
)

var db *sql.DB

func main() {
	// Load configuration
	cfg, err := config.LoadConfig()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	// Setup database
	db, err = database.SetupDatabase(cfg.DatabasePath)
	if err != nil {
		log.Fatalf("Failed to setup database: %v", err)
	}
	defer db.Close()

	log.Printf("Database initialized at %s", cfg.DatabasePath)

	// Initialize repositories
	userRepo := repository.NewUserRepository(db)
	envRepo := repository.NewEnvironmentRepository(db)
	serverRepo := repository.NewServerRepository(db, cfg)
	credRepo := repository.NewCredentialRepository(db, cfg)
	logRepo := repository.NewAccessLogRepository(db)

	// Initialize auth components
	githubOAuth := auth.NewGitHubOAuthClient(
		cfg.GithubClientID,
		cfg.GithubClientSecret,
		"http://localhost:"+cfg.Port+"/auth/github/callback",
	)
	sessions := auth.NewSessionManager(cfg.SessionSecret, cfg.SessionDuration)

	// Initialize handlers
	authHandler := handlers.NewAuthHandler(githubOAuth, sessions, userRepo)
	homeHandler := handlers.NewHomeHandler(serverRepo, envRepo)
	envHandler := handlers.NewEnvironmentHandler(envRepo)
	serverHandler := handlers.NewServerHandler(serverRepo, envRepo)
	credHandler := handlers.NewCredentialHandler(credRepo)
	topoHandler := handlers.NewTopologyHandler(serverRepo, envRepo)
	logHandler := handlers.NewLogHandler(logRepo)
	terminalHandler := handlers.NewTerminalHandler(serverRepo)

	// Setup router
	r := chi.NewRouter()

	// Global middleware
	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(appmiddleware.Logging)
	r.Use(appmiddleware.Recovery)

	// WebSocket endpoint for terminal (must be after middleware)
	r.Get("/ws/terminal", terminalHandler.Terminal)

	// Serve static files
	fileServer := http.FileServer(http.Dir("static"))
	r.Handle("/static/*", http.StripPrefix("/static/", fileServer))

	// Public routes
	r.Get("/login", authHandler.Login)
	r.Get("/auth/github/callback", authHandler.Callback)
	r.Post("/logout", authHandler.Logout)

	// Serve topology HTML page (no auth required for viewing)
	r.Get("/topology", func(w http.ResponseWriter, r *http.Request) {
		http.ServeFile(w, r, "internal/templates/topology.html")
	})

	// Serve terminal HTML page
	r.Get("/terminal", func(w http.ResponseWriter, r *http.Request) {
		http.ServeFile(w, r, "internal/templates/terminal.html")
	})

	// Public API endpoints
	r.Get("/api/topology", topoHandler.GetTopology)

	// Protected routes
	r.Group(func(r chi.Router) {
		r.Use(appmiddleware.Auth(sessions))
		r.Use(appmiddleware.CSRFProtect)

		// Dashboard
		r.Get("/", homeHandler.Dashboard)

		// Admin routes
		r.Route("/admin", func(r chi.Router) {
			r.Use(appmiddleware.AdminRequired)

			// Environments
			r.Get("/environments", envHandler.List)
			r.Post("/environments", envHandler.Create)
			r.Put("/environments", envHandler.Update)
			r.Delete("/environments", envHandler.Delete)
			r.Get("/environments/{id}/edit", envHandler.Edit)

			// Servers
			r.Get("/servers", serverHandler.List)
			r.Post("/servers", serverHandler.Create)
			r.Put("/servers", serverHandler.Update)
			r.Delete("/servers", serverHandler.Delete)
			r.Get("/servers/{id}/edit", serverHandler.Edit)

			// Credentials
			r.Get("/credentials", credHandler.List)
			r.Post("/credentials", credHandler.Create)
			r.Put("/credentials", credHandler.Update)
			r.Delete("/credentials", credHandler.Delete)

			// Logs
			r.Get("/logs", logHandler.List)
			r.Get("/logs/server", logHandler.GetByServer)
		})
	})

	// Start server
	addr := ":" + cfg.Port
	log.Printf("Starting server on %s", addr)
	log.Printf("Visit http://localhost%s to access the application", addr)
	if err := http.ListenAndServe(addr, r); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}
