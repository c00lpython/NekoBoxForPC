package compiler

import (
	"nekobox-for-pc/pkg/models"
)

type DNSBuilder struct{}

func NewDNSBuilder() *DNSBuilder {
	return &DNSBuilder{}
}

// BuildDNSConfig builds the sing-box 1.12+ / 1.14+ "dns" configuration block.
func (b *DNSBuilder) BuildDNSConfig(dns *models.DNSConfig, proxyOutboundTag string) map[string]interface{} {
	dnsMap := make(map[string]interface{})

	var servers []map[string]interface{}

	// Default remote DoH server (modern sing-box format)
	remoteDNS := map[string]interface{}{
		"tag":    "dns-remote",
		"type":   "https",
		"server": "1.1.1.1",
		"detour": proxyOutboundTag,
	}
	servers = append(servers, remoteDNS)

	// Direct DNS server for local resolving
	directDNS := map[string]interface{}{
		"tag":    "dns-direct",
		"type":   "local",
		"detour": "direct",
	}
	servers = append(servers, directDNS)

	// FakeIP server if enabled
	if dns != nil && dns.FakeIPEnabled {
		fakeRange := dns.FakeIPRange
		if fakeRange == "" {
			fakeRange = "198.18.0.0/15"
		}
		fakeDNS := map[string]interface{}{
			"tag":         "dns-fakeip",
			"type":        "fakeip",
			"inet4_range": fakeRange,
		}
		servers = append(servers, fakeDNS)
	}

	// Custom user DNS servers
	if dns != nil {
		for _, s := range dns.Servers {
			srv := map[string]interface{}{
				"tag":    s.Tag,
				"server": s.Address,
				"type":   string(s.Type),
			}
			if s.Detour != "" {
				srv["detour"] = s.Detour
			} else {
				srv["detour"] = proxyOutboundTag
			}
			servers = append(servers, srv)
		}
	}

	dnsMap["servers"] = servers

	// Build DNS Rules
	var rules []map[string]interface{}

	// 1. Direct DNS for domains that should bypass proxy
	rules = append(rules, map[string]interface{}{
		"domain_suffix": []string{".ru", ".cn", "yandex.ru", "vk.com", "mail.ru"},
		"server":        "dns-direct",
	})

	// 2. Route rules for FakeIP
	if dns != nil && dns.FakeIPEnabled {
		rules = append(rules, map[string]interface{}{
			"inbound": []string{"tun-in"},
			"server":  "dns-fakeip",
		})
	}

	// 3. User DNS Rules
	if dns != nil {
		for _, r := range dns.Rules {
			ruleMap := map[string]interface{}{
				"server": r.Server,
			}
			if len(r.Domains) > 0 {
				ruleMap["domain"] = r.Domains
			}
			if len(r.DomainSuffix) > 0 {
				ruleMap["domain_suffix"] = r.DomainSuffix
			}
			if r.Invert {
				ruleMap["invert"] = true
			}
			rules = append(rules, ruleMap)
		}
	}

	dnsMap["rules"] = rules
	dnsMap["final"] = "dns-remote"
	dnsMap["strategy"] = "prefer_ipv4"

	return dnsMap
}
