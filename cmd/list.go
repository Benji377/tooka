package cmd

import (
	"github.com/spf13/cobra"
)

var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List all tasks",
	Long:  `Lists all tasks, optionally filtered by name or state.`,
	Run: func(cmd *cobra.Command, args []string) {
		taskManager.ListTasks()
	},
}
