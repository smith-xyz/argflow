package cipher

import (
	"crypto/aes"
	"crypto/cipher"
)

const KeySize128 = 16
const KeySize256 = 32

// NewAES128 creates an AES-128 cipher
func NewAES128(key []byte) (cipher.Block, error) {
	if len(key) < KeySize128 {
		return nil, nil
	}
	return aes.NewCipher(key[:KeySize128])
}

// NewAES256 creates an AES-256 cipher with literal key size
func NewAES256(key []byte) (cipher.Block, error) {
	if len(key) < 32 {
		return nil, nil
	}
	return aes.NewCipher(key[:32])
}

// CreateGCM creates a GCM cipher from a block
func CreateGCM(block cipher.Block) (cipher.AEAD, error) {
	return cipher.NewGCM(block)
}

