package yusdk

import (
	"net/url"

	"github.com/gorilla/websocket"
	"github.com/yu-org/yu/core/protocol"
	"github.com/yu-org/yu/core/types"
)

// Subscriber listens to chain events over WebSocket.
type Subscriber struct {
	conn *websocket.Conn
}

// NewSubscriber creates a new Subscriber connected to the chain's event stream.
func (c *YuClient) NewSubscriber() (*Subscriber, error) {
	wsHost, err := wsHost(c.wsURL)
	if err != nil {
		return nil, err
	}
	u := url.URL{Scheme: "ws", Host: wsHost, Path: protocol.SubResultsPath}
	conn, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		return nil, err
	}
	return &Subscriber{conn: conn}, nil
}

// SubEvent blocks and delivers receipts to ch until the connection is closed.
// Call Close() to stop.
func (s *Subscriber) SubEvent(ch chan<- *types.Receipt) {
	for {
		_, msg, err := s.conn.ReadMessage()
		if err != nil {
			return
		}
		r := new(types.Receipt)
		if err := r.Decode(msg); err != nil {
			continue
		}
		if ch != nil {
			ch <- r
		}
	}
}

// Close terminates the WebSocket connection.
func (s *Subscriber) Close() error {
	return s.conn.Close()
}

func wsHost(wsURL string) (string, error) {
	u, err := url.Parse(wsURL)
	if err != nil {
		return "localhost:8999", err
	}
	return u.Host, nil
}
