package parsers

import (
	"testing"

	"nekobox-for-pc/pkg/models"
)

func TestParseVLESS(t *testing.T) {
	link := "vless://b831381d-6324-4d53-ad4f-8cda48b30811@127.0.0.1:443?type=ws&security=tls&path=%2Fws&sni=example.com#TestVLESS"
	entity, err := ParseUniversal(link)
	if err != nil {
		t.Fatalf("unexpected error parsing vless: %v", err)
	}

	if entity.Type != models.TypeVLESS {
		t.Errorf("expected type VLESS, got %v", entity.Type)
	}
	if entity.Server != "127.0.0.1" || entity.ServerPort != 443 {
		t.Errorf("expected server 127.0.0.1:443, got %s:%d", entity.Server, entity.ServerPort)
	}
	if entity.UUID != "b831381d-6324-4d53-ad4f-8cda48b30811" {
		t.Errorf("expected uuid match, got %s", entity.UUID)
	}
	if entity.Security != "tls" || entity.SNI != "example.com" {
		t.Errorf("expected tls + example.com, got %s + %s", entity.Security, entity.SNI)
	}
	if entity.Tag != "TestVLESS" {
		t.Errorf("expected tag TestVLESS, got %s", entity.Tag)
	}
}

func TestParseShadowsocks(t *testing.T) {
	// ss://aes-256-gcm:password123@192.168.1.1:8388#MyServer
	link := "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQxMjM=@192.168.1.1:8388#MyServer"
	entity, err := ParseUniversal(link)
	if err != nil {
		t.Fatalf("unexpected error parsing ss: %v", err)
	}

	if entity.Type != models.TypeSS {
		t.Errorf("expected type SS, got %v", entity.Type)
	}
	if entity.Server != "192.168.1.1" || entity.ServerPort != 8388 {
		t.Errorf("expected server 192.168.1.1:8388, got %s:%d", entity.Server, entity.ServerPort)
	}
	if entity.Method != "aes-256-gcm" || entity.Password != "password123" {
		t.Errorf("expected aes-256-gcm:password123, got %s:%s", entity.Method, entity.Password)
	}
	if entity.Tag != "MyServer" {
		t.Errorf("expected tag MyServer, got %s", entity.Tag)
	}
}

func TestParseHysteria2(t *testing.T) {
	link := "hy2://secret123@hy2.example.com:8443?sni=hy2.example.com&insecure=1&up=50&down=200#Hy2Node"
	entity, err := ParseUniversal(link)
	if err != nil {
		t.Fatalf("unexpected error parsing hy2: %v", err)
	}

	if entity.Type != models.TypeHysteria {
		t.Errorf("expected type Hysteria, got %v", entity.Type)
	}
	if entity.Password != "secret123" {
		t.Errorf("expected password secret123, got %s", entity.Password)
	}
	if entity.Hysteria2 == nil || entity.Hysteria2.UpMbps != 50 || entity.Hysteria2.DownMbps != 200 {
		t.Errorf("expected 50/200 Mbps, got %+v", entity.Hysteria2)
	}
}

func TestParseAmneziaWG(t *testing.T) {
	link := "awg://awg.example.com:51820?pk=testPrivateKey&pub=testPeerPublic&jc=4&jmin=50&jmax=100&s1=15&s2=20&h1=1&h2=2&h3=3&h4=4#AWGNode"
	entity, err := ParseUniversal(link)
	if err != nil {
		t.Fatalf("unexpected error parsing awg: %v", err)
	}

	if entity.Type != models.TypeAmneziaWG {
		t.Errorf("expected type AmneziaWG, got %v", entity.Type)
	}
	if entity.AmneziaWG == nil || entity.AmneziaWG.Jc != 4 || entity.AmneziaWG.H1 != "1" {
		t.Errorf("expected AWG headers parsed, got %+v", entity.AmneziaWG)
	}
}

func TestParseClashYAML(t *testing.T) {
	yamlContent := `
proxies:
  - name: "Clash-VLESS"
    type: vless
    server: vless.clash.com
    port: 443
    uuid: 12345678-1234-1234-1234-123456789012
    tls: true
    servername: vless.clash.com
  - name: "Clash-SS"
    type: ss
    server: ss.clash.com
    port: 8388
    cipher: aes-128-gcm
    password: testpass
`
	proxies, err := ParseClash(yamlContent)
	if err != nil {
		t.Fatalf("unexpected error parsing clash yaml: %v", err)
	}
	if len(proxies) != 2 {
		t.Fatalf("expected 2 proxies, got %d", len(proxies))
	}
	if proxies[0].Type != models.TypeVLESS || proxies[1].Type != models.TypeSS {
		t.Errorf("unexpected proxy types parsed from clash yaml")
	}
}

func TestParseAmneziaWGConf(t *testing.T) {
	confContent := `
[Interface]
Address = 172.16.0.2/32, fd00::2/128
PrivateKey = aGVsbG93b3JsZFByaXZhdGVLZXkxMjM0NTY3ODkwMTI=
DNS = 1.1.1.1, 8.8.8.8
MTU = 1360
Jc = 5
Jmin = 40
Jmax = 80
S1 = 15
S2 = 25
H1 = 12345678
H2 = 87654321
H3 = 11223344
H4 = 44332211

[Peer]
PublicKey = cGVlclB1YmxpY0tleTEyMzQ1Njc4OTAxMjM0NTY3ODkwMTI=
Endpoint = 162.159.192.1:2408
AllowedIPs = 0.0.0.0/0, ::/0
`
	proxies, err := ParseWireGuardConf(confContent, "WARP-Amnezia")
	if err != nil {
		t.Fatalf("unexpected error parsing amneziawg .conf: %v", err)
	}

	if len(proxies) != 1 {
		t.Fatalf("expected 1 proxy, got %d", len(proxies))
	}

	p := proxies[0]
	if p.Type != models.TypeAmneziaWG {
		t.Errorf("expected type AmneziaWG, got %v", p.Type)
	}
	if p.Server != "162.159.192.1" || p.ServerPort != 2408 {
		t.Errorf("expected 162.159.192.1:2408, got %s:%d", p.Server, p.ServerPort)
	}
	if p.AmneziaWG == nil || p.AmneziaWG.Jc != 5 || p.AmneziaWG.H1 != "12345678" {
		t.Errorf("expected AmneziaWG Jc=5 and H1=12345678, got %+v", p.AmneziaWG)
	}
}

