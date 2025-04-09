package utils

import (
	"sort"
	"unsafe"
)

// SortMapKeys map keys to string array and sort
func SortMapKeys(m map[string]string) []string {
	keys := make([]string, 0)
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

// String tips: 只有在原有bytes确保不会发生变化时可以使用
func String(data []byte) string {
	return *(*string)(unsafe.Pointer(&data))
}

// Bytes 利用反射转移字符串数据到bytes
func Bytes(src string) (data []byte) {
	return unsafe.Slice(unsafe.StringData(src), len(src))
}
