package cmd

import (
	"fmt"
	"log"
	"strconv"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/shared"
	"github.com/spf13/cobra"
)

var toggleCmd = &cobra.Command{
	Use:   "toggle [task ID]",
	Short: "Toggle completion of a task",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		id, err := strconv.Atoi(args[0])
		if err != nil {
			log.Println("Error parsing task ID:", err) // User-facing log
			return fmt.Errorf("invalid ID")
		}

		manager, err := core.NewTaskManager()
		if err != nil {
			log.Println("Error initializing task manager:", err) // User-facing log
			return err
		}

		err = manager.ToggleComplete(id)
		if err != nil {
			shared.Log.Error().Err(err).Msgf("Error toggling completion for task #%d", id) // Detailed error log
			return err
		}

		shared.Log.Info().Msgf("Toggled completion for task #%d", id) // Success log
		fmt.Printf("Toggled completion of task #%d\n", id)
		return nil
	},
}
