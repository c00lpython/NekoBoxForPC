package compiler

import (
	"encoding/json"
	"fmt"
	"strings"

	"nekobox-for-pc/pkg/models"
)

type ConfigBuilderOptions struct {
	EnableTUN        bool
	TUNInterfaceName string
	TUNStack         string // "system", "gvisor", "mixed"
	TUNMTU           int
	MixedPort        int
	AllowLAN         bool
	LogLevel         string
	ClashAPIPort     int
	ClashAPISecret   string
	CacheFilePath    string
}

func DefaultConfigBuilderOptions() *ConfigBuilderOptions {
	return &ConfigBuilderOptions{
		EnableTUN:        false,
		TUNInterfaceName: "nekobox-tun",
		TUNStack:         "system",
		TUNMTU:           9000,
		MixedPort:        2080,
		AllowLAN:         false,
		LogLevel:         "info",
		ClashAPIPort:     9090,
		ClashAPISecret:   "",
		CacheFilePath:    "cache.db",
	}
}

type ConfigBuilder struct {
	dnsBuilder     *DNSBuilder
	routingBuilder *RoutingBuilder
}

func NewConfigBuilder() *ConfigBuilder {
	return &ConfigBuilder{
		dnsBuilder:     NewDNSBuilder(),
		routingBuilder: NewRoutingBuilder(),
	}
}

// Compile generates a full sing-box JSON configuration string.
func (b *ConfigBuilder) Compile(
	proxy *models.ProxyEntity,
	routing *models.RoutingProfile,
	dns *models.DNSConfig,
	opts *ConfigBuilderOptions,
) (string, error) {
	if proxy == nil {
		return "", fmt.Errorf("proxy entity cannot be nil")
	}
	if opts == nil {
		opts = DefaultConfigBuilderOptions()
	}

	root := make(map[string]interface{})

	// 1. Log Section
	root["log"] = map[string]interface{}{
		"level":     opts.LogLevel,
		"timestamp": true,
	}

	// 2. Inbounds Section
	var inbounds []map[string]interface{}

	listenAddr := "127.0.0.1"
	if opts.AllowLAN {
		listenAddr = "0.0.0.0"
	}
	mixedInbound := map[string]interface{}{
		"type":        "mixed",
		"tag":         "mixed-in",
		"listen":      listenAddr,
		"listen_port": opts.MixedPort,
	}
	inbounds = append(inbounds, mixedInbound)

	if opts.EnableTUN {
		tunInbound := map[string]interface{}{
			"type":           "tun",
			"tag":            "tun-in",
			"interface_name": opts.TUNInterfaceName,
			"inet4_address":  "172.19.0.1/30",
			"auto_route":     true,
			"strict_route":   true,
			"stack":          opts.TUNStack,
			"mtu":            opts.TUNMTU,
		}
		inbounds = append(inbounds, tunInbound)
	}

	root["inbounds"] = inbounds

	// 3. Outbounds and Endpoints Section
	var outbounds []map[string]interface{}
	var endpoints []map[string]interface{}

	proxyTag := proxy.Tag
	if proxyTag == "" {
		proxyTag = "proxy"
	}

	isEndpoint := (proxy.Type == models.TypeWireGuard || proxy.Type == models.TypeAmneziaWG || proxy.Type == models.TypeTailscale)

	if isEndpoint {
		endpointObj, err := b.buildEndpoint(proxy, proxyTag, opts)
		if err != nil {
			return "", fmt.Errorf("failed to build endpoint: %w", err)
		}
		endpoints = append(endpoints, endpointObj)
	} else {
		mainOutbound, err := b.buildProxyOutbound(proxy, proxyTag, opts)
		if err != nil {
			return "", fmt.Errorf("failed to build proxy outbound: %w", err)
		}
		outbounds = append(outbounds, mainOutbound)
	}

	// Direct Outbound
	outbounds = append(outbounds, map[string]interface{}{
		"type": "direct",
		"tag":  "direct",
	})

	// Block Outbound
	outbounds = append(outbounds, map[string]interface{}{
		"type": "block",
		"tag":  "block",
	})

	root["outbounds"] = outbounds
	if len(endpoints) > 0 {
		root["endpoints"] = endpoints
	}

	// 4. DNS Section
	root["dns"] = b.dnsBuilder.BuildDNSConfig(dns, proxyTag)

	// 5. Route Section
	root["route"] = b.routingBuilder.BuildRouteConfig(routing, proxyTag)

	// 6. Experimental Section (Clash API & Cache)
	experimental := make(map[string]interface{})
	if opts.ClashAPIPort > 0 {
		clashAPI := map[string]interface{}{
			"external_controller": fmt.Sprintf("127.0.0.1:%d", opts.ClashAPIPort),
		}
		if opts.ClashAPISecret != "" {
			clashAPI["secret"] = opts.ClashAPISecret
		}
		experimental["clash_api"] = clashAPI
	}

	if opts.CacheFilePath != "" {
		experimental["cache_file"] = map[string]interface{}{
			"enabled": true,
			"path":    opts.CacheFilePath,
		}
	}

	if len(experimental) > 0 {
		root["experimental"] = experimental
	}

	jsonBytes, err := json.MarshalIndent(root, "", "  ")
	if err != nil {
		return "", fmt.Errorf("failed to marshal sing-box json: %w", err)
	}

	return string(jsonBytes), nil
}

func (b *ConfigBuilder) buildEndpoint(p *models.ProxyEntity, tag string, opts *ConfigBuilderOptions) (map[string]interface{}, error) {
	endpoint := make(map[string]interface{})
	endpoint["tag"] = tag

	switch p.Type {
	case models.TypeWireGuard:
		endpoint["type"] = "wireguard"
		if p.WireGuard != nil {
			endpoint["private_key"] = p.WireGuard.PrivateKey
			if p.WireGuard.MTU > 0 {
				endpoint["mtu"] = p.WireGuard.MTU
			}
			if len(p.WireGuard.LocalAddress) > 0 {
				endpoint["address"] = normalizeAddressList(p.WireGuard.LocalAddress)
			}
			peer := map[string]interface{}{
				"address":     p.Server,
				"port":        p.ServerPort,
				"public_key":  p.WireGuard.PeerPublic,
				"allowed_ips": []string{"0.0.0.0/0", "::/0"},
			}
			if p.WireGuard.PreSharedKey != "" {
				peer["pre_shared_key"] = p.WireGuard.PreSharedKey
			}
			endpoint["peers"] = []map[string]interface{}{peer}
		}

	case models.TypeAmneziaWG:
		endpoint["type"] = "awg"

		if p.AmneziaWG != nil {
			endpoint["private_key"] = p.AmneziaWG.PrivateKey
			if p.AmneziaWG.MTU > 0 {
				endpoint["mtu"] = p.AmneziaWG.MTU
			}
			if len(p.AmneziaWG.LocalAddress) > 0 {
				endpoint["address"] = normalizeAddressList(p.AmneziaWG.LocalAddress)
			}
			if p.AmneziaWG.Jc > 0 {
				endpoint["jc"] = p.AmneziaWG.Jc
			}
			if p.AmneziaWG.Jmin > 0 {
				endpoint["jmin"] = p.AmneziaWG.Jmin
			}
				if p.AmneziaWG.Jmax > 0 {
					endpoint["jmax"] = p.AmneziaWG.Jmax
				}
				if p.AmneziaWG.S1 > 0 {
					endpoint["s1"] = p.AmneziaWG.S1
				}
				if p.AmneziaWG.S2 > 0 {
					endpoint["s2"] = p.AmneziaWG.S2
				}
				if p.AmneziaWG.H1 != "" {
					endpoint["h1"] = p.AmneziaWG.H1
				}
				if p.AmneziaWG.H2 != "" {
					endpoint["h2"] = p.AmneziaWG.H2
				}
				if p.AmneziaWG.H3 != "" {
					endpoint["h3"] = p.AmneziaWG.H3
				}
				if p.AmneziaWG.H4 != "" {
					endpoint["h4"] = p.AmneziaWG.H4
				}

			peer := map[string]interface{}{
				"address":     p.Server,
				"port":        p.ServerPort,
				"public_key":  p.AmneziaWG.PeerPublic,
				"allowed_ips": []string{"0.0.0.0/0", "::/0"},
			}
			if p.AmneziaWG.PreSharedKey != "" {
				peer["pre_shared_key"] = p.AmneziaWG.PreSharedKey
			}
			endpoint["peers"] = []map[string]interface{}{peer}
		}
	}

	return endpoint, nil
}

func (b *ConfigBuilder) buildProxyOutbound(p *models.ProxyEntity, tag string, opts *ConfigBuilderOptions) (map[string]interface{}, error) {
	outbound := make(map[string]interface{})
	outbound["tag"] = tag

	switch p.Type {
	case models.TypeVLESS:
		outbound["type"] = "vless"
		outbound["server"] = p.Server
		outbound["server_port"] = p.ServerPort
		outbound["uuid"] = p.UUID
		if p.Flow != "" {
			outbound["flow"] = p.Flow
		}
		b.applyTLSAndTransport(outbound, p)

	case models.TypeVMess:
		outbound["type"] = "vmess"
		outbound["server"] = p.Server
		outbound["server_port"] = p.ServerPort
		outbound["uuid"] = p.UUID
		outbound["security"] = "auto"
		if p.Method != "" {
			outbound["security"] = p.Method
		}
		b.applyTLSAndTransport(outbound, p)

	case models.TypeSS:
		outbound["type"] = "shadowsocks"
		outbound["server"] = p.Server
		outbound["server_port"] = p.ServerPort
		outbound["method"] = p.Method
		outbound["password"] = p.Password

	case models.TypeTrojan:
		outbound["type"] = "trojan"
		outbound["server"] = p.Server
		outbound["server_port"] = p.ServerPort
		outbound["password"] = p.Password
		b.applyTLSAndTransport(outbound, p)

	case models.TypeHysteria:
		outbound["type"] = "hysteria2"
		outbound["server"] = p.Server
		outbound["server_port"] = p.ServerPort
		outbound["password"] = p.Password
		tls := map[string]interface{}{
			"enabled":     true,
			"server_name": p.SNI,
			"insecure":    p.Insecure,
		}
		outbound["tls"] = tls
		if p.Hysteria2 != nil {
			if p.Hysteria2.UpMbps > 0 {
				outbound["up_mbps"] = p.Hysteria2.UpMbps
			}
			if p.Hysteria2.DownMbps > 0 {
				outbound["down_mbps"] = p.Hysteria2.DownMbps
			}
			if p.Hysteria2.ObfsType != "" {
				outbound["obfs"] = map[string]interface{}{
					"type":     p.Hysteria2.ObfsType,
					"password": p.Hysteria2.ObfsPass,
				}
			}
		}

	case models.TypeTUIC:
		outbound["type"] = "tuic"
		outbound["server"] = p.Server
		outbound["server_port"] = p.ServerPort
		outbound["uuid"] = p.UUID
		outbound["password"] = p.Password
		outbound["tls"] = map[string]interface{}{
			"enabled":     true,
			"server_name": p.SNI,
			"insecure":    p.Insecure,
		}
		if p.TUIC != nil && p.TUIC.CongestionControl != "" {
			outbound["congestion_control"] = p.TUIC.CongestionControl
		}

	default:
		outbound["type"] = "direct"
	}

	return outbound, nil
}

func (b *ConfigBuilder) applyTLSAndTransport(outbound map[string]interface{}, p *models.ProxyEntity) {
	if p.Security == "tls" || p.Security == "reality" {
		tls := map[string]interface{}{
			"enabled":  true,
			"insecure": p.Insecure,
		}
		if p.SNI != "" {
			tls["server_name"] = p.SNI
		}
		if len(p.ALPN) > 0 {
			tls["alpn"] = p.ALPN
		}
		if p.Fingerprint != "" {
			tls["utls"] = map[string]interface{}{
				"enabled":     true,
				"fingerprint": p.Fingerprint,
			}
		}
		if p.Security == "reality" {
			tls["reality"] = map[string]interface{}{
				"enabled":    true,
				"public_key": p.PublicKey,
				"short_id":   p.ShortID,
			}
		}
		outbound["tls"] = tls
	}

	switch p.Network {
	case "ws":
		outbound["transport"] = map[string]interface{}{
			"type": "ws",
			"path": p.Path,
			"headers": map[string]string{
				"Host": p.Host,
			},
		}
	case "grpc":
		outbound["transport"] = map[string]interface{}{
			"type":         "grpc",
			"service_name": p.Path,
		}
	case "http", "h2":
		outbound["transport"] = map[string]interface{}{
			"type": "http",
			"host": []string{p.Host},
			"path": p.Path,
		}
	case "httpupgrade":
		outbound["transport"] = map[string]interface{}{
			"type": "httpupgrade",
			"host": p.Host,
			"path": p.Path,
		}
	}
}

func normalizeAddressList(addrs []string) []string {
	var res []string
	for _, a := range addrs {
		a = strings.TrimSpace(a)
		if a == "" {
			continue
		}
		if !strings.Contains(a, "/") {
			if strings.Contains(a, ":") {
				a = a + "/128"
			} else {
				a = a + "/32"
			}
		}
		res = append(res, a)
	}
	return res
}
