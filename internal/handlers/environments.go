package handlers

import (
	"html/template"
	"net/http"
	"path/filepath"
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
		"Title":        "Environments",
		"Environments": envs,
		"User":         user,
		"CSRFToken":    middleware.CSRFToken(r),
	}

	tmpl, err := template.ParseFiles(
		filepath.Join("internal", "templates", "admin", "environments.html"),
	)
	if err != nil {
		http.Error(w, "Failed to load template", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	tmpl.Execute(w, data)
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

	_, err := h.envs.Create(name, description, color, displayOrder)
	if err != nil {
		http.Error(w, "Failed to create environment", http.StatusInternalServerError)
		return
	}

	// Redirect back to environments list
	http.Redirect(w, r, "/admin/environments", http.StatusSeeOther)
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

	http.Redirect(w, r, "/admin/environments", http.StatusSeeOther)
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

	// Return empty response for HTMX
	w.WriteHeader(http.StatusOK)
}
