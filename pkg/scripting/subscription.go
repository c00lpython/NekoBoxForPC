package scripting

import (
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
	"time"

	"nekobox-for-pc/pkg/models"
	"nekobox-for-pc/pkg/parsers"
)

type SubscriptionUserInfo struct {
	Upload   int64 `json:"upload"`
	Download int64 `json:"download"`
	Total    int64 `json:"total"`
	Expire   int64 `json:"expire"`
}

type SubscriptionUpdater struct {
	httpClient *http.Client
	engine     *ScriptEngine
}

func NewSubscriptionUpdater(timeout time.Duration) *SubscriptionUpdater {
	if timeout <= 0 {
		timeout = 15 * time.Second
	}
	return &SubscriptionUpdater{
		httpClient: &http.Client{
			Timeout: timeout,
		},
		engine: NewScriptEngine(),
	}
}

// FetchAndParse updates a subscription group from its remote URL.
func (u *SubscriptionUpdater) FetchAndParse(group *models.ProxyGroup) (*SubscriptionUserInfo, error) {
	if group.SubscriptionLink == "" {
		return nil, fmt.Errorf("empty subscription link")
	}

	req, err := http.NewRequest("GET", group.SubscriptionLink, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	ua := group.CustomUserAgent
	if ua == "" {
		ua = "NekoBoxForPC/1.0 (sing-box-plus)"
	}
	req.Header.Set("User-Agent", ua)
	req.Header.Set("Accept", "*/*")

	resp, err := u.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("subscription request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return nil, fmt.Errorf("subscription request returned HTTP %d", resp.StatusCode)
	}

	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	content := string(bodyBytes)
	proxies, err := parsers.ParseRaw(content)
	if err != nil {
		return nil, fmt.Errorf("failed to parse subscription content: %w", err)
	}

	// Apply filtering
	if group.FilterRegex != "" {
		proxies, _ = u.engine.FilterByRegex(proxies, group.FilterRegex, FilterMode(group.FilterMode))
	}

	// Deduplicate and ensure unique names
	proxies = u.engine.Deduplicate(proxies)
	proxies = u.engine.EnsureUniqueTags(proxies)

	group.Proxies = proxies
	group.LastUpdated = time.Now().Unix()

	// Parse subscription userinfo header if present
	var userInfo *SubscriptionUserInfo
	if infoHeader := resp.Header.Get("Subscription-Userinfo"); infoHeader != "" {
		userInfo = parseSubscriptionUserinfoHeader(infoHeader)
	}

	return userInfo, nil
}

func parseSubscriptionUserinfoHeader(header string) *SubscriptionUserInfo {
	info := &SubscriptionUserInfo{}
	parts := strings.Split(header, ";")
	for _, part := range parts {
		kv := strings.Split(strings.TrimSpace(part), "=")
		if len(kv) == 2 {
			val, _ := strconv.ParseInt(kv[1], 10, 64)
			switch strings.ToLower(kv[0]) {
			case "upload":
				info.Upload = val
			case "download":
				info.Download = val
			case "total":
				info.Total = val
			case "expire":
				info.Expire = val
			}
		}
	}
	return info
}
