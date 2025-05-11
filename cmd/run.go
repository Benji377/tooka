package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
)

var runCmd = &cobra.Command{
	Use:   "run",
	Short: "Run tasks",
	Long:  `Run all or specific tasks based on the provided name and options`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Running tasks...")
	},
}
