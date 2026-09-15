package main

import (
	"flag"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"syscall"
	"time"

	"libcore"
)

func main() {
	configPath := flag.String("c", "config.json", "Path to configuration file (.json or .conf)")
	workDir := flag.String("w", "", "Working directory (default: current directory)")
	port := flag.Int("p", 20808, "Local mixed SOCKS5/HTTP proxy port for .conf configs")
	pingFlag := flag.Bool("ping", false, "Test proxy latency after starting")
	versionFlag := flag.Bool("v", false, "Print version information")
	logEnable := flag.Bool("log", true, "Enable log file recording")

	flag.Parse()

	if *versionFlag {
		fmt.Println("=== sing-box (libcore edition) ===")
		fmt.Println(libcore.VersionBox())
		fmt.Println("\n=== Modules ===")
		fmt.Println(libcore.VersionModules())
		os.Exit(0)
	}

	fmt.Println("Initializing sing-box core engine...")

	absWorkDir := *workDir
	if absWorkDir == "" {
		var err error
		absWorkDir, err = os.Getwd()
		if err != nil {
			absWorkDir = "."
		}
	}
	absWorkDir, _ = filepath.Abs(absWorkDir)

	err := libcore.InitPCCore(absWorkDir, *logEnable)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize PC core: %v\n", err)
		os.Exit(1)
	}

	absConfigPath := *configPath
	if !filepath.IsAbs(absConfigPath) {
		absConfigPath = filepath.Join(absWorkDir, absConfigPath)
	}

	configBytes, err := os.ReadFile(absConfigPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to read config file '%s': %v\n", absConfigPath, err)
		os.Exit(1)
	}

	configStr := string(configBytes)
	trimmedStr := strings.TrimSpace(configStr)
	listenPort := *port
	if strings.HasSuffix(strings.ToLower(absConfigPath), ".conf") || strings.HasPrefix(trimmedStr, "[") {
		fmt.Println("Detected WireGuard / AmneziaWG .conf file, converting to sing-box JSON...")
		convertedJSON, err := libcore.ConvertWGConfToSingBoxJSON(configStr, listenPort)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to parse WG/AWG .conf file: %v\n", err)
			os.Exit(1)
		}
		configStr = convertedJSON
	}

	fmt.Printf("Starting core instance with config: %s\n", absConfigPath)

	boxInstance, err := libcore.StartPCCore(configStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to start sing-box instance: %v\n", err)
		os.Exit(1)
	}

	err = boxInstance.Start()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to start sing-box instance: %v\n", err)
		os.Exit(1)
	}

	fmt.Println("\n==================================================")
	fmt.Println(" sing-box (libcore edition) Running Successfully! ")
	fmt.Printf("  Proxy Address: 127.0.0.1:%d\n", listenPort)
	fmt.Printf("  HTTP Proxy:    http://127.0.0.1:%d\n", listenPort)
	fmt.Printf("  SOCKS5 Proxy:  socks5://127.0.0.1:%d\n", listenPort)
	fmt.Println("==================================================")
	fmt.Println("Press Ctrl+C to stop.\n")

	if *pingFlag {
		go func() {
			time.Sleep(500 * time.Millisecond)
			fmt.Printf("[Ping Test] Testing connectivity via 127.0.0.1:%d...\n", listenPort)
			proxyURL, err := url.Parse(fmt.Sprintf("http://127.0.0.1:%d", listenPort))
			if err != nil {
				fmt.Printf("[Ping Test] Error parsing proxy URL: %v\n", err)
				return
			}
			client := &http.Client{
				Transport: &http.Transport{
					Proxy: http.ProxyURL(proxyURL),
				},
				Timeout: 7 * time.Second,
			}
			start := time.Now()
			resp, err := client.Get("https://www.gstatic.com/generate_204")
			if err != nil {
				fmt.Printf("[Ping Test] Failed: %v\n", err)
			} else {
				resp.Body.Close()
				elapsed := time.Since(start).Milliseconds()
				fmt.Printf("[Ping Test] Success! Latency: %d ms (Status: %s)\n", elapsed, resp.Status)
			}
		}()
	}

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	<-sigChan

	fmt.Println("\nStopping NekoBoxForPC core...")
	_ = boxInstance.Close()
	fmt.Println("Core stopped cleanly.")
}
