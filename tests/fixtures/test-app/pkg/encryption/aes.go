package encryption

import "crypto/aes"

func EncryptAES128(key []byte, plaintext []byte) ([]byte, error) {
	keySize := 16
	if len(key) < keySize {
		return nil, nil
	}
	cipher, err := aes.NewCipher(key[:keySize])
	if err != nil {
		return nil, err
	}
	ciphertext := make([]byte, len(plaintext))
	cipher.Encrypt(ciphertext, plaintext)
	return ciphertext, nil
}

