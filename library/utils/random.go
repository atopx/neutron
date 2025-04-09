package utils

import (
	"math/rand"
	"time"
)

const (
	indexbits = 6
	indexmask = 1<<indexbits - 1
	indexmax  = 63 / indexbits
	lowercase = "abcdefghijklmnopqrstuvwxyz"
	letternum = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
)

var random = rand.New(rand.NewSource(time.Now().UnixNano()))

// RandomStr 随机字符串
func RandomStr(n int, choices string) string {
	b := make([]byte, n)

	for i, cache, remain := n-1, random.Int63(), indexmax; i >= 0; {
		if remain == 0 {
			cache, remain = random.Int63(), indexmax
		}
		if idx := int(cache & indexmask); idx < len(choices) {
			b[i] = choices[idx]
			i--
		}
		cache >>= indexbits
		remain--
	}
	return string(b)
}

// RandLowwerCase 随机小写字母
func RandLowwerCase(n int) string {
	return RandomStr(n, lowercase)
}

// RandLetterNumbers 随机大小写字母和数字
func RandLetterNumbers(n int) string {
	return RandomStr(n, letternum)
}
