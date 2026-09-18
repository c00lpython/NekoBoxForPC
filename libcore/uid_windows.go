//go:build windows

package libcore

import "os"

func getUidGid() (int, int) {
	return os.Getuid(), os.Getgid()
}

