package main

import (
	"flag"
	"os"
	"time"

	"github.com/yu-org/yu/apps/asset"
	"github.com/yu-org/yu/apps/poa"
	"github.com/yu-org/yu/core/startup"
)

func main() {
	timeout := flag.Duration("timeout", 0, "auto-stop the chain after this duration (e.g. 30s). 0 means run forever.")
	flag.Parse()

	poaCfg := poa.DefaultCfg(0)
	yuCfg := startup.InitDefaultKernelConfig()
	yuCfg.IsAdmin = true

	os.RemoveAll(yuCfg.DataDir)

	chain := startup.InitDefaultKernel(yuCfg).
		WithTripods(
			poa.NewPoa(poaCfg),
			asset.NewAsset("YuCoin"),
		)

	chain.Startup()

	if *timeout > 0 {
		time.Sleep(*timeout)
		chain.Stop()
	} else {
		// Block forever until OS signal
		select {}
	}
}
