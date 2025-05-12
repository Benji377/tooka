package cmd

import (
	"encoding/json"
	"fmt"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var (
	jsonOut bool
)

var infoCmd = &cobra.Command{
	Use:   "info",
	Short: "Show information about a task",
	Long:  `Retrieves metadata about a task. Use --json for raw JSON output.`,
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		// Check if the task name is provided
		core.Log.Info().Msg("[Tooka INFO] Retrieving task information for: " + args[0])
		taskName := args[0]

		// We attempt to get the task from the manager
		core.Log.Info().Msg("[Tooka INFO] Loading task: " + taskName)
		task, _ := taskManager.GetTask(taskName)
		if task == nil {
			fmt.Println(ui.ErrorStyle.Render(fmt.Sprintf("Task '%s' not found.", taskName)))
			core.Log.Warn().Msg("[Tooka INFO] Task not found: " + taskName)
			return
		}

		if jsonOut {
			core.Log.Info().Msg("[Tooka INFO] Outputting task information in JSON format")
			data, _ := json.MarshalIndent(task, "", "  ")
			fmt.Println(string(data))
			return
		}
		core.Log.Info().Msg("[Tooka INFO] Outputting task information in formatted style")

		box := ui.BoxStyle.Render(fmt.Sprintf(
			"%s\n\n%s\n%s\n%s",
			ui.HeaderStyle.Render("Task: "+task.Name),
			ui.LabelStyle.Render("Description: ")+ui.ValueStyle.Render(task.Description),
			ui.LabelStyle.Render("Modules: ")+ui.ValueStyle.Render(fmt.Sprintf("%d", len(task.Modules))),
			ui.LabelStyle.Render("Output: ")+ui.ValueStyle.Render(task.Output),
		))
		fmt.Println(box)
	},
}

func init() {
	infoCmd.Flags().BoolVar(&jsonOut, "json", false, "Output full JSON data")
}
