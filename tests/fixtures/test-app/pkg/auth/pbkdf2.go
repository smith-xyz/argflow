package auth

import (
	"crypto/pbkdf2"
	"crypto/sha256"
)

func DeriveKey(password string, salt []byte) ([]byte, error) {
	iterations := 10000
	keyLen := 32
	hashFunc := sha256.New
	return pbkdf2.Key(hashFunc, password, salt, iterations, keyLen)
}

