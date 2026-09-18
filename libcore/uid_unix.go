//go:build unix

package libcore

import "os"

func getUidGid() (int, int) {
	return os.Getuid(), os.Getgid()
}
