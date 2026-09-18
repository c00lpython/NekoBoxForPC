package byedpi

import (
	"os/exec"
	"strconv"
	"strings"

	"nekobox-for-pc/pkg/models"
)

type ByeDPIEngine struct {
	cmd *exec.Cmd
}

func NewByeDPIEngine() *ByeDPIEngine {
	return &ByeDPIEngine{}
}

// BuildArgs converts models.ByeDPIOptions into command line arguments for the ByeDPI daemon.
func (e *ByeDPIEngine) BuildArgs(opts *models.ByeDPIOptions, listenPort int) []string {
	var args []string

	if listenPort <= 0 {
		listenPort = 1080
	}
	args = append(args, "-p", strconv.Itoa(listenPort))

	if opts == nil {
		return args
	}

	if opts.SplitPosition > 0 {
		args = append(args, "--split", strconv.Itoa(opts.SplitPosition))
	}
	if opts.SplitMarker != "" {
		args = append(args, "--split-marker", opts.SplitMarker)
	}
	if opts.Disoob {
		args = append(args, "--disoob")
	}
	if opts.Auto {
		args = append(args, "--auto")
	}
	if opts.Fake > 0 {
		args = append(args, "--fake", strconv.Itoa(opts.Fake))
	}
	if opts.TTL > 0 {
		args = append(args, "--ttl", strconv.Itoa(opts.TTL))
	}
	if opts.SNI != "" {
		args = append(args, "--sni", opts.SNI)
	}

	// Custom user arguments
	if opts.CustomArgs != "" {
		fields := strings.Fields(opts.CustomArgs)
		args = append(args, fields...)
	}

	return args
}

// Start launches the ByeDPI subprocess.
func (e *ByeDPIEngine) Start(binPath string, args []string) error {
	if binPath == "" {
		binPath = "ciadpi"
	}
	e.cmd = exec.Command(binPath, args...)
	return e.cmd.Start()
}

// Stop terminates the running ByeDPI process.
func (e *ByeDPIEngine) Stop() error {
	if e.cmd != nil && e.cmd.Process != nil {
		err := e.cmd.Process.Kill()
		e.cmd = nil
		return err
	}
	return nil
}

func (e *ByeDPIEngine) IsRunning() bool {
	return e.cmd != nil && e.cmd.Process != nil
}
