package parsers

import (
	"strings"

	"nekobox-for-pc/pkg/models"
)

// ParseRaw auto-detects content format (Clash YAML, Xray JSON, Base64 URI list, Plaintext URI list)
// and extracts all valid proxy entities.
func ParseRaw(content string) ([]*models.ProxyEntity, error) {
	trimmed := strings.TrimSpace(content)
	if trimmed == "" {
		return nil, nil
	}

	// 1. Try WireGuard / AmneziaWG .conf (look for [Interface])
	if strings.Contains(strings.ToLower(trimmed), "[interface]") {
		if proxies, err := ParseWireGuardConf(trimmed, "WARP"); err == nil && len(proxies) > 0 {
			return proxies, nil
		}
	}

	// 2. Try Clash YAML
	if strings.Contains(trimmed, "proxies:") {
		if proxies, err := ParseClash(trimmed); err == nil && len(proxies) > 0 {
			return proxies, nil
		}
	}

	// 2. Try Xray JSON
	if strings.HasPrefix(trimmed, "{") && strings.Contains(trimmed, "outbounds") {
		if proxies, err := ParseXray(trimmed); err == nil && len(proxies) > 0 {
			return proxies, nil
		}
	}

	// 3. Try Base64 decode
	if decoded, err := base64Decode(trimmed); err == nil {
		decodedStr := string(decoded)
		if list := parseLines(decodedStr); len(list) > 0 {
			return list, nil
		}
	}

	// 4. Try Plaintext line by line
	if list := parseLines(trimmed); len(list) > 0 {
		return list, nil
	}

	return nil, nil
}

func parseLines(text string) []*models.ProxyEntity {
	var results []*models.ProxyEntity
	lines := strings.Split(text, "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") || strings.HasPrefix(line, "//") {
			continue
		}

		if entity, err := ParseUniversal(line); err == nil && entity != nil {
			results = append(results, entity)
		}
	}
	return results
}
