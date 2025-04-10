package decode

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/atopx/neutron/library/proto"
	"github.com/google/cel-go/cel"
	"github.com/google/cel-go/common/decls"
	"github.com/google/cel-go/common/types/ref"
)

type CelLibrary struct {
	EnvOptions []cel.EnvOption
	ProOptions []cel.ProgramOption
}

func (c *CelLibrary) CompileOptions() []cel.EnvOption {
	return c.EnvOptions
}

func (c *CelLibrary) ProgramOptions() []cel.ProgramOption {
	return c.ProOptions
}

func (c *CelLibrary) UpdateCompileOptions(args map[string]string) {
	for k, v := range args {
		var d *decls.VariableDecl
		if strings.HasPrefix(v, "randomInt") {
			d = decls.NewVariable(k, cel.IntType)
		} else if strings.HasPrefix(v, "newReverse") {
			d = decls.NewVariable(k, cel.ObjectType("proto.Reverse"))
		} else {
			d = decls.NewVariable(k, cel.StringType)
		}
		c.EnvOptions = append(c.EnvOptions, cel.VariableDecls(d))
	}
}

func NewCelOption() (c CelLibrary) {
	// 使用新的 cel.Function API 替代已弃用的 cel.Declarations
	c.EnvOptions = []cel.EnvOption{
		cel.Container("proto"),
		// 对象类型注入
		cel.Types(&proto.UrlType{}, &proto.Request{}, &proto.Response{}, &proto.Reverse{}),

		// 定义对象
		cel.VariableDecls(
			decls.NewVariable("request", cel.ObjectType("proto.Request")),
			decls.NewVariable("response", cel.ObjectType("proto.Response")),
			decls.NewVariable("reverse", cel.ObjectType("proto.Reverse")),
		),

		// 函数定义
		cel.Function("contains",
			cel.MemberOverload(containsStringFunc.Operator,
				[]*cel.Type{cel.StringType, cel.StringType},
				cel.BoolType,
				cel.BinaryBinding(containsStringFunc.Binary))),

		cel.Function("icontains",
			cel.MemberOverload(stringIContainsStringFunc.Operator,
				[]*cel.Type{cel.StringType, cel.StringType},
				cel.BoolType,
				cel.BinaryBinding(stringIContainsStringFunc.Binary))),

		cel.Function("bcontains",
			cel.MemberOverload(bytesBContainsBytesFunc.Operator,
				[]*cel.Type{cel.BytesType, cel.BytesType},
				cel.BoolType,
				cel.BinaryBinding(bytesBContainsBytesFunc.Binary))),

		cel.Function("match",
			cel.Overload(matchesStringFunc.Operator,
				[]*cel.Type{cel.StringType, cel.StringType},
				cel.BoolType,
				cel.BinaryBinding(matchesStringFunc.Binary))),

		cel.Function("bmatch",
			cel.Overload(stringBmatchBytesFunc.Operator,
				[]*cel.Type{cel.StringType, cel.BytesType},
				cel.BoolType,
				cel.BinaryBinding(stringBmatchBytesFunc.Binary))),

		cel.Function("md5",
			cel.Overload(md5StringFunc.Operator,
				[]*cel.Type{cel.StringType},
				cel.StringType,
				cel.UnaryBinding(md5StringFunc.Unary))),

		cel.Function("randomInt",
			cel.Overload(randomIntFunc.Operator,
				[]*cel.Type{cel.IntType, cel.IntType},
				cel.IntType,
				cel.BinaryBinding(randomIntFunc.Binary))),
		
		cel.Function("randomLowercase",
			cel.Overload(randomLowercaseFunc.Operator,
				[]*cel.Type{cel.IntType},
				cel.StringType,
				cel.UnaryBinding(randomLowercaseFunc.Unary))),

		cel.Function("base64",
			cel.Overload(base64StringFunc.Operator,
				[]*cel.Type{cel.StringType},
				cel.StringType,
				cel.UnaryBinding(base64StringFunc.Unary))),

		cel.Function("base64",
			cel.Overload(base64BytesFunc.Operator,
				[]*cel.Type{cel.BytesType},
				cel.StringType,
				cel.UnaryBinding(base64BytesFunc.Unary))),

		cel.Function("base64Decode",
			cel.Overload(base64DecodeStringFunc.Operator,
				[]*cel.Type{cel.StringType},
				cel.StringType,
				cel.UnaryBinding(base64DecodeStringFunc.Unary))),

		cel.Function("base64Decode",
			cel.Overload(base64DecodeBytesFunc.Operator,
				[]*cel.Type{cel.BytesType},
				cel.StringType,
				cel.UnaryBinding(base64DecodeBytesFunc.Unary))),

		cel.Function("urlencode",
			cel.Overload(urlencodeStringFunc.Operator,
				[]*cel.Type{cel.StringType},
				cel.StringType,
				cel.UnaryBinding(urlencodeStringFunc.Unary))),

		cel.Function("urlencode",
			cel.Overload(urlencodeBytesFunc.Operator,
				[]*cel.Type{cel.BytesType},
				cel.StringType,
				cel.UnaryBinding(urlencodeBytesFunc.Unary))),

		cel.Function("urldecode",
			cel.Overload(urldecodeStringFunc.Operator,
				[]*cel.Type{cel.StringType},
				cel.StringType,
				cel.UnaryBinding(urldecodeStringFunc.Unary))),

		cel.Function("urldecode",
			cel.Overload(urldecodeBytesFunc.Operator,
				[]*cel.Type{cel.BytesType},
				cel.StringType,
				cel.UnaryBinding(urldecodeBytesFunc.Unary))),

		cel.Function("substr",
			cel.Overload("substr_string_int_int",
				[]*cel.Type{cel.StringType, cel.IntType, cel.IntType},
				cel.StringType,
				cel.FunctionBinding(substrFunc.Function))),

		cel.Function("sleep",
			cel.Overload("sleep_int",
				[]*cel.Type{cel.IntType},
				cel.NullType,
				cel.UnaryBinding(sleepFunc.Unary))),

		cel.Function("wait",
			cel.MemberOverload("reverse_wait_int",
				[]*cel.Type{cel.AnyType, cel.IntType},
				cel.BoolType,
				cel.BinaryBinding(reverseWaitFunc.Binary))),
	}

	// ProOptions 保持为空数组
	c.ProOptions = []cel.ProgramOption{}
	return c
}

// NewCelEnv new env from set
func NewCelEnv(set map[string]string) (*cel.Env, error) {
	option := NewCelOption()
	if set != nil {
		option.UpdateCompileOptions(set)
	}
	return cel.NewEnv(cel.Lib(&option))
}

// Evaluate 执行运算
func Evaluate(env *cel.Env, expression string, params map[string]any) (ref.Val, error) {
	ast, iss := env.Compile(expression)
	if iss.Err() != nil {
		return nil, fmt.Errorf("compile error: %w", iss.Err())
	}
	f, err := env.Program(ast)
	if err != nil {
		return nil, fmt.Errorf("program error: %w", err)
	}
	out, _, err := f.Eval(params)
	if err != nil {
		return nil, fmt.Errorf("evaluate error: %w", err)
	}
	return out, nil
}

// Search 完成正则匹配
func Search(re string, body string, set map[string]any) error {
	r, err := regexp.Compile(re)
	if err != nil {
		return err
	}
	result := r.FindStringSubmatch(body)
	names := r.SubexpNames()
	if len(result) > 1 && len(names) > 1 {
		for i, name := range names {
			if i > 0 && i <= len(result) {
				set[name] = result[i]
			}
		}
		return nil
	}
	return fmt.Errorf("not matched")
}
