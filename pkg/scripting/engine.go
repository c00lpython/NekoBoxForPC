package scripting

import (
	"fmt"
	"regexp"
	"strings"

	"nekobox-for-pc/pkg/models"
)

type FilterMode string

const (
	FilterDisabled FilterMode = "disabled"
	FilterInclude  FilterMode = "include"
	FilterExclude  FilterMode = "exclude"
)

// ScriptEngine executes filters, tag rewriting, deduplication and transformations on proxies.
type ScriptEngine struct{}

func NewScriptEngine() *ScriptEngine {
	return &ScriptEngine{}
}

// FilterByRegex filters proxies using an include or exclude regex pattern.
func (e *ScriptEngine) FilterByRegex(proxies []*models.ProxyEntity, pattern string, mode FilterMode) ([]*models.ProxyEntity, error) {
	if mode == FilterDisabled || strings.TrimSpace(pattern) == "" {
		return proxies, nil
	}

	re, err := regexp.Compile(pattern)
	if err != nil {
		return nil, fmt.Errorf("invalid filter regex: %w", err)
	}

	var filtered []*models.ProxyEntity
	for _, p := range proxies {
		matched := re.MatchString(p.Tag) || re.MatchString(p.Server)
		if mode == FilterInclude && matched {
			filtered = append(filtered, p)
		} else if mode == FilterExclude && !matched {
			filtered = append(filtered, p)
		}
	}

	return filtered, nil
}

// Deduplicate removes duplicate proxies based on Server + ServerPort + Type.
func (e *ScriptEngine) Deduplicate(proxies []*models.ProxyEntity) []*models.ProxyEntity {
	seen := make(map[string]bool)
	var unique []*models.ProxyEntity

	for _, p := range proxies {
		key := fmt.Sprintf("%d:%s:%d", p.Type, p.Server, p.ServerPort)
		if !seen[key] {
			seen[key] = true
			unique = append(unique, p)
		}
	}
	return unique
}

// RenameTags applies regex substitution or prefix/suffix to proxy tags (Happ / v2rayNG style).
func (e *ScriptEngine) RenameTags(proxies []*models.ProxyEntity, matchPattern string, replacePattern string) []*models.ProxyEntity {
	if matchPattern == "" {
		return proxies
	}
	re, err := regexp.Compile(matchPattern)
	if err != nil {
		return proxies
	}

	for _, p := range proxies {
		p.Tag = re.ReplaceAllString(p.Tag, replacePattern)
	}
	return proxies
}

// EnsureUniqueTags ensures no two proxies have the exact same Tag.
func (e *ScriptEngine) EnsureUniqueTags(proxies []*models.ProxyEntity) []*models.ProxyEntity {
	counts := make(map[string]int)
	for _, p := range proxies {
		baseTag := p.Tag
		if counts[baseTag] > 0 {
			p.Tag = fmt.Sprintf("%s (%d)", baseTag, counts[baseTag]+1)
		}
		counts[baseTag]++
	}
	return proxies
}
