package parsers

import (
	"encoding/json"
	"fmt"
	"strings"

	"nekobox-for-pc/pkg/models"
)

type XrayConfig struct {
	Outbounds []map[string]interface{} `json:"outbounds"`
}

// ParseXray parses an Xray/V2Ray JSON config string.
func ParseXray(content string) ([]*models.ProxyEntity, error) {
	var config XrayConfig
	if err := json.Unmarshal([]byte(content), &config); err != nil {
		return nil, fmt.Errorf("failed to parse xray json: %w", err)
	}

	var results []*models.ProxyEntity
	for _, ob := range config.Outbounds {
		protocol, _ := ob["protocol"].(string)
		tag, _ := ob["tag"].(string)
		if protocol == "" || protocol == "freedom" || protocol == "blackhole" || protocol == "dns" {
			continue
		}

		entity := &models.ProxyEntity{
			Tag: tag,
		}

		settings, _ := ob["settings"].(map[string]interface{})
		streamSettings, _ := ob["streamSettings"].(map[string]interface{})

		if streamSettings != nil {
			entity.Network, _ = streamSettings["network"].(string)
			entity.Security, _ = streamSettings["security"].(string)
			if tlsSettings, ok := streamSettings["tlsSettings"].(map[string]interface{}); ok {
				entity.SNI, _ = tlsSettings["serverName"].(string)
				entity.Fingerprint, _ = tlsSettings["fingerprint"].(string)
			}
			if realitySettings, ok := streamSettings["realitySettings"].(map[string]interface{}); ok {
				entity.Security = "reality"
				entity.SNI, _ = realitySettings["serverName"].(string)
				entity.PublicKey, _ = realitySettings["publicKey"].(string)
				entity.ShortID, _ = realitySettings["shortId"].(string)
			}
			if wsSettings, ok := streamSettings["wsSettings"].(map[string]interface{}); ok {
				entity.Path, _ = wsSettings["path"].(string)
				if headers, ok := wsSettings["headers"].(map[string]interface{}); ok {
					entity.Host, _ = headers["Host"].(string)
				}
			}
		}

		switch protocol {
		case "vless":
			entity.Type = models.TypeVLESS
			if vnext, ok := settings["vnext"].([]interface{}); ok && len(vnext) > 0 {
				if srv, ok := vnext[0].(map[string]interface{}); ok {
					entity.Server, _ = srv["address"].(string)
					entity.ServerPort = int(srv["port"].(float64))
					if users, ok := srv["users"].([]interface{}); ok && len(users) > 0 {
						if u, ok := users[0].(map[string]interface{}); ok {
							entity.UUID, _ = u["id"].(string)
							entity.Flow, _ = u["flow"].(string)
						}
					}
				}
			}

		case "vmess":
			entity.Type = models.TypeVMess
			if vnext, ok := settings["vnext"].([]interface{}); ok && len(vnext) > 0 {
				if srv, ok := vnext[0].(map[string]interface{}); ok {
					entity.Server, _ = srv["address"].(string)
					entity.ServerPort = int(srv["port"].(float64))
					if users, ok := srv["users"].([]interface{}); ok && len(users) > 0 {
						if u, ok := users[0].(map[string]interface{}); ok {
							entity.UUID, _ = u["id"].(string)
							entity.Method, _ = u["security"].(string)
						}
					}
				}
			}

		case "shadowsocks":
			entity.Type = models.TypeSS
			if servers, ok := settings["servers"].([]interface{}); ok && len(servers) > 0 {
				if srv, ok := servers[0].(map[string]interface{}); ok {
					entity.Server, _ = srv["address"].(string)
					entity.ServerPort = int(srv["port"].(float64))
					entity.Method, _ = srv["method"].(string)
					entity.Password, _ = srv["password"].(string)
				}
			}

		case "trojan":
			entity.Type = models.TypeTrojan
			if servers, ok := settings["servers"].([]interface{}); ok && len(servers) > 0 {
				if srv, ok := servers[0].(map[string]interface{}); ok {
					entity.Server, _ = srv["address"].(string)
					entity.ServerPort = int(srv["port"].(float64))
					entity.Password, _ = srv["password"].(string)
				}
			}
		}

		if entity.Server != "" {
			if entity.Tag == "" {
				entity.Tag = fmt.Sprintf("%s-%s:%d", strings.ToUpper(protocol), entity.Server, entity.ServerPort)
			}
			results = append(results, entity)
		}
	}

	return results, nil
}
