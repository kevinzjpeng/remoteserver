package config

import (
	"fmt"
	"os"
	"strconv"
	"time"

	"github.com/joho/godotenv"
)

type Config struct {
	GithubClientID     string
	GithubClientSecret string
	SessionSecret      string
	EncryptionKey      string
	DatabasePath       string
	Port               string
	SessionDuration    time.Duration
}

func LoadConfig() (*Config, error) {
	// Load .env file if it exists (ignore error in production)
	_ = godotenv.Load()

	cfg := &Config{
		GithubClientID:     os.Getenv("GITHUB_CLIENT_ID"),
		GithubClientSecret: os.Getenv("GITHUB_CLIENT_SECRET"),
		SessionSecret:      os.Getenv("SESSION_SECRET"),
		EncryptionKey:      os.Getenv("ENCRYPTION_KEY"),
		DatabasePath:       os.Getenv("DATABASE_PATH"),
		Port:               os.Getenv("PORT"),
	}

	// Set defaults
	if cfg.DatabasePath == "" {
		cfg.DatabasePath = "./data/server-access-manager.db"
	}
	if cfg.Port == "" {
		cfg.Port = "8080"
	}

	// Parse session duration
	sessionDuration := os.Getenv("SESSION_DURATION")
	if sessionDuration == "" {
		cfg.SessionDuration = 24 * time.Hour
	} else {
		// Parse duration string like "24h", "12h", etc.
		duration, err := time.ParseDuration(sessionDuration)
		if err != nil {
			// Try parsing as hours (e.g., "24")
			hours, err := strconv.Atoi(sessionDuration)
			if err != nil {
				return nil, fmt.Errorf("invalid SESSION_DURATION: %v", err)
			}
			cfg.SessionDuration = time.Duration(hours) * time.Hour
		} else {
			cfg.SessionDuration = duration
		}
	}

	// Validate required fields
	if cfg.GithubClientID == "" {
		return nil, fmt.Errorf("GITHUB_CLIENT_ID is required")
	}
	if cfg.GithubClientSecret == "" {
		return nil, fmt.Errorf("GITHUB_CLIENT_SECRET is required")
	}
	if cfg.SessionSecret == "" {
		return nil, fmt.Errorf("SESSION_SECRET is required")
	}
	if cfg.EncryptionKey == "" {
		return nil, fmt.Errorf("ENCRYPTION_KEY is required")
	}

	return cfg, nil
}
