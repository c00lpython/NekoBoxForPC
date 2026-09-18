package engine

import (
	"sync"
	"time"

	"nekobox-for-pc/pkg/compiler"
	"nekobox-for-pc/pkg/models"
)

type WatchdogConfig struct {
	MaxRestarts       int
	RestartInterval   time.Duration
	HealthCheckPeriod time.Duration
	AutoRecovery      bool
}

func DefaultWatchdogConfig() *WatchdogConfig {
	return &WatchdogConfig{
		MaxRestarts:       5,
		RestartInterval:   3 * time.Second,
		HealthCheckPeriod: 5 * time.Second,
		AutoRecovery:      true,
	}
}

type CoreWatchdog struct {
	mu           sync.Mutex
	manager      *CoreManager
	config       *WatchdogConfig
	restartCount int
	stopChan     chan struct{}
	running      bool
}

func NewCoreWatchdog(manager *CoreManager, cfg *WatchdogConfig) *CoreWatchdog {
	if cfg == nil {
		cfg = DefaultWatchdogConfig()
	}
	return &CoreWatchdog{
		manager:  manager,
		config:   cfg,
		stopChan: make(chan struct{}),
	}
}

// StartMonitoring runs periodic health checks and triggers auto-recovery on unexpected core exits.
func (w *CoreWatchdog) StartMonitoring(
	proxy *models.ProxyEntity,
	routing *models.RoutingProfile,
	dns *models.DNSConfig,
	opts *compiler.ConfigBuilderOptions,
) {
	w.mu.Lock()
	if w.running {
		w.mu.Unlock()
		return
	}
	w.running = true
	w.stopChan = make(chan struct{})
	w.mu.Unlock()

	go func() {
		ticker := time.NewTicker(w.config.HealthCheckPeriod)
		defer ticker.Stop()

		for {
			select {
			case <-w.stopChan:
				return
			case <-ticker.C:
				w.mu.Lock()
				state := w.manager.GetState()
				if state != StateRunning && w.config.AutoRecovery && w.restartCount < w.config.MaxRestarts {
					w.restartCount++
					w.mu.Unlock()

					time.Sleep(w.config.RestartInterval)
					_ = w.manager.Start(proxy, routing, dns, opts)
					continue
				}
				w.mu.Unlock()
			}
		}
	}()
}

// StopMonitoring disables the watchdog.
func (w *CoreWatchdog) StopMonitoring() {
	w.mu.Lock()
	defer w.mu.Unlock()

	if !w.running {
		return
	}
	w.running = false
	close(w.stopChan)
}
