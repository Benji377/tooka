package cmd

import (
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/ui"
	"github.com/spf13/cobra"
)

var (
	sortFlag string
	descFlag bool
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
		sortTasks(tasks, sortFlag, descFlag)

		fmt.Println(ui.RenderTaskList(tasks))
		return nil
	},
}

func init() {
	listCmd.Flags().StringVar(&sortFlag, "sort", "due-date", "Sort tasks by: name, priority, due-date, or status")
	listCmd.Flags().BoolVar(&descFlag, "desc", false, "Reverse the sort order (descending)")
}

func sortTasks(tasks []core.Task, sortBy string, desc bool) {
	lessFunc := func(i, j int) bool { return false }

	switch strings.ToLower(sortBy) {
	case "name":
		lessFunc = func(i, j int) bool {
			return strings.ToLower(tasks[i].Title) < strings.ToLower(tasks[j].Title)
		}
	case "priority":
		lessFunc = func(i, j int) bool {
			return tasks[i].Priority < tasks[j].Priority
		}
	case "status":
		lessFunc = func(i, j int) bool {
			return taskStatusRank(tasks[i]) < taskStatusRank(tasks[j])
		}
	case "due-date", "":
		fallthrough
	default:
		lessFunc = func(i, j int) bool {
			return tasks[i].DueDate.Before(tasks[j].DueDate)
		}
	}

	sort.SliceStable(tasks, func(i, j int) bool {
		if desc {
			return !lessFunc(i, j)
		}
		return lessFunc(i, j)
	})
}

func taskStatusRank(task core.Task) int {
	now := time.Now().Truncate(24 * time.Hour)
	switch {
	case task.Completed:
		return 0
	case task.DueDate.Before(now):
		return 1
	case task.DueDate.Equal(now):
		return 2
	default:
		return 3
	}
}
