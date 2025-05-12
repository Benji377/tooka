package cmd

import (
	"fmt"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var removeCmd = &cobra.Command{
	Use:   "remove <task-name>",
	Short: "Remove a task by name",
	Long:  `Removes a task by its name. Use this command to delete a task from the task manager.`,
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		core.Log.Info().Msg("[Tooka REMOVE] Removing task: " + args[0])

		name := args[0]
		if err := taskManager.RemoveTask(name); err != nil {
			fmt.Println(ui.ErrorStyle.Render(fmt.Sprintf("Error: %v", err)))
			core.Log.Error().Err(err).Msg("[Tooka REMOVE] Failed to remove task: " + err.Error())
			return
		}
		core.Log.Info().Msg("[Tooka REMOVE] Task removed successfully")

		fmt.Println(ui.HeaderStyle.Render(fmt.Sprintf("✅ Task '%s' removed.", name)))
	},
}
