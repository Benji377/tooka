package cmd

import (
	"fmt"
	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/utils"
	"github.com/spf13/cobra"
)

var addCmd = &cobra.Command{
	Use:   "add task",
	Short: "Add a new task",
	Long:  "Add a task with from the given JSON file",
	Example: "tooka add test-task.json",
	Run: func(cmd *cobra.Command, args []string) {
		// Check if the user provided a file
		if len(args) < 1 {
			fmt.Println("Please provide a file")
			return
		}
		path := args[0]

		// Validate the structure first using gjson
		if err := utils.ValidateTaskJSON(path); err != nil {
			fmt.Printf("Invalid task JSON: %v\n", err)
			return
		}

		// Load and build the task object
		task, err := core.LoadTaskFromFile(path)
		if err != nil {
			fmt.Printf("Failed to load task: %v\n", err)
			return
		}

		// TODO: Save to DB
		fmt.Printf("Task '%s' loaded and ready to run.\n", task.Name)
	},
}
