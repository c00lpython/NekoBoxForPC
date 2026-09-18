package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"sync"
	"time"

	"nekobox-for-pc/pkg/engine"
	"nekobox-for-pc/pkg/models"
)

type ClashAPIServer struct {
	server  *http.Server
	manager *engine.CoreManager
	proxies []*models.ProxyEntity
	mu      sync.RWMutex
}

func NewClashAPIServer(port int, secret string, manager *engine.CoreManager) *ClashAPIServer {
	if port <= 0 {
		port = 9090
	}

	api := &ClashAPIServer{
		manager: manager,
	}

	mux := http.NewServeMux()

	// 1. Version endpoint
	mux.HandleFunc("/version", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]interface{}{
			"version": "1.10.0 (NekoBoxForPC Core)",
			"premium": true,
		})
	})

	// 2. Traffic endpoint (stream current upload/download speed)
	mux.HandleFunc("/traffic", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		flusher, ok := w.(http.Flusher)
		if !ok {
			http.Error(w, "Streaming unsupported", http.StatusInternalServerError)
			return
		}

		ticker := time.NewTicker(1 * time.Second)
		defer ticker.Stop()

		for {
			select {
			case <-r.Context().Done():
				return
			case <-ticker.C:
				stats := manager.GetStatsCollector().GetStats()
				data, _ := json.Marshal(map[string]int64{
					"up":   stats.UploadSpeed,
					"down": stats.DownloadSpeed,
				})
				_, _ = w.Write(append(data, '\n'))
				flusher.Flush()
			}
		}
	})

	// 3. Proxies endpoint
	mux.HandleFunc("/proxies", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		api.mu.RLock()
		defer api.mu.RUnlock()

		proxiesMap := make(map[string]interface{})
		for _, p := range api.proxies {
			proxiesMap[p.Tag] = map[string]interface{}{
				"name": p.Tag,
				"type": fmt.Sprintf("%d", p.Type),
				"udp":  true,
			}
		}

		current := manager.GetCurrentProxy()
		selectedTag := "DIRECT"
		if current != nil {
			selectedTag = current.Tag
		}

		proxiesMap["GLOBAL"] = map[string]interface{}{
			"name": "GLOBAL",
			"type": "Selector",
			"now":  selectedTag,
		}

		_ = json.NewEncoder(w).Encode(map[string]interface{}{
			"proxies": proxiesMap,
		})
	})

	// 4. Rules endpoint
	mux.HandleFunc("/rules", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]interface{}{
			"rules": []map[string]string{
				{"type": "GeoIP", "payload": "CN", "proxy": "DIRECT"},
				{"type": "GeoIP", "payload": "RU", "proxy": "DIRECT"},
				{"type": "Match", "payload": "", "proxy": "GLOBAL"},
			},
		})
	})

	// 5. Connections endpoint
	mux.HandleFunc("/connections", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		stats := manager.GetStatsCollector().GetStats()
		_ = json.NewEncoder(w).Encode(map[string]interface{}{
			"downloadTotal": stats.DownloadBytes,
			"uploadTotal":   stats.UploadBytes,
			"connections":   []interface{}{},
		})
	})

	api.server = &http.Server{
		Addr:    fmt.Sprintf("127.0.0.1:%d", port),
		Handler: mux,
	}

	return api
}

func (s *ClashAPIServer) SetProxies(proxies []*models.ProxyEntity) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.proxies = proxies
}

func (s *ClashAPIServer) Start() error {
	return s.server.ListenAndServe()
}

func (s *ClashAPIServer) Close() error {
	return s.server.Close()
}
