package modules

import (
	"strings"
	"testing"

	"nekobox-for-pc/pkg/models"
	"nekobox-for-pc/pkg/modules/adblock"
	"nekobox-for-pc/pkg/modules/amneziawg"
	"nekobox-for-pc/pkg/modules/byedpi"
)

func TestByeDPIArgs(t *testing.T) {
	engine := byedpi.NewByeDPIEngine()
	opts := &models.ByeDPIOptions{
		SplitPosition: 2,
		Disoob:        true,
		Auto:          true,
		TTL:           4,
		SNI:           "fake.domain.com",
	}

	args := engine.BuildArgs(opts, 1080)
	joined := strings.Join(args, " ")

	if !strings.Contains(joined, "--split 2") || !strings.Contains(joined, "--disoob") || !strings.Contains(joined, "--ttl 4") {
		t.Errorf("expected generated arguments, got %s", joined)
	}
}

func TestAmneziaWGValidation(t *testing.T) {
	opts := &models.AmneziaWGOptions{
		WireGuardOptions: models.WireGuardOptions{
			PrivateKey: "pk123",
			PeerPublic: "pub123",
		},
		Jmin: 100,
		Jmax: 50, // Invalid: Jmin > Jmax
	}

	if err := amneziawg.ValidateAmneziaWGOptions(opts); err == nil {
		t.Errorf("expected validation error when Jmin > Jmax")
	}
}

func TestAdblockEasyListConvert(t *testing.T) {
	sample := `
! Title: EasyList
||ads.example.com^
||tracker.doubleclick.net^
badtracker.com
! Comment line
[Adblock Plus 2.0]
`
	ruleSet := adblock.ConvertEasyListToRuleSet(sample)
	if ruleSet == nil || len(ruleSet.Rules) == 0 {
		t.Fatalf("expected rule set to be generated")
	}

	foundSuffix := false
	for _, r := range ruleSet.Rules {
		if len(r.DomainSuffix) >= 2 {
			foundSuffix = true
		}
	}
	if !foundSuffix {
		t.Errorf("expected domain suffixes parsed from EasyList, got %+v", ruleSet)
	}
}
