package compiler

import (
	"nekobox-for-pc/pkg/models"
)

type RoutingBuilder struct{}

func NewRoutingBuilder() *RoutingBuilder {
	return &RoutingBuilder{}
}

// BuildRouteConfig constructs the sing-box "route" configuration block.
func (b *RoutingBuilder) BuildRouteConfig(profile *models.RoutingProfile, proxyOutboundTag string) map[string]interface{} {
	routeMap := make(map[string]interface{})

	var rules []map[string]interface{}

	// Sniff rule
	rules = append(rules, map[string]interface{}{
		"action": "sniff",
	})

	// DNS Inbound rule -> hijack-dns action
	rules = append(rules, map[string]interface{}{
		"protocol": "dns",
		"action":   "hijack-dns",
	})

	// Private / LAN IP rule -> Direct
	rules = append(rules, map[string]interface{}{
		"ip_is_private": true,
		"outbound":      "direct",
	})

	// Custom User Routing Rules
	if profile != nil {
		for _, r := range profile.Rules {
			if !r.Enabled {
				continue
			}

			ruleMap := make(map[string]interface{})
			outbound := string(r.Action)
			if r.Action == models.ActionProxy {
				if r.OutboundTag != "" {
					outbound = r.OutboundTag
				} else {
					outbound = proxyOutboundTag
				}
			}
			ruleMap["outbound"] = outbound

			if len(r.Domains) > 0 {
				ruleMap["domain"] = r.Domains
			}
			if len(r.DomainSuffix) > 0 {
				ruleMap["domain_suffix"] = r.DomainSuffix
			}
			if len(r.DomainKeyword) > 0 {
				ruleMap["domain_keyword"] = r.DomainKeyword
			}
			if len(r.DomainRegex) > 0 {
				ruleMap["domain_regex"] = r.DomainRegex
			}
			if len(r.GeoIP) > 0 {
				ruleMap["geoip"] = r.GeoIP
			}
			if len(r.GeoSite) > 0 {
				ruleMap["geosite"] = r.GeoSite
			}
			if len(r.IPCIDR) > 0 {
				ruleMap["ip_cidr"] = r.IPCIDR
			}
			if len(r.Port) > 0 {
				ruleMap["port"] = r.Port
			}
			if len(r.PortRange) > 0 {
				ruleMap["port_range"] = r.PortRange
			}
			if len(r.ProcessName) > 0 {
				ruleMap["process_name"] = r.ProcessName
			}
			if len(r.Protocol) > 0 {
				ruleMap["protocol"] = r.Protocol
			}
			if r.Invert {
				ruleMap["invert"] = true
			}

			rules = append(rules, ruleMap)
		}
	}

	// Default direct rules for domestic services if no custom rules override
	if profile == nil || len(profile.Rules) == 0 {
		rules = append(rules, map[string]interface{}{
			"domain_suffix": []string{".ru", ".cn", "yandex.ru", "vk.com", "mail.ru", "gosuslugi.ru"},
			"outbound":      "direct",
		})
	}

	routeMap["rules"] = rules
	routeMap["auto_detect_interface"] = true
	routeMap["final"] = proxyOutboundTag
	routeMap["default_domain_resolver"] = "dns-direct"

	return routeMap
}
