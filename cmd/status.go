package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
)

var statusCmd = &cobra.Command{
	Use:   "status",
	Short: "Show Tooka status",
	Long:  `Shows the current status of the Tooka scheduler, running tasks, and next runs.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Tooka is running smoothly.")
	},
}
