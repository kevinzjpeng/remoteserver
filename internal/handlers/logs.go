package handlers

import (
	"encoding/json"
	"net/http"
	"strconv"

	"server-access-manager/internal/middleware"
	"server-access-manager/internal/repository"
)

type LogHandler struct {
	logs *repository.AccessLogRepository
}

func NewLogHandler(logs *repository.AccessLogRepository) *LogHandler {
	return &LogHandler{logs: logs}
}

func (h *LogHandler) List(w http.ResponseWriter, r *http.Request) {
	limitStr := r.URL.Query().Get("limit")
	limit, _ := strconv.Atoi(limitStr)
	if limit <= 0 {
		limit = 100
	}

	logs, err := h.logs.GetAll(limit)
	if err != nil {
		http.Error(w, "Failed to load logs", http.StatusInternalServerError)
		return
	}

	user := middleware.GetUser(r)
	data := map[string]interface{}{
		"Logs":      logs,
		"User":      user,
		"CSRFToken": middleware.CSRFToken(r),
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(data)
}

func (h *LogHandler) GetByServer(w http.ResponseWriter, r *http.Request) {
	serverIDStr := r.URL.Query().Get("server_id")
	serverID, err := strconv.ParseInt(serverIDStr, 10, 64)
	if err != nil {
		http.Error(w, "Invalid server ID", http.StatusBadRequest)
		return
	}

	limitStr := r.URL.Query().Get("limit")
	limit, _ := strconv.Atoi(limitStr)
	if limit <= 0 {
		limit = 50
	}

	logs, err := h.logs.GetByServerID(serverID, limit)
	if err != nil {
		http.Error(w, "Failed to load logs", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(logs)
}
