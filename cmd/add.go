package cmd

import (
	"fmt"
	"github.com/Benji377/tooka/internal/core"
	"github.com/spf13/cobra"
)

var addCmd = &cobra.Command{
	Use:   "add",
	Short: "Add a new task",
	Long:  `Add a task with specific configuration like name, interval, and command.`,
	Run: func(cmd *cobra.Command, args []string) {
		name := "ping"
		interval := 60
		command := "echo 'Ping task executed'"

		task := &core.Task{
			Name:     name,
			Schedule: fmt.Sprintf("*/%d * * * *", interval),
			Modules: []map[string]any{
				{
					"shell": map[string]any{
						"command": command,
					},
				},
			},
			Output: "task_log.txt",
		}

		taskFile := fmt.Sprintf("%s.json", name)
		err := taskManager.SaveTask(task, taskFile)
		if err != nil {
			fmt.Printf("Error saving task: %v\n", err)
			return
		}

		taskScheduler.AddTask(task)
		fmt.Printf("Task '%s' added and scheduled.\n", name)
	},
}
