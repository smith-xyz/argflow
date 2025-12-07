package kdf

import (
	"crypto/sha256"

	"golang.org/x/crypto/pbkdf2"
)

const (
	DefaultIterations = 100000
	DefaultKeyLength  = 32
)

// DeriveKey derives a key using PBKDF2 with default parameters
func DeriveKey(password, salt []byte) []byte {
	return pbkdf2.Key(password, salt, DefaultIterations, DefaultKeyLength, sha256.New)
}

// DeriveKeyCustom derives a key with custom iterations
func DeriveKeyCustom(password, salt []byte, iterations int) []byte {
	return pbkdf2.Key(password, salt, iterations, 32, sha256.New)
}

// DeriveKeyLiteral uses literal values directly
func DeriveKeyLiteral(password, salt []byte) []byte {
	return pbkdf2.Key(password, salt, 10000, 32, sha256.New)
}

