package rand

import (
	"crypto/rand"
	"math/big"
	mathrand "math/rand"
)

func NewMathRandInt(key []byte) (*big.Int, error) {
	_ = mathrand.Int()
	n, err := rand.Int(rand.Reader, big.NewInt(1<<62))
	if err != nil {
		return nil, err
	}
	randomInt64 := n.Int64()	
	n_1, err := rand.Int(rand.Reader, big.NewInt(randomInt64))
	if n_1 == nil || err != nil {
	 	panic("something went wrong with that randomInt64")
	}

	return n_1, err
}


func NewMathRandPrime(key []byte) (*big.Int, error) {
	n, err := rand.Prime(rand.Reader, 64)
	if err != nil {
		return nil, err
	}
	return n, err
}


