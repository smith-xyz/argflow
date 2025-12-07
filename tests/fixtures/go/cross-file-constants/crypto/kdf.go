package crypto

import (
	"crypto/sha256"

	"github.com/example/cross-file-constants/config"
	"golang.org/x/crypto/pbkdf2"
)

// DeriveKey uses constants from config package
func DeriveKey(password, salt []byte) []byte {
	return pbkdf2.Key(password, salt, config.PBKDF2Iterations, config.PBKDF2KeyLength, sha256.New)
}

// DeriveKeyWithDefaults uses derived constants
func DeriveKeyWithDefaults(password, salt []byte) []byte {
	return pbkdf2.Key(password, salt, config.DefaultIterations, config.SecureKeyLength, sha256.New)
}

// DeriveKeyExpression uses a constant expression
func DeriveKeyExpression(password, salt []byte) []byte {
	iterations := config.MinIterations + 5000
	return pbkdf2.Key(password, salt, iterations, 32, sha256.New)
}

