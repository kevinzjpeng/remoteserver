package repository

import (
	"database/sql"
	"time"

	"server-access-manager/internal/models"
)

type UserRepository struct {
	db *sql.DB
}

func NewUserRepository(db *sql.DB) *UserRepository {
	return &UserRepository{db: db}
}

func (r *UserRepository) CreateByGithub(githubID, username, email, avatarURL string) (*models.User, error) {
	// Check if this is the first user (should be admin)
	var count int
	err := r.db.QueryRow("SELECT COUNT(*) FROM users").Scan(&count)
	if err != nil {
		return nil, err
	}
	isAdmin := count == 0

	result, err := r.db.Exec(
		`INSERT INTO users (github_id, username, email, avatar_url, is_admin, created_at, last_login)
		 VALUES (?, ?, ?, ?, ?, ?, ?)`,
		githubID, username, email, avatarURL, isAdmin, time.Now(), time.Now(),
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

func (r *UserRepository) GetByID(id int64) (*models.User, error) {
	user := &models.User{}
	err := r.db.QueryRow(
		`SELECT id, github_id, username, email, avatar_url, is_admin, created_at, last_login
		 FROM users WHERE id = ?`, id,
	).Scan(&user.ID, &user.GithubID, &user.Username, &user.Email, &user.AvatarURL,
		&user.IsAdmin, &user.CreatedAt, &user.LastLogin)
	if err != nil {
		return nil, err
	}
	return user, nil
}

func (r *UserRepository) GetByGithubID(githubID string) (*models.User, error) {
	user := &models.User{}
	err := r.db.QueryRow(
		`SELECT id, github_id, username, email, avatar_url, is_admin, created_at, last_login
		 FROM users WHERE github_id = ?`, githubID,
	).Scan(&user.ID, &user.GithubID, &user.Username, &user.Email, &user.AvatarURL,
		&user.IsAdmin, &user.CreatedAt, &user.LastLogin)
	if err != nil {
		return nil, err
	}
	return user, nil
}

func (r *UserRepository) UpdateLastLogin(id int64) error {
	_, err := r.db.Exec("UPDATE users SET last_login = ? WHERE id = ?", time.Now(), id)
	return err
}
