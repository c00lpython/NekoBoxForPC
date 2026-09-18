package parsers

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/url"
	"strconv"
	"strings"

	"nekobox-for-pc/pkg/models"
)

// ParseUniversal parses any supported VPN/Proxy URI string into a models.ProxyEntity.
func ParseUniversal(link string) (*models.ProxyEntity, error) {
	link = strings.TrimSpace(link)
	if link == "" {
		return nil, fmt.Errorf("empty link")
	}

	lower := strings.ToLower(link)
	switch {
	case strings.HasPrefix(lower, "vless://"):
		return parseVLESS(link)
	case strings.HasPrefix(lower, "vmess://"):
		return parseVMess(link)
	case strings.HasPrefix(lower, "ss://"):
		return parseShadowsocks(link)
	case strings.HasPrefix(lower, "trojan://"):
		return parseTrojan(link)
	case strings.HasPrefix(lower, "hysteria2://") || strings.HasPrefix(lower, "hy2://"):
		return parseHysteria2(link)
	case strings.HasPrefix(lower, "hysteria://"):
		return parseHysteria1(link)
	case strings.HasPrefix(lower, "tuic://"):
		return parseTUIC(link)
	case strings.HasPrefix(lower, "amneziawg://") || strings.HasPrefix(lower, "awg://"):
		return parseAmneziaWG(link)
	case strings.HasPrefix(lower, "wireguard://") || strings.HasPrefix(lower, "wg://"):
		return parseWireGuard(link)
	case strings.HasPrefix(lower, "byedpi://"):
		return parseByeDPI(link)
	default:
		return nil, fmt.Errorf("unsupported protocol scheme: %s", link)
	}
}

func parseVLESS(link string) (*models.ProxyEntity, error) {
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid vless url: %w", err)
	}

	port, _ := strconv.Atoi(u.Port())
	if port == 0 {
		port = 443
	}

	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("VLESS-%s:%d", u.Hostname(), port)
	}

	entity := &models.ProxyEntity{
		Type:        models.TypeVLESS,
		Tag:         tag,
		Server:      u.Hostname(),
		ServerPort:  port,
		UUID:        u.User.Username(),
		Network:     q.Get("type"),
		Security:    q.Get("security"),
		Path:        q.Get("path"),
		Host:        q.Get("host"),
		SNI:         q.Get("sni"),
		Flow:        q.Get("flow"),
		Fingerprint: q.Get("fp"),
		PublicKey:   q.Get("pbk"),
		ShortID:     q.Get("sid"),
		SpiderX:     q.Get("spx"),
		Insecure:    q.Get("allowInsecure") == "1" || q.Get("allowInsecure") == "true",
	}

	if entity.Network == "" {
		entity.Network = "tcp"
	}
	if entity.Security == "" {
		entity.Security = "none"
	}
	if alpn := q.Get("alpn"); alpn != "" {
		entity.ALPN = strings.Split(alpn, ",")
	}

	return entity, nil
}

func parseVMess(link string) (*models.ProxyEntity, error) {
	raw := strings.TrimPrefix(link, "vmess://")
	// Try standard base64 JSON first (v2rayN format)
	decodedBytes, err := base64Decode(raw)
	if err == nil {
		var vmessJSON struct {
			V    interface{} `json:"v"`
			PS   string      `json:"ps"`
			Add  string      `json:"add"`
			Port interface{} `json:"port"`
			ID   string      `json:"id"`
			Aid  interface{} `json:"aid"`
			Scy  string      `json:"scy"`
			Net  string      `json:"net"`
			Type string      `json:"type"`
			Host string      `json:"host"`
			Path string      `json:"path"`
			TLS  string      `json:"tls"`
			SNI  string      `json:"sni"`
			Alpn string      `json:"alpn"`
			Fp   string      `json:"fp"`
		}

		if err := json.Unmarshal(decodedBytes, &vmessJSON); err == nil && vmessJSON.Add != "" {
			port := 0
			switch p := vmessJSON.Port.(type) {
			case float64:
				port = int(p)
			case string:
				port, _ = strconv.Atoi(p)
			}

			tag := vmessJSON.PS
			if tag == "" {
				tag = fmt.Sprintf("VMess-%s:%d", vmessJSON.Add, port)
			}

			security := "none"
			if vmessJSON.TLS == "tls" || vmessJSON.TLS == "1" {
				security = "tls"
			}

			entity := &models.ProxyEntity{
				Type:        models.TypeVMess,
				Tag:         tag,
				Server:      vmessJSON.Add,
				ServerPort:  port,
				UUID:        vmessJSON.ID,
				Method:      vmessJSON.Scy,
				Network:     vmessJSON.Net,
				Path:        vmessJSON.Path,
				Host:        vmessJSON.Host,
				SNI:         vmessJSON.SNI,
				Security:    security,
				Fingerprint: vmessJSON.Fp,
			}
			if vmessJSON.Alpn != "" {
				entity.ALPN = strings.Split(vmessJSON.Alpn, ",")
			}
			return entity, nil
		}
	}

	// Fallback to URL format (DuckSoft/Xray format)
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid vmess url: %w", err)
	}

	port, _ := strconv.Atoi(u.Port())
	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("VMess-%s:%d", u.Hostname(), port)
	}

	return &models.ProxyEntity{
		Type:        models.TypeVMess,
		Tag:         tag,
		Server:      u.Hostname(),
		ServerPort:  port,
		UUID:        u.User.Username(),
		Network:     q.Get("type"),
		Security:    q.Get("security"),
		Path:        q.Get("path"),
		Host:        q.Get("host"),
		SNI:         q.Get("sni"),
		Fingerprint: q.Get("fp"),
	}, nil
}

func parseShadowsocks(link string) (*models.ProxyEntity, error) {
	tag := ""
	cleanLink := link
	if idx := strings.Index(link, "#"); idx != -1 {
		tag, _ = url.QueryUnescape(link[idx+1:])
		cleanLink = link[:idx]
	}

	cleanLink = strings.TrimPrefix(cleanLink, "ss://")

	var server, method, password string
	var port int

	if strings.Contains(cleanLink, "@") {
		// SIP002 format: userinfo@host:port
		parts := strings.SplitN(cleanLink, "@", 2)
		userInfo := parts[0]
		hostPort := parts[1]

		if decoded, err := base64Decode(userInfo); err == nil && strings.Contains(string(decoded), ":") {
			upParts := strings.SplitN(string(decoded), ":", 2)
			method = upParts[0]
			password = upParts[1]
		} else if strings.Contains(userInfo, ":") {
			upParts := strings.SplitN(userInfo, ":", 2)
			method = upParts[0]
			password = upParts[1]
		}

		hpParts := strings.SplitN(hostPort, ":", 2)
		server = hpParts[0]
		if len(hpParts) > 1 {
			port, _ = strconv.Atoi(strings.Split(hpParts[1], "/")[0])
		}
	} else {
		// Legacy base64 format
		decoded, err := base64Decode(cleanLink)
		if err != nil {
			return nil, fmt.Errorf("failed to decode ss link: %w", err)
		}
		str := string(decoded)
		if idx := strings.Index(str, "@"); idx != -1 {
			upParts := strings.SplitN(str[:idx], ":", 2)
			method = upParts[0]
			password = upParts[1]
			hpParts := strings.SplitN(str[idx+1:], ":", 2)
			server = hpParts[0]
			if len(hpParts) > 1 {
				port, _ = strconv.Atoi(hpParts[1])
			}
		}
	}

	if tag == "" {
		tag = fmt.Sprintf("SS-%s:%d", server, port)
	}

	return &models.ProxyEntity{
		Type:       models.TypeSS,
		Tag:        tag,
		Server:     server,
		ServerPort: port,
		Method:     method,
		Password:   password,
	}, nil
}

func parseTrojan(link string) (*models.ProxyEntity, error) {
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid trojan url: %w", err)
	}

	port, _ := strconv.Atoi(u.Port())
	if port == 0 {
		port = 443
	}
	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("Trojan-%s:%d", u.Hostname(), port)
	}

	entity := &models.ProxyEntity{
		Type:        models.TypeTrojan,
		Tag:         tag,
		Server:      u.Hostname(),
		ServerPort:  port,
		Password:    u.User.Username(),
		Network:     q.Get("type"),
		Security:    "tls",
		SNI:         q.Get("sni"),
		Path:        q.Get("path"),
		Host:        q.Get("host"),
		Fingerprint: q.Get("fp"),
		Insecure:    q.Get("allowInsecure") == "1" || q.Get("allowInsecure") == "true",
	}
	if alpn := q.Get("alpn"); alpn != "" {
		entity.ALPN = strings.Split(alpn, ",")
	}
	return entity, nil
}

func parseHysteria2(link string) (*models.ProxyEntity, error) {
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid hysteria2 url: %w", err)
	}

	port, _ := strconv.Atoi(u.Port())
	if port == 0 {
		port = 443
	}
	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("Hy2-%s:%d", u.Hostname(), port)
	}

	up, _ := strconv.Atoi(q.Get("up"))
	down, _ := strconv.Atoi(q.Get("down"))

	password := ""
	if u.User != nil {
		password = u.User.Username()
	}

	return &models.ProxyEntity{
		Type:        models.TypeHysteria,
		Tag:         tag,
		Server:      u.Hostname(),
		ServerPort:  port,
		Password:    password,
		SNI:         q.Get("sni"),
		Insecure:    q.Get("insecure") == "1" || q.Get("insecure") == "true",
		Hysteria2: &models.Hysteria2Options{
			UpMbps:   up,
			DownMbps: down,
			ObfsType: q.Get("obfs"),
			ObfsPass: q.Get("obfs-password"),
		},
	}, nil
}

func parseHysteria1(link string) (*models.ProxyEntity, error) {
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid hysteria url: %w", err)
	}
	port, _ := strconv.Atoi(u.Port())
	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("Hysteria-%s:%d", u.Hostname(), port)
	}

	up, _ := strconv.Atoi(q.Get("upmbps"))
	down, _ := strconv.Atoi(q.Get("downmbps"))

	return &models.ProxyEntity{
		Type:       models.TypeHysteria,
		Tag:        tag,
		Server:     u.Hostname(),
		ServerPort: port,
		SNI:        q.Get("peer"),
		Insecure:   q.Get("insecure") == "1" || q.Get("insecure") == "true",
		Hysteria2: &models.Hysteria2Options{
			UpMbps:   up,
			DownMbps: down,
			ObfsPass: q.Get("obfs"),
		},
	}, nil
}

func parseTUIC(link string) (*models.ProxyEntity, error) {
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid tuic url: %w", err)
	}
	port, _ := strconv.Atoi(u.Port())
	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("TUIC-%s:%d", u.Hostname(), port)
	}

	uuidStr := ""
	pass := ""
	if u.User != nil {
		uuidStr = u.User.Username()
		pass, _ = u.User.Password()
	}

	return &models.ProxyEntity{
		Type:        models.TypeTUIC,
		Tag:         tag,
		Server:      u.Hostname(),
		ServerPort:  port,
		UUID:        uuidStr,
		Password:    pass,
		SNI:         q.Get("sni"),
		Insecure:    q.Get("allow_insecure") == "1" || q.Get("allow_insecure") == "true",
		TUIC: &models.TUICOptions{
			CongestionControl: q.Get("congestion_control"),
			UDPRelayMode:      q.Get("udp_relay_mode"),
			ZeroRTT:           q.Get("zero_rtt_handshake") == "1",
		},
	}, nil
}

func parseAmneziaWG(link string) (*models.ProxyEntity, error) {
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid amneziawg url: %w", err)
	}
	port, _ := strconv.Atoi(u.Port())
	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("AmneziaWG-%s:%d", u.Hostname(), port)
	}

	jc, _ := strconv.Atoi(q.Get("jc"))
	jmin, _ := strconv.Atoi(q.Get("jmin"))
	jmax, _ := strconv.Atoi(q.Get("jmax"))
	s1, _ := strconv.Atoi(q.Get("s1"))
	s2, _ := strconv.Atoi(q.Get("s2"))

	return &models.ProxyEntity{
		Type:       models.TypeAmneziaWG,
		Tag:        tag,
		Server:     u.Hostname(),
		ServerPort: port,
		AmneziaWG: &models.AmneziaWGOptions{
			WireGuardOptions: models.WireGuardOptions{
				PrivateKey:   q.Get("pk"),
				PeerPublic:   q.Get("pub"),
				PreSharedKey: q.Get("psk"),
				LocalAddress: strings.Split(q.Get("ip"), ","),
			},
			Jc:   jc,
			Jmin: jmin,
			Jmax: jmax,
			S1:   s1,
			S2:   s2,
			H1:   q.Get("h1"),
			H2:   q.Get("h2"),
			H3:   q.Get("h3"),
			H4:   q.Get("h4"),
		},
	}, nil
}

func parseWireGuard(link string) (*models.ProxyEntity, error) {
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid wireguard url: %w", err)
	}
	port, _ := strconv.Atoi(u.Port())
	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("WG-%s:%d", u.Hostname(), port)
	}

	return &models.ProxyEntity{
		Type:       models.TypeWireGuard,
		Tag:        tag,
		Server:     u.Hostname(),
		ServerPort: port,
		WireGuard: &models.WireGuardOptions{
			PrivateKey:   q.Get("pk"),
			PeerPublic:   q.Get("pub"),
			PreSharedKey: q.Get("psk"),
			LocalAddress: strings.Split(q.Get("ip"), ","),
		},
	}, nil
}

func parseByeDPI(link string) (*models.ProxyEntity, error) {
	u, err := url.Parse(link)
	if err != nil {
		return nil, fmt.Errorf("invalid byedpi url: %w", err)
	}
	port, _ := strconv.Atoi(u.Port())
	q := u.Query()
	tag := u.Fragment
	if tag == "" {
		tag = fmt.Sprintf("ByeDPI-%s:%d", u.Hostname(), port)
	}

	split, _ := strconv.Atoi(q.Get("split"))
	fake, _ := strconv.Atoi(q.Get("fake"))
	ttl, _ := strconv.Atoi(q.Get("ttl"))

	return &models.ProxyEntity{
		Type:       models.TypeByeDPI,
		Tag:        tag,
		Server:     u.Hostname(),
		ServerPort: port,
		ByeDPI: &models.ByeDPIOptions{
			SplitPosition: split,
			SplitMarker:   q.Get("split_marker"),
			Disoob:        q.Get("disoob") == "1",
			Auto:          q.Get("auto") == "1",
			Fake:          fake,
			TTL:           ttl,
			SNI:           q.Get("sni"),
			CustomArgs:    q.Get("args"),
		},
	}, nil
}

func base64Decode(data string) ([]byte, error) {
	data = strings.TrimSpace(data)
	// Add padding if missing
	if pad := len(data) % 4; pad != 0 {
		data += strings.Repeat("=", 4-pad)
	}
	if b, err := base64.StdEncoding.DecodeString(data); err == nil {
		return b, nil
	}
	return base64.URLEncoding.DecodeString(data)
}
