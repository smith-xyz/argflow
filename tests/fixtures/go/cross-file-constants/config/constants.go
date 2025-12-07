package config

// Crypto configuration constants
const (
	// PBKDF2 settings
	PBKDF2Iterations = 100000
	PBKDF2KeyLength  = 32

	// AES settings
	AESKeySize    = 32
	AESBlockSize  = 16

	// Minimum security requirements
	MinIterations = 10000
	MaxIterations = 1000000
)

// Derived constants
const (
	DefaultIterations = PBKDF2Iterations
	SecureKeyLength   = PBKDF2KeyLength * 2 // 64 bytes for extra security
)

