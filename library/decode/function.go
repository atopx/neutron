package decode

import (
	"bytes"
	"crypto/md5"
	"encoding/base64"
	"fmt"
	"math/rand"
	"net/url"
	"regexp"
	"strings"
	"time"

	"github.com/atopx/neutron/library/proto"
	"github.com/atopx/neutron/library/utils"
	"github.com/google/cel-go/common/types"
	"github.com/google/cel-go/common/types/ref"
	"github.com/google/cel-go/interpreter/functions"
)

var containsStringFunc = &functions.Overload{
	Operator: "contains_string",
	Binary: func(lhs ref.Val, rhs ref.Val) ref.Val {
		v1, ok := lhs.(types.String)
		if !ok {
			return types.ValOrErr(lhs, "unexpected type '%v' passed to contains", lhs.Type())
		}
		v2, ok := rhs.(types.String)
		if !ok {
			return types.ValOrErr(rhs, "unexpected type '%v' passed to contains", rhs.Type())
		}
		return types.Bool(strings.Contains(string(v1), string(v2)))
	},
}

// 使用字符串忽略大小写比较
var stringIContainsStringFunc = &functions.Overload{
	Operator: "string_icontains_string",
	Binary: func(lhs ref.Val, rhs ref.Val) ref.Val {
		v1, ok := lhs.(types.String)
		if !ok {
			return types.ValOrErr(lhs, "unexpected type '%v' passed to icontains", lhs.Type())
		}
		v2, ok := rhs.(types.String)
		if !ok {
			return types.ValOrErr(rhs, "unexpected type '%v' passed to icontains", rhs.Type())
		}
		return types.Bool(strings.Contains(strings.ToLower(string(v1)), strings.ToLower(string(v2))))
	},
}

var bytesBContainsBytesFunc = &functions.Overload{
	Operator: "bytes_bcontains_bytes",
	Binary: func(lhs ref.Val, rhs ref.Val) ref.Val {
		v1, ok := lhs.(types.Bytes)
		if !ok {
			return types.ValOrErr(lhs, "unexpected type '%v' passed to bcontains", lhs.Type())
		}
		v2, ok := rhs.(types.Bytes)
		if !ok {
			return types.ValOrErr(rhs, "unexpected type '%v' passed to bcontains", rhs.Type())
		}
		return types.Bool(bytes.Contains(v1, v2))
	},
}

var matchesStringFunc = &functions.Overload{
	Operator: "matches_string",
	Binary: func(lhs ref.Val, rhs ref.Val) ref.Val {
		v1, ok := lhs.(types.String)
		if !ok {
			return types.ValOrErr(lhs, "unexpected type '%v' passed to match", lhs.Type())
		}
		v2, ok := rhs.(types.String)
		if !ok {
			return types.ValOrErr(rhs, "unexpected type '%v' passed to match", rhs.Type())
		}
		ok, err := regexp.Match(string(v1), []byte(v2))
		if err != nil {
			return types.NewErr("%v", err)
		}
		return types.Bool(ok)
	},
}

var stringBmatchBytesFunc = &functions.Overload{
	Operator: "string_bmatch_bytes",
	Binary: func(lhs ref.Val, rhs ref.Val) ref.Val {
		v1, ok := lhs.(types.String)
		if !ok {
			return types.ValOrErr(lhs, "unexpected type '%v' passed to bmatch", lhs.Type())
		}
		v2, ok := rhs.(types.Bytes)
		if !ok {
			return types.ValOrErr(rhs, "unexpected type '%v' passed to bmatch", rhs.Type())
		}
		ok, err := regexp.Match(string(v1), v2)
		if err != nil {
			return types.NewErr("%v", err)
		}
		return types.Bool(ok)
	},
}

var md5StringFunc = &functions.Overload{
	Operator: "md5_string",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.String)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to md5_string", value.Type())
		}
		return types.String(fmt.Sprintf("%x", md5.Sum([]byte(v))))
	},
}

var randomIntFunc = &functions.Overload{
	Operator: "randomInt_int_int",
	Binary: func(lhs ref.Val, rhs ref.Val) ref.Val {
		from, ok := lhs.(types.Int)
		if !ok {
			return types.ValOrErr(lhs, "unexpected type '%v' passed to randomInt", lhs.Type())
		}
		to, ok := rhs.(types.Int)
		if !ok {
			return types.ValOrErr(rhs, "unexpected type '%v' passed to randomInt", rhs.Type())
		}
		min, max := int(from), int(to)
		return types.Int(rand.Intn(max-min) + min)
	},
}

// 指定长度的小写字母组成的随机字符串
var randomLowercaseFunc = &functions.Overload{
	Operator: "randomLowercase_int",
	Unary: func(value ref.Val) ref.Val {
		n, ok := value.(types.Int)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to randomLowercase", value.Type())
		}
		return types.String(utils.RandLowwerCase(int(n)))
	},
}

// 将字符串进行 base64 编码
var base64StringFunc = &functions.Overload{
	Operator: "base64_string",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.String)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to base64_string", value.Type())
		}
		return types.String(base64.StdEncoding.EncodeToString([]byte(v)))
	},
}

// 将bytes进行 base64 编码
var base64BytesFunc = &functions.Overload{
	Operator: "base64_bytes",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.Bytes)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to base64_bytes", value.Type())
		}
		return types.String(base64.StdEncoding.EncodeToString(v))
	},
}

// 将字符串进行 base64 解码
var base64DecodeStringFunc = &functions.Overload{
	Operator: "base64Decode_string",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.String)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to base64Decode_string", value.Type())
		}
		decodeBytes, err := base64.StdEncoding.DecodeString(string(v))
		if err != nil {
			return types.NewErr("%v", err)
		}
		return types.String(decodeBytes)
	},
}

// 将bytes进行 base64 解码
var base64DecodeBytesFunc = &functions.Overload{
	Operator: "base64Decode_bytes",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.Bytes)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to base64Decode_bytes", value.Type())
		}
		decodeBytes, err := base64.StdEncoding.DecodeString(string(v))
		if err != nil {
			return types.NewErr("%v", err)
		}
		return types.String(decodeBytes)
	},
}

// 将字符串进行 urlencode 编码
var urlencodeStringFunc = &functions.Overload{
	Operator: "urlencode_string",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.String)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to urlencode_string", value.Type())
		}
		return types.String(url.QueryEscape(string(v)))
	},
}

// 将bytes进行 urlencode 编码
var urlencodeBytesFunc = &functions.Overload{
	Operator: "urlencode_bytes",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.Bytes)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to urlencode_bytes", value.Type())
		}
		return types.String(url.QueryEscape(string(v)))
	},
}

// 将字符串进行 urldecode 解码
var urldecodeStringFunc = &functions.Overload{
	Operator: "urldecode_string",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.String)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to urldecode_string", value.Type())
		}
		decodeString, err := url.QueryUnescape(string(v))
		if err != nil {
			return types.NewErr("%v", err)
		}
		return types.String(decodeString)
	},
}

// 将 bytes 进行 urldecode 解码
var urldecodeBytesFunc = &functions.Overload{
	Operator: "urldecode_bytes",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.Bytes)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to urldecode_bytes", value.Type())
		}
		decodeString, err := url.QueryUnescape(string(v))
		if err != nil {
			return types.NewErr("%v", err)
		}
		return types.String(decodeString)
	},
}

// 截取字符串
var substrFunc = &functions.Overload{
	Operator: "substr_string_int_int",
	Function: func(values ...ref.Val) ref.Val {
		if len(values) == 3 {
			str, ok := values[0].(types.String)
			if !ok {
				return types.NewErr("invalid string to 'substr'")
			}
			start, ok := values[1].(types.Int)
			if !ok {
				return types.NewErr("invalid start to 'substr'")
			}
			length, ok := values[2].(types.Int)
			if !ok {
				return types.NewErr("invalid length to 'substr'")
			}
			runes := []rune(str)
			if start < 0 || length < 0 || int(start+length) > len(runes) {
				return types.NewErr("invalid start or length to 'substr'")
			}
			return types.String(runes[start : start+length])
		} else {
			return types.NewErr("too many arguments to 'substr'")
		}
	},
}

// 暂停执行等待指定的秒数
var sleepFunc = &functions.Overload{
	Operator: "sleep_int",
	Unary: func(value ref.Val) ref.Val {
		v, ok := value.(types.Int)
		if !ok {
			return types.ValOrErr(value, "unexpected type '%v' passed to sleep", value.Type())
		}
		time.Sleep(time.Duration(v) * time.Second)
		return nil
	},
}

// 反连平台结果
var reverseWaitFunc = &functions.Overload{
	Operator: "reverse_wait_int",
	Binary: func(lhs ref.Val, rhs ref.Val) ref.Val {
		rev, ok := lhs.Value().(*proto.Reverse)
		if !ok {
			return types.ValOrErr(lhs, "unexpected type '%v' passed to 'wait'", lhs.Type())
		}
		timeout, ok := rhs.Value().(int64)
		if !ok {
			return types.ValOrErr(rhs, "unexpected type '%v' passed to 'wait'", rhs.Type())
		}
		return types.Bool(proto.VerifyReverse(rev, timeout))
	},
}
