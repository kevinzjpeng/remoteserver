package repository

import (
	"database/sql"
	"encoding/json"
	"time"

	"server-access-manager/internal/config"
	"server-access-manager/internal/crypto"
	"server-access-manager/internal/models"
)

type CredentialRepository struct {
	db            *sql.DB
	encryptionKey []byte
}

func NewCredentialRepository(db *sql.DB, cfg *config.Config) *CredentialRepository {
	return &CredentialRepository{
		db:            db,
		encryptionKey: []byte(cfg.EncryptionKey),
	}
}

func (r *CredentialRepository) GetAll() ([]models.ServiceCredential, error) {
	rows, err := r.db.Query(
		`SELECT id, name, service_type, config_json, is_active, created_at, updated_at
		 FROM service_credentials ORDER BY name`,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var credentials []models.ServiceCredential
	for rows.Next() {
		var cred models.ServiceCredential
		err := rows.Scan(&cred.ID, &cred.Name, &cred.ServiceType, &cred.ConfigJSON,
			&cred.IsActive, &cred.CreatedAt, &cred.UpdatedAt)
		if err != nil {
			return nil, err
		}
		credentials = append(credentials, cred)
	}
	return credentials, nil
}

func (r *CredentialRepository) GetByID(id int64) (*models.ServiceCredential, error) {
	cred := &models.ServiceCredential{}
	err := r.db.QueryRow(
		`SELECT id, name, service_type, config_json, is_active, created_at, updated_at
		 FROM service_credentials WHERE id = ?`, id,
	).Scan(&cred.ID, &cred.Name, &cred.ServiceType, &cred.ConfigJSON,
		&cred.IsActive, &cred.CreatedAt, &cred.UpdatedAt)
	if err != nil {
		return nil, err
	}
	return cred, nil
}

func (r *CredentialRepository) GetByType(serviceType string) (*models.ServiceCredential, error) {
	cred := &models.ServiceCredential{}
	err := r.db.QueryRow(
		`SELECT id, name, service_type, credentials_encrypted, config_json, is_active, created_at, updated_at
		 FROM service_credentials WHERE service_type = ? AND is_active = 1`, serviceType,
	).Scan(&cred.ID, &cred.Name, &cred.ServiceType, &cred.Credentials, &cred.ConfigJSON,
		&cred.IsActive, &cred.CreatedAt, &cred.UpdatedAt)
	if err != nil {
		return nil, err
	}
	return cred, nil
}

func (r *CredentialRepository) Create(cred *models.ServiceCredential) (*models.ServiceCredential, error) {
	result, err := r.db.Exec(
		`INSERT INTO service_credentials (name, service_type, credentials_encrypted, config_json, is_active, created_at, updated_at)
		 VALUES (?, ?, ?, ?, ?, ?, ?)`,
		cred.Name, cred.ServiceType, r.encryptCredentials(cred.Credentials),
		cred.ConfigJSON, cred.IsActive, time.Now(), time.Now(),
	)
	if err != nil {
		return nil, err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, err
	}

	return r.GetByID(id)
}

func (r *CredentialRepository) Update(cred *models.ServiceCredential) error {
	_, err := r.db.Exec(
		`UPDATE service_credentials SET name = ?, service_type = ?, credentials_encrypted = ?,
		                              config_json = ?, is_active = ?, updated_at = ?
		 WHERE id = ?`,
		cred.Name, cred.ServiceType, r.encryptCredentials(cred.Credentials),
		cred.ConfigJSON, cred.IsActive, time.Now(), cred.ID,
	)
	return err
}

func (r *CredentialRepository) Delete(id int64) error {
	_, err := r.db.Exec("DELETE FROM service_credentials WHERE id = ?", id)
	return err
}

func (r *CredentialRepository) GetDecryptedCredentials(id int64) (json.RawMessage, error) {
	var encrypted string
	err := r.db.QueryRow("SELECT credentials_encrypted FROM service_credentials WHERE id = ?", id).Scan(&encrypted)
	if err != nil {
		return nil, err
	}

	decrypted, err := crypto.Decrypt([]byte(encrypted), r.encryptionKey)
	if err != nil {
		return nil, err
	}

	return json.RawMessage(decrypted), nil
}

func (r *CredentialRepository) encryptCredentials(creds json.RawMessage) string {
	if len(creds) == 0 {
		return ""
	}
	encrypted, err := crypto.Encrypt(creds, r.encryptionKey)
	if err != nil {
		return ""
	}
	return string(encrypted)
}
