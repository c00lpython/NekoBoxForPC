package models

type DNSType string

const (
	DNSUDP   DNSType = "udp"
	DNSTCP   DNSType = "tcp"
	DNSDoH   DNSType = "https"
	DNSDoT   DNSType = "tls"
	DNSDoQ   DNSType = "quic"
	DNSFake  DNSType = "fakeip"
	DNSLocal DNSType = "local"
)

type DNSServer struct {
	Tag            string   `json:"tag"`
	Address        string   `json:"address"`
	Type           DNSType  `json:"type"`
	Detour         string   `json:"detour,omitempty"` // outbound tag to route DNS query
	Domains        []string `json:"domains,omitempty"`
	Rules          []string `json:"rules,omitempty"`
	Strategy       string   `json:"strategy,omitempty"` // ipv4_only, prefer_ipv4, prefer_ipv6
	ClientSubnet   string   `json:"client_subnet,omitempty"`
}

type DNSConfig struct {
	Servers       []*DNSServer `json:"servers"`
	Rules         []*DNSRule   `json:"rules,omitempty"`
	Final         string       `json:"final"`
	Strategy      string       `json:"strategy,omitempty"`
	FakeIPEnabled bool         `json:"fake_ip_enabled,omitempty"`
	FakeIPRange   string       `json:"fake_ip_range,omitempty"`
}

type DNSRule struct {
	Server       string   `json:"server"`
	Domains      []string `json:"domains,omitempty"`
	DomainSuffix []string `json:"domain_suffix,omitempty"`
	GeoSite      []string `json:"geosite,omitempty"`
	Invert       bool     `json:"invert,omitempty"`
	OutboundTags []string `json:"outbound_tags,omitempty"`
}
