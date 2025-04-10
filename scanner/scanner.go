package scanner

import (
	"fmt"
	"net/url"
	"strings"

	"maps"

	"github.com/atopx/neutron/build"
	"github.com/atopx/neutron/library/decode"
	"github.com/atopx/neutron/library/http"
	"github.com/atopx/neutron/library/proto"
	"github.com/google/cel-go/cel"
	"github.com/valyala/fasthttp"
)


type scanner struct {
	env *cel.Env
	set map[string]any
}

func (s *scanner) Start(target string, rules []build.PocRule) (verify bool, err error) {
	for i := 0; i < len(rules); i++ {
		rules[i].DecodeSet(s.set)
		verify, err = s.scan(target, &rules[i])
		if request, ok := s.set["request"]; ok {
			// 回收 model request
			proto.ReleaseRequest(request.(*proto.Request))
		}
		if response, ok := s.set["response"]; ok {
			// 回收 model response
			proto.ReleaseResponse(response.(*proto.Response))
		}
		if err != nil {
			return verify, err
		}
		if !verify {
			return verify, nil
		}
	}
	return verify, nil
}

func (s *scanner) StartByGroups(target string, groups map[string][]build.PocRule) (verify bool, err error) {
	for _, rules := range groups {
		for i := 0; i < len(rules); i++ {
			if verify, err = s.Start(target, rules); err != nil {
				return verify, err
			}
			if verify {
				return verify, nil
			}
		}
	}
	return verify, nil
}

// scan 扫描逻辑
func (s *scanner) scan(target string, rule *build.PocRule) (bool, error) {
	
	set := make(map[string]any, len(s.set))
	maps.Copy(set, s.set)

	urlpath, _ := url.JoinPath(target, rule.Path)

	request, err := proto.SetupRequest(rule.Method, urlpath, rule.Body, rule.Headers)
	if err != nil {
		return false, err
	}
	set["request"] = request
	origin := request.ToFasthttp()
	response, err := http.Do(origin, rule.FollowRedirects)
	defer func() {
		response.SetConnectionClose()
		fasthttp.ReleaseResponse(response)
	}()
	if err != nil {
		return false, err
	}
	resp, err := proto.SetupResponse(response, origin)
	if err != nil {
		return false, err
	}
	if rule.Search != "" {
		if err = decode.Search(strings.TrimSpace(rule.Search), string(resp.Body), set); err != nil {
			return false, nil
		}
		return true, nil
	}
	set["response"] = resp
	out, err := decode.Evaluate(s.env, rule.Expression, set)
	if err != nil {
		return false, fmt.Errorf("scan failed: %w", err)
	}
	return out.Value().(bool), nil
}

func New(poc *build.PocEvent) ( *scanner, error) {
	env, err := decode.NewCelEnv(poc.Set)
	if err != nil {
		return nil, fmt.Errorf("parse poc env failed: %w", err)
	}
	scan := &scanner{
		env: env,
		set: poc.DecodeSet(env),
	}
	return scan, nil
}

