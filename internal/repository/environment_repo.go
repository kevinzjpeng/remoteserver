package repository

import (
	"database/sql"
	"time"

	"server-access-manager/internal/models"
)

type EnvironmentRepository struct {
	db *sql.DB
}

func NewEnvironmentRepository(db *sql.DB) *EnvironmentRepository {
	return &EnvironmentRepository{db: db}
}

func (r *EnvironmentRepository) GetAll() ([]models.Environment, error) {
	rows, err := r.db.Query(
		`SELECT id, name, description, color, display_order, created_at
		 FROM environments ORDER BY display_order`,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var environments []models.Environment
	for rows.Next() {
		var env models.Environment
		err := rows.Scan(&env.ID, &env.Name, &env.Description, &env.Color, &env.DisplayOrder, &env.CreatedAt)
		if err != nil {
			return nil, err
		}
		environments = append(environments, env)
	}
	return environments, nil
}

func (r *EnvironmentRepository) GetByID(id int64) (*models.Environment, error) {
	env := &models.Environment{}
	err := r.db.QueryRow(
		`SELECT id, name, description, color, display_order, created_at
		 FROM environments WHERE id = ?`, id,
	).Scan(&env.ID, &env.Name, &env.Description, &env.Color, &env.DisplayOrder, &env.CreatedAt)
	if err != nil {
		return nil, err
	}
	return env, nil
}

func (r *EnvironmentRepository) Create(name, description, color string, displayOrder int) (*models.Environment, error) {
	result, err := r.db.Exec(
		`INSERT INTO environments (name, description, color, display_order, created_at)
		 VALUES (?, ?, ?, ?, ?)`,
		name, description, color, displayOrder, time.Now(),
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

func (r *EnvironmentRepository) Update(id int64, name, description, color string, displayOrder int) error {
	_, err := r.db.Exec(
		`UPDATE environments SET name = ?, description = ?, color = ?, display_order = ?
		 WHERE id = ?`,
		name, description, color, displayOrder, id,
	)
	return err
}

func (r *EnvironmentRepository) Delete(id int64) error {
	_, err := r.db.Exec("DELETE FROM environments WHERE id = ?", id)
	return err
}
