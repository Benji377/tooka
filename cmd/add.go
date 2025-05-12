package cmd

import (
	"fmt"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var addCmd = &cobra.Command{
	Use:   "add <task-file>",
	Short: "Add a new task",
	Long:  "Add a task from the given JSON file",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		core.Log.Info().Msg("[Tooka ADD] Adding a new task: " + args[0])
		path := args[0]

		// We attempt to create a Task object from the provided file
		core.Log.Info().Msg("[Tooka ADD] Loading task from file: " + path)
		task, err := core.LoadTaskFromFile(path)
		if err != nil {
			fmt.Println(ui.ErrorStyle.Render(fmt.Sprintf("Error: Failed to load task: %v", err)))
			core.Log.Error().Err(err).Msg("[Tooka ADD] Failed to load task: " + err.Error())
			return
		}

		// Add the task to the manager
		core.Log.Info().Msg("[Tooka ADD] Adding task to manager")
		if err := taskManager.AddTask(task); err != nil {
			fmt.Println(ui.ErrorStyle.Render(fmt.Sprintf("Error: Failed to add task: %v", err)))
			core.Log.Error().Err(err).Msg("[Tooka ADD] Failed to add task: " + err.Error())
			return
		}
		core.Log.Info().Msg("[Tooka ADD] Task added successfully")

		output := fmt.Sprintf(
			"✅ Task '%s' loaded successfully!\n\n%s\n%s",
			ui.HeaderStyle.Render(task.Name),
			ui.LabelStyle.Render("Description:")+" "+ui.ValueStyle.Render(task.Description),
			ui.LabelStyle.Render("Modules Loaded:")+" "+ui.ValueStyle.Render(fmt.Sprintf("%d", len(task.Modules))),
		)
		fmt.Println(ui.BoxStyle.Render(output))
	},
}
