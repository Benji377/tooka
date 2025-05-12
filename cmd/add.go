package cmd

import (
	"fmt"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var addCmd = &cobra.Command{
	Use:     "add [task.json]",
	Short:   "Add a new task",
	Long:    "Add a task from the given JSON file",
	Example: "tooka add tasks/my-task.json",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) < 1 {
			fmt.Println(ui.ErrorStyle.Render("❌ Please provide a task file."))
			return
		}
		path := args[0]

		task, err := core.LoadTaskFromFile(path)
		if err != nil {
			fmt.Println(ui.ErrorStyle.Render(fmt.Sprintf("Error: Failed to load task: %v", err)))
			return
		}

		// Add the task to the manager
		if err := taskManager.AddTask(task); err != nil {
			fmt.Println(ui.ErrorStyle.Render(fmt.Sprintf("Error: Failed to add task: %v", err)))
			return
		}

		output := fmt.Sprintf(
			"✅ Task '%s' loaded successfully!\n\n%s\n%s",
			ui.HeaderStyle.Render(task.Name),
			ui.LabelStyle.Render("Description:")+" "+ui.ValueStyle.Render(task.Description),
			ui.LabelStyle.Render("Modules Loaded:")+" "+ui.ValueStyle.Render(fmt.Sprintf("%d", len(task.Modules))),
		)
		fmt.Println(ui.BoxStyle.Render(output))
	},
}
