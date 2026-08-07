package handlers

import (
	"encoding/json"
	"net/http"

	"server-access-manager/internal/middleware"
	"server-access-manager/internal/repository"
)

type HomeHandler struct {
	servers *repository.ServerRepository
	envs    *repository.EnvironmentRepository
}

func NewHomeHandler(servers *repository.ServerRepository, envs *repository.EnvironmentRepository) *HomeHandler {
	return &HomeHandler{servers: servers, envs: envs}
}

func (h *HomeHandler) Dashboard(w http.ResponseWriter, r *http.Request) {
	servers, err := h.servers.GetAll()
	if err != nil {
		http.Error(w, "Failed to load servers", http.StatusInternalServerError)
		return
	}

	envs, err := h.envs.GetAll()
	if err != nil {
		http.Error(w, "Failed to load environments", http.StatusInternalServerError)
		return
	}

	user := middleware.GetUser(r)
	data := map[string]interface{}{
		"Servers":      servers,
		"Environments": envs,
		"User":         user,
		"CSRFToken":    middleware.CSRFToken(r),
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(data)
}
