package libcore

import (
	"encoding/json"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strings"

	"github.com/matsuridayo/libneko/neko_common"
	"github.com/matsuridayo/libneko/neko_log"
)

// PCPlatformInterface provides a default implementation of BoxPlatformInterface for Desktop PC.
type PCPlatformInterface struct{}

func (p *PCPlatformInterface) AutoDetectInterfaceControl(fd int32) error {
	return nil
}

func (p *PCPlatformInterface) OpenTun(singTunOptionsJson, tunPlatformOptionsJson string) (int, error) {
	return -1, fmt.Errorf("native system tun is used on PC")
}

func (p *PCPlatformInterface) UseProcFS() bool {
	return false
}

func (p *PCPlatformInterface) FindConnectionOwner(ipProtocol int32, sourceAddress string, sourcePort int32, destinationAddress string, destinationPort int32) (int32, error) {
	return 0, fmt.Errorf("not supported on PC")
}

func (p *PCPlatformInterface) PackageNameByUid(uid int32) (string, error) {
	return "", nil
}

func (p *PCPlatformInterface) UIDByPackageName(packageName string) (int32, error) {
	return 0, nil
}

func (p *PCPlatformInterface) WIFIState() string {
	return ""
}

func (p *PCPlatformInterface) DefaultInterface() string {
	return ""
}

func (p *PCPlatformInterface) NetworkInterfaces() string {
	return ""
}

func (p *PCPlatformInterface) SendNotification(identifier, typeName, title, body, openURL string) error {
	fmt.Printf("[%s] %s: %s\n", typeName, title, body)
	return nil
}

// PCNB4AInterface provides a default implementation of NB4AInterface for Desktop PC.
type PCNB4AInterface struct{}

func (p *PCNB4AInterface) UseOfficialAssets() bool {
	return true
}

func (p *PCNB4AInterface) Selector_OnProxySelected(selectorTag string, tag string) {
	fmt.Printf("[Selector] %s selected: %s\n", selectorTag, tag)
}

func (p *PCNB4AInterface) MasterDnsVPNResolverProgress(found int32, total int32, ready bool) {
	fmt.Printf("[DNS] Progress: %d/%d (ready: %v)\n", found, total, ready)
}

func (p *PCNB4AInterface) MasterDnsVPNStartupFailed(noWorkingDNS bool, message string) {
	fmt.Printf("[DNS] Startup Failed: %s (noWorkingDNS=%v)\n", message, noWorkingDNS)
}

// InitPCCore initializes libcore for PC environment.
func InitPCCore(workDir string, logEnable bool) error {
	if workDir == "" {
		var err error
		workDir, err = os.Getwd()
		if err != nil {
			workDir = "."
		}
	}

	cacheDir := filepath.Join(workDir, "cache")
	assetsDir := filepath.Join(workDir, "assets")
	os.MkdirAll(cacheDir, 0755)
	os.MkdirAll(assetsDir, 0755)

	neko_common.RunMode = neko_common.RunMode_NekoBoxForAndroid

	workingPath = workDir
	tempPath = cacheDir
	externalAssetsPath = assetsDir
	internalAssetsPath = assetsDir

	neko_log.SetLogEnabled(logEnable)
	neko_log.SetupLog(10*1024*1024, filepath.Join(cacheDir, "nekobox.log"))
	PCLogStdout = logEnable

	intfNB4A = &PCNB4AInterface{}
	intfBox = &PCPlatformInterface{}

	return nil
}

// StartPCCore starts a sing-box instance on PC with given config content.
func StartPCCore(configContent string) (*BoxInstance, error) {
	return NewSingBoxInstance(configContent, nil)
}

func cleanHexOrString(val string) string {
	val = strings.TrimSpace(val)
	if strings.HasPrefix(val, "<b") || strings.HasPrefix(val, "<B") {
		val = strings.TrimPrefix(val, "<b")
		val = strings.TrimPrefix(val, "<B")
		val = strings.TrimSuffix(val, ">")
		val = strings.TrimSpace(val)
	}
	if strings.HasPrefix(val, "0x") || strings.HasPrefix(val, "0X") {
		val = val[2:]
	}
	return strings.TrimSpace(val)
}

// ConvertWGConfToSingBoxJSON converts a WireGuard / AmneziaWG .conf INI string into sing-box JSON configuration.
func ConvertWGConfToSingBoxJSON(confStr string, listenPort int) (string, error) {
	if listenPort <= 0 {
		listenPort = 20808
	}
	lines := strings.Split(confStr, "\n")
	var section string

	var privateKey, publicKey, endpoint string
	var addresses []string
	var mtu int = 1420
	var jc, jmin, jmax, s1, s2, s3, s4 int
	var h1, h2, h3, h4, i1, i2, i3, i4, i5 string
	var peerReserved []int

	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") || strings.HasPrefix(line, ";") {
			continue
		}
		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			section = strings.ToLower(line[1 : len(line)-1])
			continue
		}

		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			continue
		}
		key := strings.ToLower(strings.TrimSpace(parts[0]))
		val := strings.TrimSpace(parts[1])

		switch section {
		case "interface":
			switch key {
			case "privatekey":
				privateKey = val
			case "address":
				for _, addr := range strings.Split(val, ",") {
					addr = strings.TrimSpace(addr)
					if addr != "" {
						if !strings.Contains(addr, "/") {
							if strings.Contains(addr, ":") {
								addr += "/128"
							} else {
								addr += "/32"
							}
						}
						addresses = append(addresses, addr)
					}
				}
			case "mtu":
				fmt.Sscanf(val, "%d", &mtu)
			case "jc":
				fmt.Sscanf(val, "%d", &jc)
			case "jmin":
				fmt.Sscanf(val, "%d", &jmin)
			case "jmax":
				fmt.Sscanf(val, "%d", &jmax)
			case "s1":
				fmt.Sscanf(val, "%d", &s1)
			case "s2":
				fmt.Sscanf(val, "%d", &s2)
			case "s3":
				fmt.Sscanf(val, "%d", &s3)
			case "s4":
				fmt.Sscanf(val, "%d", &s4)
			case "h1":
				h1 = cleanHexOrString(val)
			case "h2":
				h2 = cleanHexOrString(val)
			case "h3":
				h3 = cleanHexOrString(val)
			case "h4":
				h4 = cleanHexOrString(val)
			case "i1":
				i1 = cleanHexOrString(val)
			case "i2":
				i2 = cleanHexOrString(val)
			case "i3":
				i3 = cleanHexOrString(val)
			case "i4":
				i4 = cleanHexOrString(val)
			case "i5":
				i5 = cleanHexOrString(val)
			}
		case "peer":
			switch key {
			case "publickey":
				publicKey = val
			case "endpoint":
				endpoint = val
			case "reserved":
				cleaned := cleanHexOrString(val)
				if len(cleaned) > 0 && len(cleaned)%2 == 0 {
					var resInts []int
					for i := 0; i < len(cleaned); i += 2 {
						var b int
						fmt.Sscanf(cleaned[i:i+2], "%02x", &b)
						resInts = append(resInts, b)
					}
					if len(resInts) > 0 {
						peerReserved = resInts
					}
				} else {
					var resInts []int
					for _, item := range strings.Split(val, ",") {
						item = strings.TrimSpace(item)
						var b int
						if _, err := fmt.Sscanf(item, "%d", &b); err == nil {
							resInts = append(resInts, b)
						}
					}
					if len(resInts) > 0 {
						peerReserved = resInts
					}
				}
			}
		}
	}

	host, portStr, err := net.SplitHostPort(endpoint)
	if err != nil {
		host = endpoint
		portStr = "51820"
	}
	var port int
	fmt.Sscanf(portStr, "%d", &port)

	peerObj := map[string]interface{}{
		"address":     host,
		"port":        port,
		"public_key":  publicKey,
		"allowed_ips": []string{"0.0.0.0/0", "::/0"},
	}
	if len(peerReserved) > 0 {
		peerObj["reserved"] = peerReserved
	}

	endpointType := "wireguard"
	if jc > 0 || jmin > 0 || jmax > 0 || s1 > 0 || s2 > 0 || h1 != "" || h2 != "" || i1 != "" {
		endpointType = "awg"
	}

	endpointMap := map[string]interface{}{
		"type":        endpointType,
		"tag":         "proxy",
		"address":     addresses,
		"private_key": privateKey,
		"mtu":         mtu,
		"peers":       []map[string]interface{}{peerObj},
	}

	if jc > 0 {
		endpointMap["jc"] = jc
	}
	if jmin > 0 {
		endpointMap["jmin"] = jmin
	}
	if jmax > 0 {
		endpointMap["jmax"] = jmax
	}
	if s1 > 0 {
		endpointMap["s1"] = s1
	}
	if s2 > 0 {
		endpointMap["s2"] = s2
	}
	if s3 > 0 {
		endpointMap["s3"] = s3
	}
	if s4 > 0 {
		endpointMap["s4"] = s4
	}
	if h1 != "" {
		endpointMap["h1"] = h1
	}
	if h2 != "" {
		endpointMap["h2"] = h2
	}
	if h3 != "" {
		endpointMap["h3"] = h3
	}
	if h4 != "" {
		endpointMap["h4"] = h4
	}
	if i1 != "" {
		endpointMap["i1"] = i1
	}
	if i2 != "" {
		endpointMap["i2"] = i2
	}
	if i3 != "" {
		endpointMap["i3"] = i3
	}
	if i4 != "" {
		endpointMap["i4"] = i4
	}
	if i5 != "" {
		endpointMap["i5"] = i5
	}

	configObj := map[string]interface{}{
		"log": map[string]interface{}{
			"level": "info",
		},
		"route": map[string]interface{}{
			"final": "proxy",
		},
		"inbounds": []map[string]interface{}{
			{
				"type":        "mixed",
				"tag":         "mixed-in",
				"listen":      "127.0.0.1",
				"listen_port": listenPort,
			},
		},
		"outbounds": []map[string]interface{}{
			{
				"type": "direct",
				"tag":  "direct",
			},
		},
		"endpoints": []map[string]interface{}{
			endpointMap,
		},
	}

	resBytes, err := json.MarshalIndent(configObj, "", "  ")
	if err != nil {
		return "", err
	}
	return string(resBytes), nil
}
