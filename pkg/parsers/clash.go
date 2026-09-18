package parsers

import (
	"fmt"
	"strconv"
	"strings"

	"gopkg.in/yaml.v3"
	"nekobox-for-pc/pkg/models"
)

type ClashConfig struct {
	Port               int                      `yaml:"port,omitempty"`
	SocksPort          int                      `yaml:"socks-port,omitempty"`
	AllowLan           bool                     `yaml:"allow-lan,omitempty"`
	Mode               string                   `yaml:"mode,omitempty"`
	LogLevel           string                   `yaml:"log-level,omitempty"`
	GlobalFingerprint  string                   `yaml:"global-client-fingerprint,omitempty"`
	Proxies            []map[string]interface{} `yaml:"proxies,omitempty"`
	ProxyGroups        []map[string]interface{} `yaml:"proxy-groups,omitempty"`
	Rules              []string                 `yaml:"rules,omitempty"`
}

// ParseClash parses Clash YAML content into a slice of ProxyEntity.
func ParseClash(content string) ([]*models.ProxyEntity, error) {
	var config ClashConfig
	if err := yaml.Unmarshal([]byte(content), &config); err != nil {
		return nil, fmt.Errorf("failed to parse clash yaml: %w", err)
	}

	if len(config.Proxies) == 0 {
		return nil, fmt.Errorf("no proxies found in clash config")
	}

	var results []*models.ProxyEntity
	for _, raw := range config.Proxies {
		entity, err := parseClashProxy(raw, config.GlobalFingerprint)
		if err == nil && entity != nil {
			results = append(results, entity)
		}
	}

	return results, nil
}

func parseClashProxy(proxy map[string]interface{}, globalFp string) (*models.ProxyEntity, error) {
	t, _ := proxy["type"].(string)
	t = strings.ToLower(t)
	name, _ := proxy["name"].(string)
	server, _ := proxy["server"].(string)

	port := 0
	switch p := proxy["port"].(type) {
	case int:
		port = p
	case string:
		port, _ = strconv.Atoi(p)
	}

	if name == "" {
		name = fmt.Sprintf("%s-%s:%d", strings.ToUpper(t), server, port)
	}

	entity := &models.ProxyEntity{
		Tag:        name,
		Server:     server,
		ServerPort: port,
	}

	switch t {
	case "ss":
		entity.Type = models.TypeSS
		entity.Method, _ = proxy["cipher"].(string)
		entity.Password, _ = proxy["password"].(string)

	case "vmess":
		entity.Type = models.TypeVMess
		entity.UUID, _ = proxy["uuid"].(string)
		entity.Method, _ = proxy["cipher"].(string)
		entity.Network, _ = proxy["network"].(string)
		if tls, _ := proxy["tls"].(bool); tls {
			entity.Security = "tls"
		}
		entity.SNI, _ = proxy["servername"].(string)
		if entity.SNI == "" {
			entity.SNI, _ = proxy["sni"].(string)
		}
		entity.Fingerprint, _ = proxy["client-fingerprint"].(string)
		if entity.Fingerprint == "" {
			entity.Fingerprint = globalFp
		}
		if wsOpts, ok := proxy["ws-opts"].(map[string]interface{}); ok {
			entity.Path, _ = wsOpts["path"].(string)
			if headers, ok := wsOpts["headers"].(map[string]interface{}); ok {
				entity.Host, _ = headers["Host"].(string)
			}
		}

	case "vless":
		entity.Type = models.TypeVLESS
		entity.UUID, _ = proxy["uuid"].(string)
		entity.Network, _ = proxy["network"].(string)
		entity.Flow, _ = proxy["flow"].(string)
		if tls, _ := proxy["tls"].(bool); tls {
			entity.Security = "tls"
		}
		if realityOpts, ok := proxy["reality-opts"].(map[string]interface{}); ok {
			entity.Security = "reality"
			entity.PublicKey, _ = realityOpts["public-key"].(string)
			entity.ShortID, _ = realityOpts["short-id"].(string)
		}
		entity.SNI, _ = proxy["servername"].(string)
		entity.Fingerprint, _ = proxy["client-fingerprint"].(string)

	case "trojan":
		entity.Type = models.TypeTrojan
		entity.Password, _ = proxy["password"].(string)
		entity.Security = "tls"
		entity.SNI, _ = proxy["sni"].(string)
		entity.Network, _ = proxy["network"].(string)

	case "hysteria2":
		entity.Type = models.TypeHysteria
		entity.Password, _ = proxy["password"].(string)
		entity.SNI, _ = proxy["sni"].(string)
		up, _ := proxy["up"].(int)
		down, _ := proxy["down"].(int)
		obfs, _ := proxy["obfs"].(string)
		obfsPass, _ := proxy["obfs-password"].(string)
		entity.Hysteria2 = &models.Hysteria2Options{
			UpMbps:   up,
			DownMbps: down,
			ObfsType: obfs,
			ObfsPass: obfsPass,
		}

	case "wireguard":
		entity.Type = models.TypeWireGuard
		privKey, _ := proxy["private-key"].(string)
		pubKey, _ := proxy["public-key"].(string)
		psk, _ := proxy["preshared-key"].(string)
		var ips []string
		if ip, ok := proxy["ip"].(string); ok {
			ips = []string{ip}
		}
		entity.WireGuard = &models.WireGuardOptions{
			PrivateKey:   privKey,
			PeerPublic:   pubKey,
			PreSharedKey: psk,
			LocalAddress: ips,
		}

	case "tuic":
		entity.Type = models.TypeTUIC
		entity.UUID, _ = proxy["uuid"].(string)
		entity.Password, _ = proxy["password"].(string)
		entity.SNI, _ = proxy["sni"].(string)
		cc, _ := proxy["congestion-controller"].(string)
		entity.TUIC = &models.TUICOptions{
			CongestionControl: cc,
		}

	case "socks5":
		entity.Type = models.TypeSOCKS
		entity.Password, _ = proxy["password"].(string)

	case "http":
		entity.Type = models.TypeHTTP
		entity.Password, _ = proxy["password"].(string)

	default:
		return nil, fmt.Errorf("unsupported clash proxy type: %s", t)
	}

	return entity, nil
}
