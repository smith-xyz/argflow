package hash

import (
	"crypto/sha256"
	"crypto/sha512"
	"hash"
)

// HashSHA256 hashes data with SHA-256
func HashSHA256(data []byte) []byte {
	h := sha256.New()
	h.Write(data)
	return h.Sum(nil)
}

// HashSHA512 hashes data with SHA-512
func HashSHA512(data []byte) []byte {
	h := sha512.New()
	h.Write(data)
	return h.Sum(nil)
}

// GetHasher returns a hasher based on algorithm name
func GetHasher(algorithm string) hash.Hash {
	switch algorithm {
	case "sha256":
		return sha256.New()
	case "sha512":
		return sha512.New()
	default:
		return sha256.New()
	}
}

