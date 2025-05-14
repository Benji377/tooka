package cmd

import (
	"fmt"
	"strconv"

	"github.com/spf13/cobra"
	"github.com/Benji377/tooka/internal/core"
)

var toggleCmd = &cobra.Command{
	Use:   "toggle [task ID]",
	Short: "Toggle completion of a task",
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

		err = manager.ToggleComplete(id)
		if err != nil {
			return err
		}

		fmt.Printf("Toggled completion of task #%d\n", id)
		return nil
	},
}
