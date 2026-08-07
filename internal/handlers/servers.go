package handlers

import (
	"encoding/json"
	"net/http"
	"strconv"
	"strings"

	"server-access-manager/internal/middleware"
	"server-access-manager/internal/models"
	"server-access-manager/internal/repository"
)

type ServerHandler struct {
	servers *repository.ServerRepository
	envs    *repository.EnvironmentRepository
}

func NewServerHandler(servers *repository.ServerRepository, envs *repository.EnvironmentRepository) *ServerHandler {
	return &ServerHandler{servers: servers, envs: envs}
}

func (h *ServerHandler) List(w http.ResponseWriter, r *http.Request) {
	servers, err := h.servers.GetAll()
	if err != nil {
		http.Error(w, "Failed to load servers", http.StatusInternalServerError)
		return
	}

	user := middleware.GetUser(r)
	data := map[string]interface{}{
		"Servers":   servers,
		"User":      user,
		"CSRFToken": middleware.CSRFToken(r),
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(data)
}

func (h *ServerHandler) Create(w http.ResponseWriter, r *http.Request) {
	if err := r.ParseForm(); err != nil {
		http.Error(w, "Invalid form data", http.StatusBadRequest)
		return
	}

	server := &models.Server{
		Name:        r.FormValue("name"),
		Hostname:    r.FormValue("hostname"),
		Username:    r.FormValue("username"),
		AuthType:    r.FormValue("auth_type"),
		Password:    r.FormValue("password"),
		PrivateKey:  r.FormValue("private_key"),
		Passphrase:  r.FormValue("passphrase"),
		Description: r.FormValue("description"),
		OSType:      r.FormValue("os_type"),
		Notes:       r.FormValue("notes"),
	}

	if portStr := r.FormValue("port"); portStr != "" {
		port, _ := strconv.Atoi(portStr)
		server.Port = port
	} else {
		server.Port = 22
	}

	if envIDStr := r.FormValue("environment_id"); envIDStr != "" {
		envID, _ := strconv.ParseInt(envIDStr, 10, 64)
		server.EnvironmentID = &envID
	}

	if tags := r.FormValue("tags"); tags != "" {
		server.Tags = strings.Split(tags, ",")
		for i := range server.Tags {
			server.Tags[i] = strings.TrimSpace(server.Tags[i])
		}
	}

	created, err := h.servers.Create(server)
	if err != nil {
		http.Error(w, "Failed to create server", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(created)
}

func (h *ServerHandler) Update(w http.ResponseWriter, r *http.Request) {
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

	server := &models.Server{
		ID:          id,
		Name:        r.FormValue("name"),
		Hostname:    r.FormValue("hostname"),
		Username:    r.FormValue("username"),
		AuthType:    r.FormValue("auth_type"),
		Password:    r.FormValue("password"),
		PrivateKey:  r.FormValue("private_key"),
		Passphrase:  r.FormValue("passphrase"),
		Description: r.FormValue("description"),
		OSType:      r.FormValue("os_type"),
		Notes:       r.FormValue("notes"),
	}

	if portStr := r.FormValue("port"); portStr != "" {
		port, _ := strconv.Atoi(portStr)
		server.Port = port
	}

	if envIDStr := r.FormValue("environment_id"); envIDStr != "" {
		envID, _ := strconv.ParseInt(envIDStr, 10, 64)
		server.EnvironmentID = &envID
	}

	if tags := r.FormValue("tags"); tags != "" {
		server.Tags = strings.Split(tags, ",")
		for i := range server.Tags {
			server.Tags[i] = strings.TrimSpace(server.Tags[i])
		}
	}

	if err := h.servers.Update(server); err != nil {
		http.Error(w, "Failed to update server", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func (h *ServerHandler) Delete(w http.ResponseWriter, r *http.Request) {
	idStr := r.URL.Query().Get("id")
	id, err := strconv.ParseInt(idStr, 10, 64)
	if err != nil {
		http.Error(w, "Invalid ID", http.StatusBadRequest)
		return
	}

	if err := h.servers.Delete(id); err != nil {
		http.Error(w, "Failed to delete server", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}
