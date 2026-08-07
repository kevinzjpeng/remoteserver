package repository

import (
	"database/sql"
	"encoding/json"
	"time"

	"server-access-manager/internal/config"
	"server-access-manager/internal/crypto"
	"server-access-manager/internal/models"
)

type ServerRepository struct {
	db            *sql.DB
	encryptionKey []byte
}

func NewServerRepository(db *sql.DB, cfg *config.Config) *ServerRepository {
	return &ServerRepository{
		db:            db,
		encryptionKey: []byte(cfg.EncryptionKey),
	}
}

func (r *ServerRepository) GetAll() ([]models.Server, error) {
	rows, err := r.db.Query(
		`SELECT s.id, s.name, s.hostname, s.port, s.username, s.environment_id,
		        s.auth_type, s.description, s.os_type, s.tags, s.notes,
		        s.created_at, s.updated_at, s.last_accessed,
		        e.id, e.name, e.color
		 FROM servers s
		 LEFT JOIN environments e ON s.environment_id = e.id
		 ORDER BY s.name`,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var servers []models.Server
	for rows.Next() {
		var server models.Server
		var tagsJSON sql.NullString
		var envID sql.NullInt64
		var envName, envColor sql.NullString
		var lastAccessed sql.NullTime

		err := rows.Scan(
			&server.ID, &server.Name, &server.Hostname, &server.Port, &server.Username,
			&envID, &server.AuthType, &server.Description, &server.OSType, &tagsJSON,
			&server.Notes, &server.CreatedAt, &server.UpdatedAt, &lastAccessed,
			&envID, &envName, &envColor,
		)
		if err != nil {
			return nil, err
		}

		// Parse tags
		if tagsJSON.Valid {
			json.Unmarshal([]byte(tagsJSON.String), &server.Tags)
		}

		// Set environment if exists
		if envID.Valid {
			server.Environment = &models.Environment{
				ID:    envID.Int64,
				Name:  envName.String,
				Color: envColor.String,
			}
		}

		// Set last accessed if not null
		if lastAccessed.Valid {
			server.LastAccessed = lastAccessed.Time
		}

		servers = append(servers, server)
	}
	return servers, nil
}

func (r *ServerRepository) GetByID(id int64) (*models.Server, error) {
	server := &models.Server{}
	var tagsJSON sql.NullString
	var envID sql.NullInt64
	var envName, envColor sql.NullString
	var lastAccessed sql.NullTime

	err := r.db.QueryRow(
		`SELECT s.id, s.name, s.hostname, s.port, s.username, s.environment_id,
		        s.auth_type, s.description, s.os_type, s.tags, s.notes,
		        s.created_at, s.updated_at, s.last_accessed,
		        e.id, e.name, e.color
		 FROM servers s
		 LEFT JOIN environments e ON s.environment_id = e.id
		 WHERE s.id = ?`, id,
	).Scan(
		&server.ID, &server.Name, &server.Hostname, &server.Port, &server.Username,
		&envID, &server.AuthType, &server.Description, &server.OSType, &tagsJSON,
		&server.Notes, &server.CreatedAt, &server.UpdatedAt, &lastAccessed,
		&envID, &envName, &envColor,
	)
	if err != nil {
		return nil, err
	}

	if tagsJSON.Valid {
		json.Unmarshal([]byte(tagsJSON.String), &server.Tags)
	}

	if envID.Valid {
		server.Environment = &models.Environment{
			ID:    envID.Int64,
			Name:  envName.String,
			Color: envColor.String,
		}
	}

	// Set last accessed if not null
	if lastAccessed.Valid {
		server.LastAccessed = lastAccessed.Time
	}

	return server, nil
}

func (r *ServerRepository) Create(server *models.Server) (*models.Server, error) {
	tagsJSON, _ := json.Marshal(server.Tags)

	result, err := r.db.Exec(
		`INSERT INTO servers (name, hostname, port, username, environment_id, auth_type,
		                      password_encrypted, private_key_encrypted, passphrase_encrypted,
		                      description, os_type, tags, notes, created_at, updated_at)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		server.Name, server.Hostname, server.Port, server.Username, server.EnvironmentID,
		server.AuthType,
		r.encryptField(server.Password),
		r.encryptField(server.PrivateKey),
		r.encryptField(server.Passphrase),
		server.Description, server.OSType, string(tagsJSON), server.Notes,
		time.Now(), time.Now(),
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

func (r *ServerRepository) Update(server *models.Server) error {
	tagsJSON, _ := json.Marshal(server.Tags)

	_, err := r.db.Exec(
		`UPDATE servers SET name = ?, hostname = ?, port = ?, username = ?, environment_id = ?,
		                      auth_type = ?, password_encrypted = ?, private_key_encrypted = ?,
		                      passphrase_encrypted = ?, description = ?, os_type = ?, tags = ?,
		                      notes = ?, updated_at = ?
		 WHERE id = ?`,
		server.Name, server.Hostname, server.Port, server.Username, server.EnvironmentID,
		server.AuthType,
		r.encryptField(server.Password),
		r.encryptField(server.PrivateKey),
		r.encryptField(server.Passphrase),
		server.Description, server.OSType, string(tagsJSON), server.Notes,
		time.Now(), server.ID,
	)
	return err
}

func (r *ServerRepository) Delete(id int64) error {
	_, err := r.db.Exec("DELETE FROM servers WHERE id = ?", id)
	return err
}

func (r *ServerRepository) UpdateLastAccessed(id int64) error {
	_, err := r.db.Exec("UPDATE servers SET last_accessed = ? WHERE id = ?", time.Now(), id)
	return err
}

func (r *ServerRepository) GetPassword(id int64) (string, error) {
	var encrypted sql.NullString
	err := r.db.QueryRow("SELECT password_encrypted FROM servers WHERE id = ?", id).Scan(&encrypted)
	if err != nil {
		return "", err
	}
	if !encrypted.Valid {
		return "", nil
	}
	return r.decryptField(encrypted.String)
}

func (r *ServerRepository) GetPrivateKey(id int64) (string, error) {
	var encrypted sql.NullString
	err := r.db.QueryRow("SELECT private_key_encrypted FROM servers WHERE id = ?", id).Scan(&encrypted)
	if err != nil {
		return "", err
	}
	if !encrypted.Valid {
		return "", nil
	}
	return r.decryptField(encrypted.String)
}

func (r *ServerRepository) encryptField(value string) string {
	if value == "" {
		return ""
	}
	encrypted, err := crypto.Encrypt([]byte(value), r.encryptionKey)
	if err != nil {
		return ""
	}
	return string(encrypted)
}

func (r *ServerRepository) decryptField(value string) (string, error) {
	if value == "" {
		return "", nil
	}
	decrypted, err := crypto.Decrypt([]byte(value), r.encryptionKey)
	if err != nil {
		return "", err
	}
	return string(decrypted), nil
}
