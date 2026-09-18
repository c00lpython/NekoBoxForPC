//go:build windows

package libcore

func dupFd(fd int) (int, error) {
	return fd, nil
}
