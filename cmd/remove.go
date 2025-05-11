package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
)

var removeCmd = &cobra.Command{
	Use:   "remove",
	Short: "Remove a task",
	Long:  `Remove a task by name.`,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			fmt.Println("Please provide a task name to remove.")
			return
		}

		name := args[0]
		err := taskManager.RemoveTask(name)
		if err != nil {
			fmt.Printf("Error removing task: %v\n", err)
			return
		}
		fmt.Printf("Task '%s' removed.\n", name)
	},
}
