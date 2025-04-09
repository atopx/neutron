package proto

import (
	"bytes"
	"fmt"
	"net/url"
	"time"

	"github.com/atopx/neutron/library/http"
	"github.com/atopx/neutron/library/utils"
	"github.com/valyala/fasthttp"
)

var reverseDomain, reverseToken string

func SetupReverse(domain, token string) {
	reverseDomain = domain
	reverseToken = token
}

func NewReverse() *Reverse {
	if reverseDomain == "" {
		return &Reverse{}
	}
	var flag = utils.RandLetterNumbers(8)
	urlStr := fmt.Sprintf("http://%s.%s", flag, reverseDomain)
	u, _ := url.Parse(urlStr)
	return &Reverse{
		Url:                SetupURL(u),
		Flag:               flag,
		Domain:             u.Hostname(),
		Ip:                 "",
		IsDomainNameServer: false,
	}
}

// ceye.io api
const ceyeIoApi = "http://api.ceye.io/v1/records?token=%s&type=%s&filter=%s"

var emptyData = utils.Bytes(`"data": []`)

// VerifyReverse 验证反连平台
func VerifyReverse(r *Reverse, timeout int64) bool {

	if reverseToken == "" {
		return false
	}
	// 延迟 x 秒获取结果
	time.Sleep(time.Second * time.Duration(timeout))

	//check dns
	if getReverseResp(fmt.Sprintf(ceyeIoApi, reverseToken, "dns", r.Flag)) {
		return true
	}

	//check request
	if getReverseResp(fmt.Sprintf(ceyeIoApi, reverseToken, "http", r.Flag)) {
		return true
	}

	return false
}

// getReverseResp 发送请求
func getReverseResp(verifyUrl string) bool {
	var origin = fasthttp.AcquireRequest()
	origin.Header.SetMethod(fasthttp.MethodGet)
	origin.SetRequestURI(verifyUrl)
	response, err := http.Do(origin, false)
	defer func() {
		response.SetConnectionClose()
		fasthttp.ReleaseResponse(response)
	}()
	if err != nil {
		return false
	}

	return !bytes.Contains(response.Body(), emptyData)
}
