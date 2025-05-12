package main

import (
	"log"

	"github.com/Benji377/tooka/cmd"
	"github.com/Benji377/tooka/internal/core"
)

func main() {
	// Initialize the logger
	core.InitLogger()
	if err := cmd.Execute(); err != nil {
		log.Fatalf("Error executing Tooka CLI: %v", err)
	}
}
