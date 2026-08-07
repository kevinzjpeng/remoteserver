package models

import "time"

type Server struct {
	ID            int64        `json:"id"`
	Name          string       `json:"name"`
	Hostname      string       `json:"hostname"`
	Port          int          `json:"port"`
	Username      string       `json:"username"`
	EnvironmentID *int64       `json:"environment_id"`
	Environment   *Environment `json:"environment,omitempty"`

	// Authentication
	AuthType   string `json:"auth_type"`
	Password   string `json:"password,omitempty"`
	PrivateKey string `json:"private_key,omitempty"`
	Passphrase string `json:"passphrase,omitempty"`

	// Metadata
	Description string   `json:"description"`
	OSType      string   `json:"os_type"`
	Tags        []string `json:"tags"`
	Notes       string   `json:"notes"`

	// Jump hosts
	JumpHosts []Server `json:"jump_hosts,omitempty"`

	// Timestamps
	CreatedAt    time.Time `json:"created_at"`
	UpdatedAt    time.Time `json:"updated_at"`
	LastAccessed time.Time `json:"last_accessed"`
}
