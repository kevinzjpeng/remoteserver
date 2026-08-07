package handlers

import (
	"encoding/json"
	"net/http"
	"strconv"

	"server-access-manager/internal/middleware"
	"server-access-manager/internal/repository"
)

type EnvironmentHandler struct {
	envs *repository.EnvironmentRepository
}

func NewEnvironmentHandler(envs *repository.EnvironmentRepository) *EnvironmentHandler {
	return &EnvironmentHandler{envs: envs}
}

func (h *EnvironmentHandler) List(w http.ResponseWriter, r *http.Request) {
	envs, err := h.envs.GetAll()
	if err != nil {
		http.Error(w, "Failed to load environments", http.StatusInternalServerError)
		return
	}

	user := middleware.GetUser(r)
	data := map[string]interface{}{
		"Environments": envs,
		"User":         user,
		"CSRFToken":    middleware.CSRFToken(r),
	}

	// For now, return JSON - will add HTML templates in Task 14
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(data)
}

func (h *EnvironmentHandler) Create(w http.ResponseWriter, r *http.Request) {
	if err := r.ParseForm(); err != nil {
		http.Error(w, "Invalid form data", http.StatusBadRequest)
		return
	}

	name := r.FormValue("name")
	description := r.FormValue("description")
	color := r.FormValue("color")
	displayOrderStr := r.FormValue("display_order")

	displayOrder, _ := strconv.Atoi(displayOrderStr)

	env, err := h.envs.Create(name, description, color, displayOrder)
	if err != nil {
		http.Error(w, "Failed to create environment", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(env)
}

func (h *EnvironmentHandler) Update(w http.ResponseWriter, r *http.Request) {
	idStr := r.URL.Query().Get("id")
	id, err := strconv.ParseInt(idStr, 10, 64)
	if err != nil {
		http.Error(w, "Invalid ID", http.StatusBadRequest)
		return
	}

	if err := r.ParseForm(); err != nil {
		http.Error(w, "Invalid form data", http.StatusBadRequest)
		return
	}

	name := r.FormValue("name")
	description := r.FormValue("description")
	color := r.FormValue("color")
	displayOrderStr := r.FormValue("display_order")

	displayOrder, _ := strconv.Atoi(displayOrderStr)

	if err := h.envs.Update(id, name, description, color, displayOrder); err != nil {
		http.Error(w, "Failed to update environment", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func (h *EnvironmentHandler) Delete(w http.ResponseWriter, r *http.Request) {
	idStr := r.URL.Query().Get("id")
	id, err := strconv.ParseInt(idStr, 10, 64)
	if err != nil {
		http.Error(w, "Invalid ID", http.StatusBadRequest)
		return
	}

	if err := h.envs.Delete(id); err != nil {
		http.Error(w, "Failed to delete environment", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}
