package cmd

import (
	"encoding/json"
	"fmt"

	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var (
	taskName string
	jsonOut  bool
)

var infoCmd = &cobra.Command{
	Use:   "info",
	Short: "Show information about a task",
	Long:  `Retrieves metadata about a task. Use --json for raw JSON output.`,
	Run: func(cmd *cobra.Command, args []string) {
		if taskName == "" {
			fmt.Println(ui.ErrorStyle.Render("❌ Please provide a task name using --name"))
			return
		}

		task, _ := taskManager.GetTask(taskName)
		if task == nil {
			fmt.Println(ui.ErrorStyle.Render(fmt.Sprintf("Task '%s' not found.", taskName)))
			return
		}

		if jsonOut {
			data, _ := json.MarshalIndent(task, "", "  ")
			fmt.Println(string(data))
			return
		}

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
	infoCmd.Flags().StringVarP(&taskName, "name", "n", "", "Task name")
	infoCmd.Flags().BoolVar(&jsonOut, "json", false, "Output full JSON data")
}
