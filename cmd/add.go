package cmd

import (
	"fmt"
	"log"
	"time"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/shared"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var (
	title       string
	description string
	dueDate     string
	priority    int
)

var addCmd = &cobra.Command{
	Use:   "add",
	Short: "Add a new task",
	RunE: func(cmd *cobra.Command, args []string) error {
		manager, err := core.NewTaskManager()
		if err != nil {
			log.Println("Error initializing task manager:", err) // Using log for console error
			return err
		}

		var task *core.Task

		// Use interactive prompt if required fields aren't provided
		if title == "" || dueDate == "" {
			// Launch interactive UI to fill the fields
			task, err = ui.PromptForTask(nil)
			if err != nil {
				if err.Error() == "cancelled" {
					// If the user pressed ESC or canceled, just exit gracefully
					log.Println("Interactive editor canceled.") // Use log for user-facing message
					return nil                                  // Gracefully exit
				}
				shared.Log.Error().Err(err).Msg("Error while prompting for task") // Log the error in detail
				return err
			}
		} else {
			// Create a task from the provided fields
			due, err := time.Parse("2006-01-02", dueDate)
			if err != nil {
				shared.Log.Error().Err(err).Msg("Invalid due date format") // Log error details
				return fmt.Errorf("invalid due date format, use YYYY-MM-DD")
			}
			task = &core.Task{
				Title:       title,
				Description: description,
				DueDate:     due,
				Priority:    core.Priority(priority),
			}
		}

		// Save the newly added task
		if err := manager.Add(*task); err != nil {
			shared.Log.Error().Err(err).Msg("Error adding task") // Log the error details
			return err
		}

		shared.Log.Info().Msgf("Task added: %s", task.Title) // Log success
		fmt.Println("Task added:", task.Title)
		return nil
	},
}

func init() {
	addCmd.Flags().StringVarP(&title, "title", "t", "", "Title of the task")
	addCmd.Flags().StringVarP(&description, "description", "d", "", "Description")
	addCmd.Flags().StringVarP(&dueDate, "due", "D", "", "Due date (YYYY-MM-DD)")
	addCmd.Flags().IntVarP(&priority, "priority", "p", 0, "Priority (0:Low, 1:Medium, 2:Severe)")
}
