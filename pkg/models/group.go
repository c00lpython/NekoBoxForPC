package models

type GroupType int

const (
	GroupBasic        GroupType = 0
	GroupSubscription GroupType = 1
)

type GroupOrder int

const (
	OrderOrigin GroupOrder = 0
	OrderName   GroupOrder = 1
	OrderPing   GroupOrder = 2
)

// ProxyGroup represents a collection of proxies or a remote subscription.
type ProxyGroup struct {
	ID               int64             `json:"id"`
	Name             string            `json:"name"`
	Type             GroupType         `json:"type"`
	Order            GroupOrder        `json:"order"`
	IsSelector       bool              `json:"is_selector"`
	SubscriptionLink string            `json:"subscription_link,omitempty"`
	AutoUpdate       bool              `json:"auto_update,omitempty"`
	AutoUpdateDelay  int               `json:"auto_update_delay,omitempty"` // in minutes
	LastUpdated      int64             `json:"last_updated,omitempty"`
	FilterRegex      string            `json:"filter_regex,omitempty"`
	FilterMode       string            `json:"filter_mode,omitempty"` // "include", "exclude", "disabled"
	CustomUserAgent  string            `json:"custom_user_agent,omitempty"`
	Proxies          []*ProxyEntity    `json:"proxies,omitempty"`
	SelectedProxyTag string            `json:"selected_proxy_tag,omitempty"`
}
