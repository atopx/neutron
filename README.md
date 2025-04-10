# neutron

> Next Generation Vulnerability Scanner.


## Support

- [x] xray
- [ ] nuclei

## Download

```bash
go get github.com/atopx/neutron
```

## Example

```go
package main

import (
	"log"
	"sync"

	"github.com/atopx/neutron/build"
	"github.com/atopx/neutron/library/http"
	"github.com/atopx/neutron/scanner"
)

const pocYamlStr = `name: poc-yaml-weblogic-console

rules:
  - method: GET
    path: /console/login/LoginForm.jsp
    headers:
      User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:52.0) Gecko/20100101 Firefox/52.0
    expression: response.status==200
`

var urls = []string{
	"http://117.161.6.2:8180",
	"https://27.221.68.244:443",
	"http://13.75.117.202:3000",
	"https://113.108.174.45:443",
}

func main() {
	http.Setup(5, 5)
	poc, err := build.NewPocEventWithYamlStr(pocYamlStr)
	if err != nil {
		log.Fatal(err)
	}

	scan, err := scanner.New(poc)
	if err != nil {
		log.Fatal(err)
	}
	wg := new(sync.WaitGroup)
	wg.Add(len(urls))

	for _, url := range urls {
		go func() {
			var verify bool
			if poc.Rules != nil {
				verify, err = scan.Start(url, poc.Rules)
			} else {
				verify, err = scan.StartByGroups(url, poc.Groups)
			}
			if err != nil {
				log.Printf("%s scan failed: %s\n", url, err.Error())
			} else {
				log.Printf("%s scan success: %v\n", url, verify)
			}
			wg.Done()
		}()
	}

	wg.Wait()
}
```
