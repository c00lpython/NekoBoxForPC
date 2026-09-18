package scripting

import (
	"testing"

	"nekobox-for-pc/pkg/models"
)

func TestScriptEngineFilteringAndDeduplication(t *testing.T) {
	engine := NewScriptEngine()

	proxies := []*models.ProxyEntity{
		{Tag: "HK-01", Server: "1.1.1.1", ServerPort: 443, Type: models.TypeVLESS},
		{Tag: "US-01", Server: "2.2.2.2", ServerPort: 443, Type: models.TypeVLESS},
		{Tag: "HK-02", Server: "1.1.1.1", ServerPort: 443, Type: models.TypeVLESS}, // Duplicate server:port
		{Tag: "RU-01", Server: "3.3.3.3", ServerPort: 443, Type: models.TypeSS},
	}

	// 1. Test Filter Include
	hkOnly, err := engine.FilterByRegex(proxies, "^HK", FilterInclude)
	if err != nil {
		t.Fatalf("unexpected filter error: %v", err)
	}
	if len(hkOnly) != 2 {
		t.Errorf("expected 2 HK proxies, got %d", len(hkOnly))
	}

	// 2. Test Deduplication
	deduped := engine.Deduplicate(proxies)
	if len(deduped) != 3 {
		t.Errorf("expected 3 unique proxies, got %d", len(deduped))
	}

	// 3. Test Rename Tags
	renamed := engine.RenameTags(deduped, "^HK", "HongKong")
	if renamed[0].Tag != "HongKong-01" {
		t.Errorf("expected tag HongKong-01, got %s", renamed[0].Tag)
	}
}
