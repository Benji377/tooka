package cmd

import (
	"fmt"
	"strconv"

	"github.com/Benji377/tooka/internal/core"
	"github.com/spf13/cobra"
)

var removeCmd = &cobra.Command{
	Use:   "remove [task ID]",
	Short: "Remove a task by ID",
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

		err = manager.Remove(id)
		if err != nil {
			return err
		}

		fmt.Printf("Task #%d removed.\n", id)
		return nil
	},
}
