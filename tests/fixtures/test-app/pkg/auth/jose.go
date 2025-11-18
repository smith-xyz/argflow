package auth

import (
	"crypto/rand"

	"github.com/go-jose/go-jose/v3"
)

func EncryptWithJOSE(plaintext []byte, key []byte) (string, error) {
	encrypter, err := jose.NewEncrypter(
		jose.A128GCM,
		jose.Recipient{Algorithm: jose.DIRECT, Key: key},
		nil,
	)
	if err != nil {
		return "", err
	}

	object, err := encrypter.Encrypt(plaintext)
	if err != nil {
		return "", err
	}

	return object.CompactSerialize()
}

func GenerateJOSEKey() ([]byte, error) {
	key := make([]byte, 16)
	_, err := rand.Read(key)
	return key, err
}

