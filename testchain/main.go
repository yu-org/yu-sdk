package main

import (
	"os"

	"github.com/yu-org/yu/apps/asset"
	"github.com/yu-org/yu/apps/poa"
	"github.com/yu-org/yu/core/startup"
)

func main() {
	poaCfg := poa.DefaultCfg(0)
	yuCfg := startup.InitDefaultKernelConfig()
	yuCfg.IsAdmin = true

	// Reset history data for a clean test run
	os.RemoveAll(yuCfg.DataDir)

	startup.InitDefaultKernel(yuCfg).
		WithTripods(
			poa.NewPoa(poaCfg),
			asset.NewAsset("YuCoin"),
		).Startup()
}
