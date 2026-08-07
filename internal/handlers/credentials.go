package handlers

import (
	"encoding/json"
	"net/http"
	"strconv"

	"server-access-manager/internal/middleware"
	"server-access-manager/internal/models"
	"server-access-manager/internal/repository"
)

type CredentialHandler struct {
	creds *repository.CredentialRepository
}

func NewCredentialHandler(creds *repository.CredentialRepository) *CredentialHandler {
	return &CredentialHandler{creds: creds}
}

func (h *CredentialHandler) List(w http.ResponseWriter, r *http.Request) {
	creds, err := h.creds.GetAll()
	if err != nil {
		http.Error(w, "Failed to load credentials", http.StatusInternalServerError)
		return
	}

	user := middleware.GetUser(r)
	data := map[string]interface{}{
		"Credentials": creds,
		"User":        user,
		"CSRFToken":   middleware.CSRFToken(r),
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(data)
}

func (h *CredentialHandler) Create(w http.ResponseWriter, r *http.Request) {
	if err := r.ParseForm(); err != nil {
		http.Error(w, "Invalid form data", http.StatusBadRequest)
		return
	}

	cred := &models.ServiceCredential{
		Name:        r.FormValue("name"),
		ServiceType: r.FormValue("service_type"),
		ConfigJSON:  r.FormValue("config_json"),
		IsActive:    r.FormValue("is_active") == "true",
	}

	if credentials := r.FormValue("credentials"); credentials != "" {
		cred.Credentials = json.RawMessage(credentials)
	}

	created, err := h.creds.Create(cred)
	if err != nil {
		http.Error(w, "Failed to create credential", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(created)
}

func (h *CredentialHandler) Update(w http.ResponseWriter, r *http.Request) {
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

	cred := &models.ServiceCredential{
		ID:          id,
		Name:        r.FormValue("name"),
		ServiceType: r.FormValue("service_type"),
		ConfigJSON:  r.FormValue("config_json"),
		IsActive:    r.FormValue("is_active") == "true",
	}

	if credentials := r.FormValue("credentials"); credentials != "" {
		cred.Credentials = json.RawMessage(credentials)
	}

	if err := h.creds.Update(cred); err != nil {
		http.Error(w, "Failed to update credential", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func (h *CredentialHandler) Delete(w http.ResponseWriter, r *http.Request) {
	idStr := r.URL.Query().Get("id")
	id, err := strconv.ParseInt(idStr, 10, 64)
	if err != nil {
		http.Error(w, "Invalid ID", http.StatusBadRequest)
		return
	}

	if err := h.creds.Delete(id); err != nil {
		http.Error(w, "Failed to delete credential", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}
