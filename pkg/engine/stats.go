package engine

import (
	"fmt"
	"net"
	"net/http"
	"sync"
	"time"
)

type TrafficStats struct {
	UploadBytes   int64 `json:"upload_bytes"`
	DownloadBytes int64 `json:"download_bytes"`
	UploadSpeed   int64 `json:"upload_speed"`   // bytes/sec
	DownloadSpeed int64 `json:"download_speed"` // bytes/sec
	LastUpdated   int64 `json:"last_updated"`
}

type StatsCollector struct {
	mu           sync.RWMutex
	stats        TrafficStats
	lastUpBytes  int64
	lastDownBytes int64
	lastCheck    time.Time
}

func NewStatsCollector() *StatsCollector {
	return &StatsCollector{
		lastCheck: time.Now(),
	}
}

// RecordTraffic updates the cumulative uploaded and downloaded bytes and calculates current speed.
func (c *StatsCollector) RecordTraffic(up, down int64) {
	c.mu.Lock()
	defer c.mu.Unlock()

	now := time.Now()
	elapsed := now.Sub(c.lastCheck).Seconds()

	if elapsed >= 1.0 {
		c.stats.UploadSpeed = int64(float64(up-c.lastUpBytes) / elapsed)
		c.stats.DownloadSpeed = int64(float64(down-c.lastDownBytes) / elapsed)
		c.lastUpBytes = up
		c.lastDownBytes = down
		c.lastCheck = now
	}

	c.stats.UploadBytes = up
	c.stats.DownloadBytes = down
	c.stats.LastUpdated = now.Unix()
}

func (c *StatsCollector) GetStats() TrafficStats {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.stats
}

// TestDelay checks connection latency to a target host or URL (TCP or HTTP ping).
func TestDelay(target string, timeout time.Duration) (int64, error) {
	if timeout <= 0 {
		timeout = 3 * time.Second
	}

	start := time.Now()

	if target == "" {
		target = "http://www.gstatic.com/generate_204"
	}

	client := &http.Client{
		Timeout: timeout,
	}

	resp, err := client.Get(target)
	if err != nil {
		// Fallback to TCP ping if target is host:port
		conn, errTcp := net.DialTimeout("tcp", target, timeout)
		if errTcp != nil {
			return -1, fmt.Errorf("ping failed: %w", err)
		}
		_ = conn.Close()
	} else {
		_ = resp.Body.Close()
	}

	latency := time.Since(start).Milliseconds()
	return latency, nil
}
