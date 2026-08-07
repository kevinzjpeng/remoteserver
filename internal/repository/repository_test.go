package repository

import (
	"database/sql"
	"os"
	"testing"

	"server-access-manager/internal/database"
)

var testDB *sql.DB

func TestMain(m *testing.M) {
	// Create temporary database
	tmpFile, err := os.CreateTemp("", "test_repo_*.db")
	if err != nil {
		panic(err)
	}
	defer os.Remove(tmpFile.Name())
	tmpFile.Close()

	testDB, err = database.SetupDatabase(tmpFile.Name())
	if err != nil {
		panic(err)
	}
	defer testDB.Close()

	os.Exit(m.Run())
}

func TestUserRepo_CreateAndGet(t *testing.T) {
	repo := NewUserRepository(testDB)

	user, err := repo.CreateByGithub("12345", "testuser", "test@example.com", "https://avatar.url")
	if err != nil {
		t.Fatalf("CreateByGithub() error = %v", err)
	}

	if user.ID == 0 {
		t.Error("User ID should be set")
	}
	if user.GithubID != "12345" {
		t.Errorf("GithubID = %v, want %v", user.GithubID, "12345")
	}
	if user.Username != "testuser" {
		t.Errorf("Username = %v, want %v", user.Username, "testuser")
	}
	if !user.IsAdmin {
		t.Error("First user should be admin")
	}

	// Get by GitHub ID
	retrieved, err := repo.GetByGithubID("12345")
	if err != nil {
		t.Fatalf("GetByGithubID() error = %v", err)
	}
	if retrieved.ID != user.ID {
		t.Errorf("Retrieved user ID = %v, want %v", retrieved.ID, user.ID)
	}
}
