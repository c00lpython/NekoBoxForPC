package engine

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sync"

	"nekobox-for-pc/pkg/compiler"
	"nekobox-for-pc/pkg/models"
	"nekobox-for-pc/pkg/network/sysproxy"
)

type ProcessState string

const (
	StateStopped ProcessState = "stopped"
	StateRunning ProcessState = "running"
	StateError   ProcessState = "error"
)

type LogHandler func(line string)

type CoreManager struct {
	mu           sync.Mutex
	cmd          *exec.Cmd
	state        ProcessState
	currentProxy *models.ProxyEntity
	compiler     *compiler.ConfigBuilder
	sysProxy     *sysproxy.SystemProxyManager
	configPath   string
	binPath      string
	logHandler   LogHandler
	stats        *StatsCollector
}

func NewCoreManager(binPath string, configDir string) *CoreManager {
	if binPath == "" {
		binPath = filepath.Join(".", "sing-box")
	}
	if configDir == "" {
		configDir = "."
	}
	return &CoreManager{
		binPath:    binPath,
		configPath: filepath.Join(configDir, "config.json"),
		compiler:   compiler.NewConfigBuilder(),
		sysProxy:   sysproxy.NewSystemProxyManager(),
		state:      StateStopped,
		stats:      NewStatsCollector(),
	}
}

func (m *CoreManager) SetLogHandler(handler LogHandler) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.logHandler = handler
}

func (m *CoreManager) GetState() ProcessState {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.state
}

func (m *CoreManager) GetCurrentProxy() *models.ProxyEntity {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.currentProxy
}

func (m *CoreManager) GetStatsCollector() *StatsCollector {
	return m.stats
}

// Start compiles the config and launches the sing-box engine.
func (m *CoreManager) Start(
	proxy *models.ProxyEntity,
	routing *models.RoutingProfile,
	dns *models.DNSConfig,
	opts *compiler.ConfigBuilderOptions,
) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	if m.state == StateRunning {
		return fmt.Errorf("core is already running")
	}

	// 1. Compile sing-box JSON configuration
	configJSON, err := m.compiler.Compile(proxy, routing, dns, opts)
	if err != nil {
		return fmt.Errorf("config compilation failed: %w", err)
	}

	// 2. Write config to disk
	if err := os.WriteFile(m.configPath, []byte(configJSON), 0644); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	// 3. Launch sing-box process
	cmd := exec.Command(m.binPath, "run", "-c", m.configPath)
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to pipe stdout: %w", err)
	}
	cmd.Stderr = cmd.Stdout

	if err := cmd.Start(); err != nil {
		m.state = StateError
		return fmt.Errorf("failed to start sing-box: %w", err)
	}

	m.cmd = cmd
	m.state = StateRunning
	m.currentProxy = proxy

	// 4. Stream process logs in background
	go func() {
		scanner := bufio.NewScanner(stdout)
		for scanner.Scan() {
			line := scanner.Text()
			m.mu.Lock()
			handler := m.logHandler
			m.mu.Unlock()
			if handler != nil {
				handler(line)
			}
		}

		_ = cmd.Wait()
		m.mu.Lock()
		if m.state == StateRunning {
			m.state = StateStopped
			m.cmd = nil
		}
		m.mu.Unlock()
	}()

	return nil
}

// Stop terminates the running core engine and cleans up routes/proxies.
func (m *CoreManager) Stop() error {
	m.mu.Lock()
	defer m.mu.Unlock()

	if m.state != StateRunning || m.cmd == nil || m.cmd.Process == nil {
		m.state = StateStopped
		return nil
	}

	_ = m.sysProxy.ClearSystemProxy()

	err := m.cmd.Process.Kill()
	m.cmd = nil
	m.state = StateStopped
	m.currentProxy = nil

	return err
}
