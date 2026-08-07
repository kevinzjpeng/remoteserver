package models

import "time"

type AccessLog struct {
	ID             int64     `json:"id"`
	ServerID       int64     `json:"server_id"`
	Server         *Server   `json:"server,omitempty"`
	UserID         int64     `json:"user_id"`
	User           *User     `json:"user,omitempty"`
	AccessedAt     time.Time `json:"accessed_at"`
	ConnectionType string    `json:"connection_type"`
	Success        bool      `json:"success"`
	Notes          string    `json:"notes"`
}
