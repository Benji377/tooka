package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
)

var toggleCmd = &cobra.Command{
	Use:   "toggle <task_name>",
	Short: "Enable or disable a task",
	Long:  `Enable or disable a task by name. Use the --enable or --disable flag to specify the action.`,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			fmt.Println("Please provide a task name to toggle.")
			return
		}

		name := args[0]

		// Determine if we're enabling or disabling the task
		enable, _ := cmd.Flags().GetBool("enable")
		disable, _ := cmd.Flags().GetBool("disable")

		// Validate only one action is provided
		if enable && disable {
			fmt.Println("Please specify either --enable or --disable, not both.")
			return
		} else if !enable && !disable {
			fmt.Println("Please specify either --enable or --disable.")
			return
		}

		// Perform the action (enable or disable)
		err := taskManager.ToggleTask(name, enable)
		if err != nil {
			fmt.Printf("Error toggling task: %v\n", err)
			return
		}

		if enable {
			fmt.Printf("Task '%s' enabled.\n", name)
		} else {
			fmt.Printf("Task '%s' disabled.\n", name)
		}
	},
}

func init() {
	// Add flags for enabling and disabling a task
	toggleCmd.Flags().Bool("enable", false, "Enable the task")
	toggleCmd.Flags().Bool("disable", false, "Disable the task")

	// Add the new command to the root command
	rootCmd.AddCommand(toggleCmd)
}
