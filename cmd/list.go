package cmd

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/Benji377/tooka/internal/core"
	"github.com/spf13/cobra"
)

var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List all tasks",
	Long:  `Lists all tasks, optionally filtered by name pattern and enabled/disabled status.`,
	Run: func(cmd *cobra.Command, args []string) {
		// Get flags for filtering
		namePattern, _ := cmd.Flags().GetString("name")
		enabled, _ := cmd.Flags().GetBool("enabled") // Filter by enabled/disabled status

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
			if enabled && !task.Enabled {
				// Filter by enabled status (only include enabled tasks)
				continue
			}
			if !enabled && task.Enabled {
				// Filter by disabled status (only include disabled tasks)
				continue
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
	listCmd.Flags().BoolP("enabled", "e", false, "Only show enabled tasks") // New flag for enabled tasks

	// Add the command to the root command
	rootCmd.AddCommand(listCmd)
}

// printTasks prints tasks in a table format
func printTasks(tasks []*core.Task) {
	// Headers with styling using lipgloss
	header := fmt.Sprintf("%-20s %-20s %-10s %-20s %-10s", "Name", "Schedule", "Modules", "Output", "Enabled")
	fmt.Println(header)
	fmt.Println(strings.Repeat("=", len(header)))

	// Rows for each task
	for _, task := range tasks {
		enabledStr := "No"
		if task.Enabled {
			enabledStr = "Yes"
		}
		fmt.Printf("%-20s %-20s %-10d %-20s %-10s\n", task.Name, task.Schedule, len(task.Modules), task.Output, enabledStr)
	}
}
