package models

// ProxyType defines the numeric and string protocol types.
type ProxyType int

const (
	TypeSOCKS       ProxyType = 0
	TypeHTTP        ProxyType = 1
	TypeSS          ProxyType = 2
	TypeSSR         ProxyType = 3
	TypeVMess       ProxyType = 4
	TypeVLESS       ProxyType = 5
	TypeTrojan      ProxyType = 6
	TypeTrojanGo    ProxyType = 7
	TypeChain       ProxyType = 8
	TypeNaive       ProxyType = 9
	TypeHysteria    ProxyType = 15
	TypeSSH         ProxyType = 17
	TypeWireGuard   ProxyType = 18
	TypeShadowTLS   ProxyType = 19
	TypeTUIC        ProxyType = 20
	TypeMieru       ProxyType = 21
	TypeAnyTLS      ProxyType = 22
	TypeJuicity     ProxyType = 23
	TypeAmneziaWG   ProxyType = 24
	TypeSnell       ProxyType = 25
	TypeProxySet    ProxyType = 26
	TypeMasterDNS   ProxyType = 27
	TypeByeDPI      ProxyType = 28
	TypeTrustTunnel ProxyType = 29
	TypeDirect      ProxyType = 31
	TypeTailscale   ProxyType = 32
	TypeConfig      ProxyType = 998
)

// ProxyEntity represents a unified outbound profile.
type ProxyEntity struct {
	ID             int64                  `json:"id"`
	GroupID        int64                  `json:"group_id"`
	Type           ProxyType              `json:"type"`
	Tag            string                 `json:"tag"`
	Server         string                 `json:"server"`
	ServerPort     int                    `json:"server_port"`
	Password       string                 `json:"password,omitempty"`
	UUID           string                 `json:"uuid,omitempty"`
	Method         string                 `json:"method,omitempty"`
	Security       string                 `json:"security,omitempty"`
	Network        string                 `json:"network,omitempty"`
	Path           string                 `json:"path,omitempty"`
	Host           string                 `json:"host,omitempty"`
	SNI            string                 `json:"sni,omitempty"`
	ALPN           []string               `json:"alpn,omitempty"`
	Fingerprint    string                 `json:"fingerprint,omitempty"`
	Insecure       bool                   `json:"insecure,omitempty"`
	Flow           string                 `json:"flow,omitempty"`
	PublicKey      string                 `json:"public_key,omitempty"`
	ShortID        string                 `json:"short_id,omitempty"`
	SpiderX        string                 `json:"spider_x,omitempty"`
	WireGuard      *WireGuardOptions      `json:"wireguard,omitempty"`
	AmneziaWG      *AmneziaWGOptions      `json:"amneziawg,omitempty"`
	Hysteria2      *Hysteria2Options      `json:"hysteria2,omitempty"`
	TUIC           *TUICOptions           `json:"tuic,omitempty"`
	ByeDPI         *ByeDPIOptions         `json:"byedpi,omitempty"`
	Multiplex      *MultiplexOptions      `json:"multiplex,omitempty"`
	RawConfig      string                 `json:"raw_config,omitempty"`
	CustomSettings map[string]interface{} `json:"custom_settings,omitempty"`
	Ping           int                    `json:"ping,omitempty"`
	Status         int                    `json:"status,omitempty"`
}

type WireGuardOptions struct {
	PrivateKey   string   `json:"private_key"`
	PeerPublic   string   `json:"peer_public"`
	PreSharedKey string   `json:"preshared_key,omitempty"`
	LocalAddress []string `json:"local_address"`
	MTU          int      `json:"mtu,omitempty"`
	Reserved     string   `json:"reserved,omitempty"`
}

type AmneziaWGOptions struct {
	WireGuardOptions
	Jc   int    `json:"jc,omitempty"`
	Jmin int    `json:"jmin,omitempty"`
	Jmax int    `json:"jmax,omitempty"`
	S1   int    `json:"s1,omitempty"`
	S2   int    `json:"s2,omitempty"`
	H1   string `json:"h1,omitempty"`
	H2   string `json:"h2,omitempty"`
	H3   string `json:"h3,omitempty"`
	H4   string `json:"h4,omitempty"`
}

type Hysteria2Options struct {
	UpMbps   int    `json:"up_mbps,omitempty"`
	DownMbps int    `json:"down_mbps,omitempty"`
	ObfsType string `json:"obfs_type,omitempty"`
	ObfsPass string `json:"obfs_pass,omitempty"`
}

type TUICOptions struct {
	CongestionControl string `json:"congestion_control,omitempty"`
	UDPRelayMode      string `json:"udp_relay_mode,omitempty"`
	ZeroRTT           bool   `json:"zero_rtt,omitempty"`
}

type ByeDPIOptions struct {
	SplitPosition int    `json:"split_position,omitempty"`
	SplitMarker   string `json:"split_marker,omitempty"`
	Disoob        bool   `json:"disoob,omitempty"`
	Auto          bool   `json:"auto,omitempty"`
	Fake          int    `json:"fake,omitempty"`
	TTL           int    `json:"ttl,omitempty"`
	SNI           string `json:"sni,omitempty"`
	CustomArgs    string `json:"custom_args,omitempty"`
}

type MultiplexOptions struct {
	Enabled        bool   `json:"enabled"`
	Protocol       string `json:"protocol,omitempty"` // smux, yamux, h2mux
	MaxConnections int    `json:"max_connections,omitempty"`
	MinStreams     int    `json:"min_streams,omitempty"`
	Padding        bool   `json:"padding,omitempty"`
}
