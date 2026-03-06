package tests

import (
	"encoding/json"
	"math/big"
	"os"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	yusdk "github.com/yu-org/yu-sdk/go"
	"github.com/yu-org/yu/apps/asset"
	"github.com/yu-org/yu/apps/poa"
	"github.com/yu-org/yu/core/keypair"
	"github.com/yu-org/yu/core/startup"
	"github.com/yu-org/yu/core/types"
)

func TestTransferAsset(t *testing.T) {
	var wg sync.WaitGroup
	wg.Add(1)
	go startChain(t, &wg)
	time.Sleep(2 * time.Second)

	runTransferTest(t)
	wg.Wait()
}

func startChain(_ *testing.T, wg *sync.WaitGroup) {
	poaCfg := poa.DefaultCfg(0)
	yuCfg := startup.InitDefaultKernelConfig()
	yuCfg.IsAdmin = true
	os.RemoveAll(yuCfg.DataDir)

	startup.InitDefaultKernel(yuCfg).
		WithTripods(
			poa.NewPoa(poaCfg),
			asset.NewAsset("YuCoin"),
		).Startup()
	wg.Done()
}

func runTransferTest(t *testing.T) {
	client := yusdk.NewClient("http://localhost:7999")

	pubkey, privkey, err := keypair.GenKeyPair(keypair.Sr25519)
	assert.NoError(t, err)
	toPubkey, _, err := keypair.GenKeyPair(keypair.Sr25519)
	assert.NoError(t, err)

	client.WithKeys(privkey, pubkey)

	sub, err := client.NewSubscriber()
	assert.NoError(t, err)
	defer sub.Close()

	eventCh := make(chan *types.Receipt, 10)
	go sub.SubEvent(eventCh)

	const (
		createAmount uint64 = 500
		transfer1    uint64 = 50
		transfer2    uint64 = 100
	)

	t.Log("--- CreateAccount ---")
	err = client.WriteChain("asset", "CreateAccount", map[string]any{"amount": createAmount}, 0, 0)
	assert.NoError(t, err)
	time.Sleep(10 * time.Second)
	assert.Equal(t, createAmount, queryBalance(t, client, pubkey.Address().String()))

	t.Log("--- Transfer 1 ---")
	err = client.WriteChain("asset", "Transfer", map[string]any{
		"to":     toPubkey.Address().String(),
		"amount": transfer1,
	}, 0, 0)
	assert.NoError(t, err)
	time.Sleep(8 * time.Second)
	assert.Equal(t, createAmount-transfer1, queryBalance(t, client, pubkey.Address().String()))
	assert.Equal(t, transfer1, queryBalance(t, client, toPubkey.Address().String()))

	t.Log("--- Transfer 2 ---")
	err = client.WriteChain("asset", "Transfer", map[string]any{
		"to":     toPubkey.Address().String(),
		"amount": transfer2,
	}, 0, 0)
	assert.NoError(t, err)
	time.Sleep(6 * time.Second)
	assert.Equal(t, createAmount-transfer1-transfer2, queryBalance(t, client, pubkey.Address().String()))
	assert.Equal(t, transfer1+transfer2, queryBalance(t, client, toPubkey.Address().String()))

	assert.NoError(t, client.StopChain())
}

func queryBalance(t *testing.T, client *yusdk.YuClient, addr string) uint64 {
	resp, err := client.ReadChain("asset", "QueryBalance", map[string]string{"account": addr})
	assert.NoError(t, err)
	var result map[string]*big.Int
	err = json.Unmarshal(resp, &result)
	assert.NoError(t, err)
	return result["amount"].Uint64()
}
