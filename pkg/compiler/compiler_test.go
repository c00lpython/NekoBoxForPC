package compiler

import (
	"encoding/json"
	"testing"

	"nekobox-for-pc/pkg/models"
)

func TestConfigBuilderCompile(t *testing.T) {
	proxy := &models.ProxyEntity{
		Type:       models.TypeVLESS,
		Tag:        "TestVLESS",
		Server:     "example.com",
		ServerPort: 443,
		UUID:       "12345678-1234-1234-1234-123456789012",
		Security:   "tls",
		SNI:        "example.com",
		Network:    "ws",
		Path:       "/websocket",
	}

	builder := NewConfigBuilder()
	opts := DefaultConfigBuilderOptions()
	opts.MixedPort = 2080
	opts.EnableTUN = true

	configJSON, err := builder.Compile(proxy, nil, nil, opts)
	if err != nil {
		t.Fatalf("unexpected compilation error: %v", err)
	}

	var root map[string]interface{}
	if err := json.Unmarshal([]byte(configJSON), &root); err != nil {
		t.Fatalf("generated config is not valid JSON: %v\nConfig:\n%s", err, configJSON)
	}

	// Verify inbounds
	inbounds, ok := root["inbounds"].([]interface{})
	if !ok || len(inbounds) < 2 {
		t.Errorf("expected at least 2 inbounds (mixed + tun), got %v", inbounds)
	}

	// Verify outbounds
	outbounds, ok := root["outbounds"].([]interface{})
	if !ok || len(outbounds) < 3 {
		t.Errorf("expected at least 3 outbounds (proxy, direct, block), got %v", outbounds)
	}

	// Verify DNS and Route exist
	if _, ok := root["dns"]; !ok {
		t.Errorf("expected dns section in config")
	}
	if _, ok := root["route"]; !ok {
		t.Errorf("expected route section in config")
	}
}
