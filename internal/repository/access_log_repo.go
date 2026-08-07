package repository

import (
	"database/sql"
	"time"

	"server-access-manager/internal/models"
)

type AccessLogRepository struct {
	db *sql.DB
}

func NewAccessLogRepository(db *sql.DB) *AccessLogRepository {
	return &AccessLogRepository{db: db}
}

func (r *AccessLogRepository) Create(serverID, userID int64, connectionType string, success bool, notes string) error {
	_, err := r.db.Exec(
		`INSERT INTO access_logs (server_id, user_id, accessed_at, connection_type, success, notes)
		 VALUES (?, ?, ?, ?, ?, ?)`,
		serverID, userID, time.Now(), connectionType, success, notes,
	)
	return err
}

func (r *AccessLogRepository) GetAll(limit int) ([]models.AccessLog, error) {
	rows, err := r.db.Query(
		`SELECT al.id, al.server_id, al.user_id, al.accessed_at, al.connection_type, al.success, al.notes,
		        s.name, s.hostname, u.username
		 FROM access_logs al
		 JOIN servers s ON al.server_id = s.id
		 JOIN users u ON al.user_id = u.id
		 ORDER BY al.accessed_at DESC
		 LIMIT ?`, limit,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var logs []models.AccessLog
	for rows.Next() {
		var log models.AccessLog
		log.Server = &models.Server{}
		log.User = &models.User{}

		err := rows.Scan(&log.ID, &log.ServerID, &log.UserID, &log.AccessedAt,
			&log.ConnectionType, &log.Success, &log.Notes,
			&log.Server.Name, &log.Server.Hostname, &log.User.Username)
		if err != nil {
			return nil, err
		}
		logs = append(logs, log)
	}
	return logs, nil
}

func (r *AccessLogRepository) GetByServerID(serverID int64, limit int) ([]models.AccessLog, error) {
	rows, err := r.db.Query(
		`SELECT id, server_id, user_id, accessed_at, connection_type, success, notes
		 FROM access_logs WHERE server_id = ?
		 ORDER BY accessed_at DESC LIMIT ?`,
		serverID, limit,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var logs []models.AccessLog
	for rows.Next() {
		var log models.AccessLog
		err := rows.Scan(&log.ID, &log.ServerID, &log.UserID, &log.AccessedAt,
			&log.ConnectionType, &log.Success, &log.Notes)
		if err != nil {
			return nil, err
		}
		logs = append(logs, log)
	}
	return logs, nil
}
