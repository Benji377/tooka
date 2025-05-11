package cmd

import (
	"fmt"
	"time"

	"github.com/spf13/cobra"
	"github.com/Benji377/tooka/internal/core"
	"github.com/charmbracelet/lipgloss"
)

var statusCmd = &cobra.Command{
	Use:   "status",
	Short: "Show Tooka status",
	Long:  `Shows the current status of the Tooka scheduler, running tasks, and next runs.`,
	Run: func(cmd *cobra.Command, args []string) {
		taskManager := core.GetManager("your/backup/dir")
		tasks := taskManager.ListTasks()

		// Lipgloss styles
		boldStyle := lipgloss.NewStyle().Bold(true)
		highlightStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("205")).Bold(true)
		greenStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("2"))
		grayStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("240"))
		blueStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("33"))

		// Display overall status
		totalTasks := len(tasks)
		disabledTasks := 0
		tasksRunLast24Hours := 0
		nextRuns := make(map[string]time.Time)

		// Get current time to calculate last run time for 24 hours
		now := time.Now()
		oneDayAgo := now.Add(-24 * time.Hour)

		// Iterate through tasks and gather relevant information
		for _, task := range tasks {
			if !task.Enabled {
				disabledTasks++
			}

			// Checking if task has been run in the last 24 hours
			if !task.LastRun.IsZero() && task.LastRun.After(oneDayAgo) {
				tasksRunLast24Hours++
			}

			// Determine the next run for scheduled tasks
			if job, exists := taskScheduler.Jobs[task.Name]; exists {
				nextRun := job.NextRun()
				nextRuns[task.Name] = nextRun
			}
		}

		// Format and display the status with lipgloss
		fmt.Println(highlightStyle.Render("Tooka Status"))
		fmt.Println(boldStyle.Render("Total Tasks: ") + fmt.Sprintf("%d", totalTasks))
		fmt.Println(grayStyle.Render("Disabled Tasks: ") + fmt.Sprintf("%d", disabledTasks))
		fmt.Println(greenStyle.Render("Tasks Run in the Last 24 Hours: ") + fmt.Sprintf("%d", tasksRunLast24Hours))

		// Print next scheduled runs for each task
		fmt.Println("\n" + blueStyle.Render("Next Runs:"))
		for taskName, nextRun := range nextRuns {
			fmt.Printf("%s %s next run: %s\n", blueStyle.Render(taskName), blueStyle.Render("->"), nextRun.Format(time.RFC3339))
		}
	},
}

