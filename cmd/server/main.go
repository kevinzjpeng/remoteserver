package main

import (
	"database/sql"
	"fmt"
	"log"
	"net/http"

	"server-access-manager/internal/config"
	"server-access-manager/internal/database"
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

	// Basic HTTP server
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Server Access Manager - Database Connected!")
	})

	addr := ":" + cfg.Port
	log.Printf("Starting server on %s", addr)
	if err := http.ListenAndServe(addr, nil); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}
