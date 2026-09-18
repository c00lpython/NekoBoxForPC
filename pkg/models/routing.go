package models

type RuleAction string

const (
	ActionDirect RuleAction = "direct"
	ActionProxy  RuleAction = "proxy"
	ActionBlock  RuleAction = "block"
	ActionByeDPI RuleAction = "byedpi"
	ActionDNS    RuleAction = "dns"
)

type RoutingRule struct {
	ID          int64      `json:"id"`
	Name        string     `json:"name"`
	Action      RuleAction `json:"action"`
	OutboundTag string     `json:"outbound_tag,omitempty"`
	Domains     []string   `json:"domains,omitempty"`
	DomainSuffix []string  `json:"domain_suffix,omitempty"`
	DomainKeyword []string `json:"domain_keyword,omitempty"`
	DomainRegex []string   `json:"domain_regex,omitempty"`
	GeoIP       []string   `json:"geoip,omitempty"`
	GeoSite     []string   `json:"geosite,omitempty"`
	IPCIDR      []string   `json:"ip_cidr,omitempty"`
	Port        []int      `json:"port,omitempty"`
	PortRange   []string   `json:"port_range,omitempty"`
	ProcessName []string   `json:"process_name,omitempty"`
	Protocol    []string   `json:"protocol,omitempty"` // http, tls, quic, etc.
	Invert      bool       `json:"invert,omitempty"`
	Enabled     bool       `json:"enabled"`
}

type RoutingProfile struct {
	ID       int64          `json:"id"`
	Name     string         `json:"name"`
	Rules    []*RoutingRule `json:"rules"`
	Default  RuleAction     `json:"default_action"`
}
