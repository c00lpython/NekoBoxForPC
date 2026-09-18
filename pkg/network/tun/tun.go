package tun

import (
	"fmt"
	"net"
	"runtime"
)

type TUNStatus struct {
	Active        bool   `json:"active"`
	InterfaceName string `json:"interface_name"`
	IPAddress     string `json:"ip_address"`
	MTU           int    `json:"mtu"`
}

type TUNManager struct {
	InterfaceName string
}

func NewTUNManager(name string) *TUNManager {
	if name == "" {
		name = "nekobox-tun"
	}
	return &TUNManager{
		InterfaceName: name,
	}
}

// CheckInterface checks if the TUN interface exists and is up.
func (m *TUNManager) CheckInterface() (*TUNStatus, error) {
	ifaces, err := net.Interfaces()
	if err != nil {
		return nil, fmt.Errorf("failed to list network interfaces: %w", err)
	}

	for _, iface := range ifaces {
		if iface.Name == m.InterfaceName {
			status := &TUNStatus{
				Active:        (iface.Flags & net.FlagUp) != 0,
				InterfaceName: iface.Name,
				MTU:           iface.MTU,
			}
			addrs, _ := iface.Addrs()
			if len(addrs) > 0 {
				status.IPAddress = addrs[0].String()
			}
			return status, nil
		}
	}

	return &TUNStatus{
		Active:        false,
		InterfaceName: m.InterfaceName,
	}, nil
}

// GetDefaultGateway returns the primary gateway interface info based on OS.
func (m *TUNManager) GetPlatform() string {
	return runtime.GOOS
}
