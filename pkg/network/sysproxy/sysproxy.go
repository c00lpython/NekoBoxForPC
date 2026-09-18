package sysproxy

import (
	"fmt"
	"os/exec"
	"runtime"
)

type SystemProxyManager struct{}

func NewSystemProxyManager() *SystemProxyManager {
	return &SystemProxyManager{}
}

// SetSystemProxy configures the OS HTTP/HTTPS proxy to 127.0.0.1:<port>.
func (m *SystemProxyManager) SetSystemProxy(port int) error {
	server := fmt.Sprintf("127.0.0.1:%d", port)

	switch runtime.GOOS {
	case "windows":
		// Set Windows Internet Settings in Registry
		regKey := `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings`
		if err := exec.Command("reg", "add", regKey, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "1", "/f").Run(); err != nil {
			return fmt.Errorf("failed to enable proxy in registry: %w", err)
		}
		if err := exec.Command("reg", "add", regKey, "/v", "ProxyServer", "/t", "REG_SZ", "/d", server, "/f").Run(); err != nil {
			return fmt.Errorf("failed to set proxy server in registry: %w", err)
		}
		// Bypass local addresses
		_ = exec.Command("reg", "add", regKey, "/v", "ProxyOverride", "/t", "REG_SZ", "/d", "<local>;localhost;127.*;10.*;192.168.*", "/f").Run()

	case "darwin":
		// macOS networksetup
		_ = exec.Command("networksetup", "-setwebproxy", "Wi-Fi", "127.0.0.1", fmt.Sprintf("%d", port)).Run()
		_ = exec.Command("networksetup", "-setsecurewebproxy", "Wi-Fi", "127.0.0.1", fmt.Sprintf("%d", port)).Run()

	case "linux":
		// GNOME desktop proxy settings
		_ = exec.Command("gsettings", "set", "org.gnome.system.proxy", "mode", "manual").Run()
		_ = exec.Command("gsettings", "set", "org.gnome.system.proxy.http", "host", "127.0.0.1").Run()
		_ = exec.Command("gsettings", "set", "org.gnome.system.proxy.http", "port", fmt.Sprintf("%d", port)).Run()
	}

	return nil
}

// ClearSystemProxy disables the OS HTTP/HTTPS proxy.
func (m *SystemProxyManager) ClearSystemProxy() error {
	switch runtime.GOOS {
	case "windows":
		regKey := `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings`
		if err := exec.Command("reg", "add", regKey, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f").Run(); err != nil {
			return fmt.Errorf("failed to disable proxy in registry: %w", err)
		}

	case "darwin":
		_ = exec.Command("networksetup", "-setwebproxystate", "Wi-Fi", "off").Run()
		_ = exec.Command("networksetup", "-setsecurewebproxystate", "Wi-Fi", "off").Run()

	case "linux":
		_ = exec.Command("gsettings", "set", "org.gnome.system.proxy", "mode", "none").Run()
	}

	return nil
}
