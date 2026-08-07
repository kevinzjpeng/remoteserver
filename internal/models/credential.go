package models

import (
	"encoding/json"
	"time"
)

type ServiceCredential struct {
	ID          int64           `json:"id"`
	Name        string          `json:"name"`
	ServiceType string          `json:"service_type"`
	Credentials json.RawMessage `json:"credentials,omitempty"`
	ConfigJSON  string          `json:"config_json"`
	IsActive    bool            `json:"is_active"`
	CreatedAt   time.Time       `json:"created_at"`
	UpdatedAt   time.Time       `json:"updated_at"`
}

// GitHubOAuthCredentials represents GitHub OAuth credentials
type GitHubOAuthCredentials struct {
	ClientID     string `json:"client_id"`
	ClientSecret string `json:"client_secret"`
}

// AWSCredentials represents AWS credentials
type AWSCredentials struct {
	AccessKeyID     string `json:"access_key_id"`
	SecretAccessKey string `json:"secret_access_key"`
	Region          string `json:"region"`
}

// GenericAPICredentials represents generic API credentials
type GenericAPICredentials struct {
	APIKey   string `json:"api_key"`
	Endpoint string `json:"endpoint,omitempty"`
	Username string `json:"username,omitempty"`
	Password string `json:"password,omitempty"`
}
