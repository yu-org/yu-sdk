package yusdk

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/url"

	"github.com/HyperService-Consortium/go-hexutil"
	"github.com/yu-org/yu/common"
	"github.com/yu-org/yu/core/keypair"
	"github.com/yu-org/yu/core/protocol"
)

// YuClient is the SDK client for interacting with a yu blockchain node.
type YuClient struct {
	httpURL string // e.g. "http://localhost:7999"
	wsURL   string // e.g. "ws://localhost:8999"
	privkey keypair.PrivKey
	pubkey  keypair.PubKey
}

// NewClient creates a new YuClient with the given HTTP base URL.
// The WebSocket URL defaults to port 8999 on the same host.
func NewClient(httpURL string) *YuClient {
	return &YuClient{httpURL: httpURL, wsURL: "ws://localhost:8999"}
}

// WithWsURL sets a custom WebSocket base URL for event subscription.
func (c *YuClient) WithWsURL(wsURL string) *YuClient {
	c.wsURL = wsURL
	return c
}

// WithKeys sets the key pair used to sign writing transactions.
func (c *YuClient) WithKeys(privkey keypair.PrivKey, pubkey keypair.PubKey) *YuClient {
	c.privkey, c.pubkey = privkey, pubkey
	return c
}

// WriteChain sends a signed writing call to the chain.
// params is any JSON-serializable value.
func (c *YuClient) WriteChain(tripodName, funcName string, params any, leiPrice, tips uint64) error {
	paramsByt, err := json.Marshal(params)
	if err != nil {
		return err
	}
	wrCall := &common.WrCall{
		TripodName: tripodName,
		FuncName:   funcName,
		Params:     string(paramsByt),
		LeiPrice:   leiPrice,
		Tips:       tips,
	}

	callByt, err := json.Marshal(wrCall)
	if err != nil {
		return err
	}
	msgHash := common.BytesToHash(callByt)
	sig, err := c.privkey.SignData(msgHash.Bytes())
	if err != nil {
		return err
	}

	postBody := &protocol.WritingPostBody{
		Pubkey:    c.pubkey.StringWithType(),
		Address:   c.pubkey.Address().String(),
		Signature: hexutil.Encode(sig),
		Call:      wrCall,
	}
	bodyByt, err := json.Marshal(postBody)
	if err != nil {
		return err
	}

	u := url.URL{Scheme: "http", Host: c.httpHost(), Path: protocol.WrApiPath}
	_, err = http.Post(u.String(), "application/json", bytes.NewReader(bodyByt))
	return err
}

// ReadChain sends a reading call to the chain and returns the raw response body.
// params is any JSON-serializable value.
func (c *YuClient) ReadChain(tripodName, funcName string, params any) ([]byte, error) {
	paramsByt, err := json.Marshal(params)
	if err != nil {
		return nil, err
	}
	rdCall := &common.RdCall{
		TripodName: tripodName,
		FuncName:   funcName,
		Params:     string(paramsByt),
	}
	byt, err := json.Marshal(rdCall)
	if err != nil {
		return nil, err
	}

	u := url.URL{Scheme: "http", Host: c.httpHost(), Path: protocol.RdApiPath}
	resp, err := http.Post(u.String(), "application/json", bytes.NewReader(byt))
	if err != nil {
		return nil, err
	}
	return io.ReadAll(resp.Body)
}

// httpHost returns just the "host:port" portion from the httpURL.
func (c *YuClient) httpHost() string {
	u, err := url.Parse(c.httpURL)
	if err != nil {
		return "localhost:7999"
	}
	return u.Host
}
