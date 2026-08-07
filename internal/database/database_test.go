package database

import (
	"os"
	"testing"
)

func TestSetupDatabase(t *testing.T) {
	// Create temporary database file
	tmpFile, err := os.CreateTemp("", "test_db_*.db")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(tmpFile.Name())
	tmpFile.Close()

	db, err := SetupDatabase(tmpFile.Name())
	if err != nil {
		t.Fatalf("SetupDatabase() error = %v", err)
	}
	defer db.Close()

	// Verify tables exist
	tables := []string{"users", "environments", "servers", "jump_hosts", "service_credentials", "access_logs"}
	for _, table := range tables {
		var name string
		err := db.QueryRow("SELECT name FROM sqlite_master WHERE type='table' AND name=?", table).Scan(&name)
		if err != nil {
			t.Errorf("Table %s does not exist", table)
		}
	}

	// Verify default environments were created
	var count int
	err = db.QueryRow("SELECT COUNT(*) FROM environments").Scan(&count)
	if err != nil {
		t.Fatalf("Failed to count environments: %v", err)
	}
	if count != 3 {
		t.Errorf("Expected 3 default environments, got %d", count)
	}
}

func TestSetupDatabaseIdempotent(t *testing.T) {
	tmpFile, err := os.CreateTemp("", "test_db_*.db")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(tmpFile.Name())
	tmpFile.Close()

	// Setup twice should not error
	db1, err := SetupDatabase(tmpFile.Name())
	if err != nil {
		t.Fatalf("First SetupDatabase() error = %v", err)
	}
	db1.Close()

	db2, err := SetupDatabase(tmpFile.Name())
	if err != nil {
		t.Fatalf("Second SetupDatabase() error = %v", err)
	}
	db2.Close()
}
