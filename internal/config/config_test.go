package config

import (
	"os"
	"testing"
	"time"
)

func TestLoadConfig(t *testing.T) {
	// Set test environment variables
	os.Setenv("GITHUB_CLIENT_ID", "test_client_id")
	os.Setenv("GITHUB_CLIENT_SECRET", "test_client_secret")
	os.Setenv("SESSION_SECRET", "test_session_secret_32_bytes_long!!")
	os.Setenv("ENCRYPTION_KEY", "test_encryption_key_32_bytes_long!")
	os.Setenv("DATABASE_PATH", "/tmp/test.db")
	os.Setenv("PORT", "9090")
	os.Setenv("SESSION_DURATION", "12h")

	defer func() {
		os.Unsetenv("GITHUB_CLIENT_ID")
		os.Unsetenv("GITHUB_CLIENT_SECRET")
		os.Unsetenv("SESSION_SECRET")
		os.Unsetenv("ENCRYPTION_KEY")
		os.Unsetenv("DATABASE_PATH")
		os.Unsetenv("PORT")
		os.Unsetenv("SESSION_DURATION")
	}()

	cfg, err := LoadConfig()
	if err != nil {
		t.Fatalf("LoadConfig() error = %v", err)
	}

	if cfg.GithubClientID != "test_client_id" {
		t.Errorf("GithubClientID = %v, want %v", cfg.GithubClientID, "test_client_id")
	}
	if cfg.GithubClientSecret != "test_client_secret" {
		t.Errorf("GithubClientSecret = %v, want %v", cfg.GithubClientSecret, "test_client_secret")
	}
	if cfg.SessionSecret != "test_session_secret_32_bytes_long!!" {
		t.Errorf("SessionSecret = %v, want %v", cfg.SessionSecret, "test_session_secret_32_bytes_long!!")
	}
	if cfg.EncryptionKey != "test_encryption_key_32_bytes_long!" {
		t.Errorf("EncryptionKey = %v, want %v", cfg.EncryptionKey, "test_encryption_key_32_bytes_long!")
	}
	if cfg.DatabasePath != "/tmp/test.db" {
		t.Errorf("DatabasePath = %v, want %v", cfg.DatabasePath, "/tmp/test.db")
	}
	if cfg.Port != "9090" {
		t.Errorf("Port = %v, want %v", cfg.Port, "9090")
	}
	if cfg.SessionDuration != 12*time.Hour {
		t.Errorf("SessionDuration = %v, want %v", cfg.SessionDuration, 12*time.Hour)
	}
}

func TestLoadConfigDefaults(t *testing.T) {
	os.Setenv("GITHUB_CLIENT_ID", "test")
	os.Setenv("GITHUB_CLIENT_SECRET", "test")
	os.Setenv("SESSION_SECRET", "test_session_secret_32_bytes_long!!")
	os.Setenv("ENCRYPTION_KEY", "test_encryption_key_32_bytes_long!")

	defer func() {
		os.Unsetenv("GITHUB_CLIENT_ID")
		os.Unsetenv("GITHUB_CLIENT_SECRET")
		os.Unsetenv("SESSION_SECRET")
		os.Unsetenv("ENCRYPTION_KEY")
		os.Unsetenv("DATABASE_PATH")
		os.Unsetenv("PORT")
		os.Unsetenv("SESSION_DURATION")
	}()

	cfg, err := LoadConfig()
	if err != nil {
		t.Fatalf("LoadConfig() error = %v", err)
	}

	if cfg.Port != "8080" {
		t.Errorf("Default Port = %v, want %v", cfg.Port, "8080")
	}
	if cfg.SessionDuration != 24*time.Hour {
		t.Errorf("Default SessionDuration = %v, want %v", cfg.SessionDuration, 24*time.Hour)
	}
}
