package handlers

import (
	"encoding/json"
	"net/http"

	"server-access-manager/internal/repository"
)

type TopologyHandler struct {
	servers *repository.ServerRepository
	envs    *repository.EnvironmentRepository
}

func NewTopologyHandler(servers *repository.ServerRepository, envs *repository.EnvironmentRepository) *TopologyHandler {
	return &TopologyHandler{servers: servers, envs: envs}
}

type TopologyNode struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Environment string `json:"environment"`
	Type        string `json:"type"`
	Status      string `json:"status"`
	Username    string `json:"username"`
	Hostname    string `json:"hostname"`
	Port        int    `json:"port"`
	Password    string `json:"password,omitempty"`
}

type TopologyEdge struct {
	Source string `json:"source"`
	Target string `json:"target"`
	Type   string `json:"type"`
	Label  string `json:"label"`
}

type TopologyGroup struct {
	ID      string   `json:"id"`
	Name    string   `json:"name"`
	Color   string   `json:"color"`
	NodeIDs []string `json:"node_ids"`
}

type TopologyResponse struct {
	Nodes    []TopologyNode  `json:"nodes"`
	Edges    []TopologyEdge  `json:"edges"`
	Groups   []TopologyGroup `json:"groups"`
	Metadata struct {
		TotalServers int    `json:"total_servers"`
		Environments int    `json:"environments"`
		GeneratedAt  string `json:"generated_at"`
	} `json:"metadata"`
}

func (h *TopologyHandler) GetTopology(w http.ResponseWriter, r *http.Request) {
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

	response := TopologyResponse{
		Nodes:  []TopologyNode{},
		Edges:  []TopologyEdge{},
		Groups: []TopologyGroup{},
	}

	// Build nodes
	for _, server := range servers {
		envName := "ungrouped"
		if server.Environment != nil {
			envName = server.Environment.Name
		}

		// Fetch password if auth type is password
		password := ""
		if server.AuthType == "password" {
			if pwd, err := h.servers.GetPassword(server.ID); err == nil {
				password = pwd
			}
		}

		node := TopologyNode{
			ID:          string(rune(server.ID)),
			Name:        server.Name,
			Environment: envName,
			Type:        server.OSType,
			Status:      "healthy",
			Username:    server.Username,
			Hostname:    server.Hostname,
			Port:        server.Port,
			Password:    password,
		}
		response.Nodes = append(response.Nodes, node)
	}

	// Build groups by environment
	groupMap := make(map[string]*TopologyGroup)
	for _, env := range envs {
		groupMap[env.Name] = &TopologyGroup{
			ID:      env.Name,
			Name:    env.Name,
			Color:   env.Color,
			NodeIDs: []string{},
		}
	}

	for _, server := range servers {
		envName := "ungrouped"
		if server.Environment != nil {
			envName = server.Environment.Name
		}

		if group, ok := groupMap[envName]; ok {
			group.NodeIDs = append(group.NodeIDs, string(rune(server.ID)))
		}
	}

	for _, group := range groupMap {
		if len(group.NodeIDs) > 0 {
			response.Groups = append(response.Groups, *group)
		}
	}

	// Metadata
	response.Metadata.TotalServers = len(servers)
	response.Metadata.Environments = len(response.Groups)
	response.Metadata.GeneratedAt = "2026-08-07T22:30:00Z"

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}
