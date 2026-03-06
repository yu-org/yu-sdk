package yusdk

import (
	"net/http"
	"net/url"

	"github.com/yu-org/yu/core/protocol"
)

// StopChain sends an admin stop request to the chain node.
func (c *YuClient) StopChain() error {
	u := url.URL{Scheme: "http", Host: c.httpHost(), Path: protocol.AdminApiPath + "/stop"}
	_, err := http.Get(u.String())
	return err
}
