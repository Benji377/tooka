package cmd

import (
	"fmt"
	"log"
	"strconv"
	"time"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/shared"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var (
	newTitle       string
	newDescription string
	newDue         string
	newPriority    int
)

var editCmd = &cobra.Command{
	Use:   "edit [task ID]",
	Short: "Edit an existing task",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		id, err := strconv.Atoi(args[0])
		if err != nil {
			log.Println("Error parsing task ID:", err) // Using log for user-facing error
			return fmt.Errorf("invalid ID")
		}

		manager, err := core.NewTaskManager()
		if err != nil {
			log.Println("Error initializing task manager:", err) // Using log for user-facing error
			return err
		}

		task, err := manager.Get(id)
		if err != nil {
			log.Println("Error fetching task by ID:", err) // Using log for user-facing error
			return err
		}

		// Prompt interactively if no fields are provided
		if newTitle == "" && newDescription == "" && newDue == "" && newPriority == -1 {
			shared.Log.Debug().Msg("Launching interactive editor to update task") // Log for debug
			inputTask, err := ui.PromptForTask(task)                              // Pass existing task to pre-fill fields
			if err != nil {
				if err.Error() == "cancelled" {
					log.Println("Interactive editor canceled.") // Use log for user-facing message
					return nil                                  // Gracefully exit
				}
				shared.Log.Error().Err(err).Msg("Error while prompting for task update") // Log error details
				return err
			}
			task.Title = inputTask.Title
			task.Description = inputTask.Description
			task.DueDate = inputTask.DueDate
			task.Priority = inputTask.Priority
		} else {
			// Only update the fields provided
			if newTitle != "" {
				task.Title = newTitle
			}
			if newDescription != "" {
				task.Description = newDescription
			}
			if newDue != "" {
				due, err := time.Parse("2006-01-02", newDue)
				if err == nil {
					task.DueDate = due
				}
			}
			if newPriority >= 0 && newPriority <= 2 {
				task.Priority = core.Priority(newPriority)
			}
		}

		// Save the updated task
		err = manager.Edit(id, *task)
		if err != nil {
			shared.Log.Error().Err(err).Msg("Error updating task") // Log error details
			return err
		}

		shared.Log.Info().Msgf("Task #%d updated", id) // Log success
		fmt.Printf("Task #%d updated.\n", id)
		return nil
	},
}

func init() {
	editCmd.Flags().StringVar(&newTitle, "title", "", "New title")
	editCmd.Flags().StringVar(&newDescription, "description", "", "New description")
	editCmd.Flags().StringVar(&newDue, "due", "", "New due date (YYYY-MM-DD)")
	editCmd.Flags().IntVar(&newPriority, "priority", -1, "New priority (0:Low, 1:Medium, 2:Severe)")
}
