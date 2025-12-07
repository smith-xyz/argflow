package utils

import (
	"fmt"
	"strings"
)

// FormatKey formats a key as hex string
func FormatKey(key []byte) string {
	return fmt.Sprintf("%x", key)
}

// ParseKeySize parses key size from string
func ParseKeySize(size string) int {
	switch strings.ToLower(size) {
	case "128":
		return 16
	case "256":
		return 32
	default:
		return 32
	}
}

// ValidatePassword checks if password meets requirements
func ValidatePassword(password string) bool {
	return len(password) >= 8
}

