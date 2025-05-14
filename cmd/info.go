package cmd

import (
	"fmt"
	"log"
	"strconv"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/shared"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var infoCmd = &cobra.Command{
	Use:   "info [task ID]",
	Short: "Show detailed info about a task",
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

		task, err := manager.Get(id)
		if err != nil {
			log.Println("Error fetching task info:", err) // User-facing log
			return err
		}

		shared.Log.Debug().Msgf("Rendering task details for task ID: %d", id) // Debug log for detailed info
		fmt.Println(ui.RenderTaskDetails(*task))
		return nil
	},
}
