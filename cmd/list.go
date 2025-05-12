package cmd

import (
	"fmt"
	"regexp"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List all tasks",
	Long:  `Lists all tasks, optionally filtered by name pattern.`,
	Run: func(cmd *cobra.Command, args []string) {
		// Get flags for filtering
		namePattern, _ := cmd.Flags().GetString("name")

		// Fetch tasks
		tasks := taskManager.ListTasks()

		// Apply filtering logic based on flags
		var filteredTasks []*core.Task
		for _, task := range tasks {
			if namePattern != "" {
				// Filter by name pattern using regex
				matched, _ := regexp.MatchString(namePattern, task.Name)
				if !matched {
					continue
				}
			}

			// If no filters apply, include the task
			filteredTasks = append(filteredTasks, task)
		}

		// Print filtered tasks in a table format
		printTasks(filteredTasks)
	},
}

func init() {
	// Define flags for filtering tasks
	listCmd.Flags().StringP("name", "n", "", "Filter tasks by name pattern (regex)")

	// Add the command to the root command
	rootCmd.AddCommand(listCmd)
}

func printTasks(tasks []*core.Task) {
	if len(tasks) == 0 {
		fmt.Println(ui.ErrorStyle.Render("No tasks found."))
		return
	}

	rows := [][]string{}
	for _, task := range tasks {
		rows = append(rows, []string{
			task.Name,
			fmt.Sprintf("%d", len(task.Modules)),
			task.Output,
		})
	}

	fmt.Println(ui.HeaderStyle.Render("Tasks:\n"))
	fmt.Println(ui.RenderTable([]string{"Name", "Modules", "Output"}, rows))
}
