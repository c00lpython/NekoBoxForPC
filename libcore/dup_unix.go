//go:build unix

package libcore

import "syscall"

func dupFd(fd int) (int, error) {
	return syscall.Dup(fd)
}
