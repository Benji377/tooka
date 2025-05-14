package main

import (
	"log"

	"github.com/Benji377/tooka/cmd"
	"github.com/Benji377/tooka/internal/shared"
)

func main() {
	// Initialize the logger
	shared.InitLogger()
	log.SetFlags(log.Ldate | log.Ltime | log.Lshortfile)
	if err := cmd.Execute(); err != nil {
		log.Fatalf("Error executing Tooka CLI: %v", err)
	}
}
