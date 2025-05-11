package main

import (
	"github.com/Benji377/tooka/cmd"
	"log"
)

func main() {
	if err := cmd.Execute(); err != nil {
		log.Fatalf("Error executing Tooka CLI: %v", err)
	}
}
