package cmd

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"
	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
)

var infoCmd = &cobra.Command{
	Use:   "info [task ID]",
	Short: "Show detailed info about a task",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		id, err := strconv.Atoi(args[0])
		if err != nil {
			return fmt.Errorf("invalid ID")
		}

		manager, err := core.NewTaskManager()
		if err != nil {
			return err
		}

		task, err := manager.Get(id)
		if err != nil {
			return err
		}

		fmt.Println(ui.RenderTaskDetails(*task))
		return nil
	},
}
