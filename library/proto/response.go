package proto

import (
	"net/url"
	"strings"
	"sync"

	"github.com/atopx/neutron/library/utils"
	"github.com/valyala/fasthttp"
)

var responsePool = sync.Pool{New: func() any { return new(Response) }}

// SetupResponse create a response object
func SetupResponse(response *fasthttp.Response, req *fasthttp.Request) (*Response, error) {
	u, err := url.Parse(req.URI().String())
	if err != nil {
		return nil, err
	}
	headers := make(map[string]string)
	lines := strings.Split(response.Header.String(), "\r\n")
	for i := 0; i < len(lines); i++ {
		values := strings.SplitN(lines[i], ":", 2)
		if len(values) != 2 {
			continue
		}
		headers[strings.ToLower(values[0])] = strings.TrimSpace(values[1])
	}
	resp := responsePool.Get().(*Response)
	resp.Url = SetupURL(u)
	resp.Headers = headers
	resp.Status = int32(response.StatusCode())
	resp.ContentType = string(response.Header.Peek("Content-Type"))
	resp.Body = response.Body()
	resp.BodyString = utils.String(resp.Body)
	return resp, nil
}

// ReleaseResponse recycle a response object
func ReleaseResponse(response *Response) {
	if response != nil {
		response.Reset()
		responsePool.Put(response)
	}
}
