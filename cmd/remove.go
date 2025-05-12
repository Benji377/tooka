package cmd

import (
	"fmt"

	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var removeCmd = &cobra.Command{
	Use:   "remove [name]",
	Short: "Remove a task by name",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			fmt.Println(ui.ErrorStyle.Render("❌ Please provide a task name to remove."))
			return
		}

		name := args[0]
		if err := taskManager.RemoveTask(name); err != nil {
			fmt.Println(ui.ErrorStyle.Render(fmt.Sprintf("Error: %v", err)))
			return
		}

		fmt.Println(ui.HeaderStyle.Render(fmt.Sprintf("✅ Task '%s' removed.", name)))
	},
}
