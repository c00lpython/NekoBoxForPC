package amneziawg

import (
	"fmt"
	"nekobox-for-pc/pkg/models"
)

// ValidateAmneziaWGOptions verifies that obfuscation headers and params conform to AWG specs.
func ValidateAmneziaWGOptions(opts *models.AmneziaWGOptions) error {
	if opts == nil {
		return fmt.Errorf("amneziawg options cannot be nil")
	}
	if opts.PrivateKey == "" {
		return fmt.Errorf("private key is required")
	}
	if opts.PeerPublic == "" {
		return fmt.Errorf("peer public key is required")
	}
	if opts.Jmin > opts.Jmax && opts.Jmax > 0 {
		return fmt.Errorf("jmin (%d) cannot be greater than jmax (%d)", opts.Jmin, opts.Jmax)
	}
	return nil
}
