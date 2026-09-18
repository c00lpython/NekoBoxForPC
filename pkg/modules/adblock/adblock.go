package adblock

import (
	"bufio"
	"strings"
)

type RuleSet struct {
	Version int          `json:"version"`
	Rules   []RuleSetRule `json:"rules"`
}

type RuleSetRule struct {
	Domain       []string `json:"domain,omitempty"`
	DomainSuffix []string `json:"domain_suffix,omitempty"`
	DomainRegex  []string `json:"domain_regex,omitempty"`
}

// ConvertEasyListToRuleSet parses an adblock / EasyList text file into a sing-box RuleSet structure.
func ConvertEasyListToRuleSet(content string) *RuleSet {
	ruleSet := &RuleSet{
		Version: 1,
	}

	var domains []string
	var domainSuffixes []string

	scanner := bufio.NewScanner(strings.NewReader(content))
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "!") || strings.HasPrefix(line, "[") {
			continue
		}

		// EasyList domain rules: ||example.com^
		if strings.HasPrefix(line, "||") && strings.HasSuffix(line, "^") {
			cleanDomain := strings.TrimPrefix(strings.TrimSuffix(line, "^"), "||")
			domainSuffixes = append(domainSuffixes, cleanDomain)
		} else if !strings.Contains(line, "$") && !strings.Contains(line, "##") && !strings.Contains(line, "#@#") {
			if !strings.Contains(line, "/") && !strings.Contains(line, "*") {
				domains = append(domains, line)
			}
		}
	}

	var rules []RuleSetRule
	if len(domains) > 0 {
		rules = append(rules, RuleSetRule{Domain: domains})
	}
	if len(domainSuffixes) > 0 {
		rules = append(rules, RuleSetRule{DomainSuffix: domainSuffixes})
	}

	ruleSet.Rules = rules
	return ruleSet
}
