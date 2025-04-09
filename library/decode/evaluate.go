package decode

import (
	"errors"
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

// func (c *CelLibrary) UpdateCompileOptions(args map[string]string) {
// 	for k, v := range args {
// 		var d *exp.Decl
// 		if strings.HasPrefix(v, "randomInt") {
// 			d = decls.NewVar(k, decls.Int)
// 		} else if strings.HasPrefix(v, "newReverse") {
// 			d = decls.NewVar(k, decls.NewObjectType("proto.Reverse"))
// 		} else {
// 			d = decls.NewVar(k, decls.String)
// 		}
// 		c.EnvOptions = append(c.EnvOptions, cel.Declarations(d))
// 	}
// }

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
	c.EnvOptions = []cel.EnvOption{
		cel.Container("proto"),
		// 对象类型注入
		cel.Types(
			&proto.UrlType{},
			&proto.Request{},
			&proto.Response{},
			&proto.Reverse{},
		),
		// 定义对象
		cel.VariableDecls(
			decls.NewVariable("request", cel.ObjectType("proto.Request")),
			decls.NewVariable("response", cel.ObjectType("proto.Response")),
		),

		// 定义运算符
		// Declarations option extends the declaration set configured in the environment.
		//
		// Note: Declarations will by default be appended to the pre-existing declaration set configured
		// for the environment. The NewEnv call builds on top of the standard CEL declarations. For a
		// purely custom set of declarations use NewCustomEnv.
		//
		// Deprecated: use FunctionDecls and VariableDecls or FromConfig instead.

		cel.Declarations(
			bytesBContainsBytesDecl, stringIContainsStringDecl, stringBmatchBytesDecl, md5StringDecl,
			stringInMapKeyDecl, randomIntDecl, randomLowercaseDecl, base64StringDecl,
			base64BytesDecl, base64DecodeStringDecl, base64DecodeBytesDecl, urlencodeStringDecl,
			urlencodeBytesDecl, urldecodeStringDecl, urldecodeBytesDecl, substrDecl, sleepDecl, reverseWaitDecl,
		),
	}
	// 定义运算逻辑
	// Functions adds function overloads that extend or override the set of CEL built-ins.
	//
	// Deprecated: use Function() instead to declare the function, its overload signatures,
	// and the overload implementations.
	c.ProOptions = []cel.ProgramOption{cel.Functions(
		containsStringFunc, stringIContainsStringFunc, bytesBContainsBytesFunc, matchesStringFunc, md5StringFunc,
		stringInMapKeyFunc, randomIntFunc, randomLowercaseFunc, stringBmatchBytesFunc, base64StringFunc,
		base64BytesFunc, base64DecodeStringFunc, base64DecodeBytesFunc, urlencodeStringFunc, urlencodeBytesFunc,
		urldecodeStringFunc, urldecodeBytesFunc, substrFunc, sleepFunc, reverseWaitFunc,
	)}
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
		return nil, iss.Err()
	}
	prg, err := env.Program(ast)
	if err != nil {
		return nil, err
	}
	out, _, err := prg.Eval(params)
	if err != nil {
		return nil, err
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
	return errors.New("not matched")
}
