package parsers

import (
	"bufio"
	"fmt"
	"strconv"
	"strings"

	"nekobox-for-pc/pkg/models"
)

// ParseWireGuardConf parses a WireGuard / AmneziaWG INI-style .conf document.
func ParseWireGuardConf(content string, fileName string) ([]*models.ProxyEntity, error) {
	scanner := bufio.NewScanner(strings.NewReader(content))

	var currentSection string
	interfaceOpts := make(map[string]string)
	var peers []map[string]string
	var currentPeer map[string]string

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		// Strip comments
		if idx := strings.Index(line, "#"); idx != -1 {
			line = strings.TrimSpace(line[:idx])
		}
		if idx := strings.Index(line, ";"); idx != -1 {
			line = strings.TrimSpace(line[:idx])
		}
		if line == "" {
			continue
		}

		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			currentSection = strings.ToLower(strings.Trim(line, "[]"))
			if currentSection == "peer" {
				currentPeer = make(map[string]string)
				peers = append(peers, currentPeer)
			}
			continue
		}

		parts := strings.SplitN(line, "=", 2)
		if len(parts) == 2 {
			k := strings.ToLower(strings.TrimSpace(parts[0]))
			v := strings.TrimSpace(parts[1])

			if currentSection == "interface" {
				interfaceOpts[k] = v
			} else if currentSection == "peer" && currentPeer != nil {
				currentPeer[k] = v
			}
		}
	}

	if len(peers) == 0 {
		return nil, fmt.Errorf("no [Peer] sections found in config")
	}

	// Determine if it's AmneziaWG
	isAmnezia := false
	amneziaKeys := []string{"jc", "jmin", "jmax", "s1", "s2", "h1", "h2", "h3", "h4"}
	for _, k := range amneziaKeys {
		if _, exists := interfaceOpts[k]; exists {
			isAmnezia = true
			break
		}
	}

	var results []*models.ProxyEntity

	for i, peer := range peers {
		endpoint := peer["endpoint"]
		if endpoint == "" {
			continue
		}

		host := endpoint
		port := 51820
		if strings.Contains(endpoint, ":") {
			hp := strings.Split(endpoint, ":")
			host = hp[0]
			port, _ = strconv.Atoi(hp[1])
		}

		tag := fileName
		if tag == "" {
			tag = fmt.Sprintf("WARP-%s:%d", host, port)
		}
		if len(peers) > 1 {
			tag = fmt.Sprintf("%s (%d)", tag, i+1)
		}

		var addresses []string
		if addr, ok := interfaceOpts["address"]; ok {
			for _, a := range strings.Split(addr, ",") {
				addresses = append(addresses, strings.TrimSpace(a))
			}
		}

		mtu, _ := strconv.Atoi(interfaceOpts["mtu"])
		if mtu == 0 {
			mtu = 1280
		}

		wgOpts := models.WireGuardOptions{
			PrivateKey:   interfaceOpts["privatekey"],
			PeerPublic:   peer["publickey"],
			PreSharedKey: peer["presharedkey"],
			LocalAddress: addresses,
			MTU:          mtu,
			Reserved:     peer["reserved"],
		}

		if isAmnezia {
			jc, _ := strconv.Atoi(interfaceOpts["jc"])
			jmin, _ := strconv.Atoi(interfaceOpts["jmin"])
			jmax, _ := strconv.Atoi(interfaceOpts["jmax"])
			s1, _ := strconv.Atoi(interfaceOpts["s1"])
			s2, _ := strconv.Atoi(interfaceOpts["s2"])

			entity := &models.ProxyEntity{
				Type:       models.TypeAmneziaWG,
				Tag:        tag,
				Server:     host,
				ServerPort: port,
				AmneziaWG: &models.AmneziaWGOptions{
					WireGuardOptions: wgOpts,
					Jc:               jc,
					Jmin:             jmin,
					Jmax:             jmax,
					S1:               s1,
					S2:               s2,
					H1:               interfaceOpts["h1"],
					H2:               interfaceOpts["h2"],
					H3:               interfaceOpts["h3"],
					H4:               interfaceOpts["h4"],
				},
			}
			results = append(results, entity)
		} else {
			entity := &models.ProxyEntity{
				Type:       models.TypeWireGuard,
				Tag:        tag,
				Server:     host,
				ServerPort: port,
				WireGuard:  &wgOpts,
			}
			results = append(results, entity)
		}
	}

	return results, nil
}
