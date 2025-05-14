package cmd

import (
	"fmt"
	"strconv"
	"time"

	"github.com/spf13/cobra"
	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
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

		// Prompt interactively if nothing is provided
		if newTitle == "" && newDescription == "" && newDue == "" && newPriority == -1 {
			fmt.Println("No fields provided. Launching interactive editor...")
			inputTask, err := ui.PromptForTask()
			if err != nil {
				return err
			}
			task.Title = inputTask.Title
			task.Description = inputTask.Description
			task.DueDate = inputTask.DueDate
			task.Priority = inputTask.Priority
		} else {
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

		err = manager.Edit(id, *task)
		if err != nil {
			return err
		}

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
