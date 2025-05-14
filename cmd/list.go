package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
)

var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List all tasks",
	RunE: func(cmd *cobra.Command, args []string) error {
		manager, err := core.NewTaskManager()
		if err != nil {
			return err
		}
		tasks := manager.List()
		fmt.Println(ui.RenderTaskList(tasks))
		return nil
	},
}
