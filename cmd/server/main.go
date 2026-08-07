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

	// Setup router
	r := chi.NewRouter()

	// Global middleware
	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(appmiddleware.Logging)
	r.Use(appmiddleware.Recovery)

	// Public routes
	r.Get("/login", authHandler.Login)
	r.Get("/auth/github/callback", authHandler.Callback)
	r.Post("/logout", authHandler.Logout)

	// Protected routes
	r.Group(func(r chi.Router) {
		r.Use(appmiddleware.Auth(sessions))
		r.Use(appmiddleware.CSRFProtect)

		// Dashboard
		r.Get("/", homeHandler.Dashboard)

		// Topology
		r.Get("/api/topology", topoHandler.GetTopology)

		// Admin routes
		r.Route("/admin", func(r chi.Router) {
			r.Use(appmiddleware.AdminRequired)

			// Environments
			r.Get("/environments", envHandler.List)
			r.Post("/environments", envHandler.Create)
			r.Put("/environments", envHandler.Update)
			r.Delete("/environments", envHandler.Delete)

			// Servers
			r.Get("/servers", serverHandler.List)
			r.Post("/servers", serverHandler.Create)
			r.Put("/servers", serverHandler.Update)
			r.Delete("/servers", serverHandler.Delete)

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
