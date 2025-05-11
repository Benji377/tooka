package cmd

import (
	"fmt"
	"github.com/Benji377/tooka/internal/core"
	"github.com/spf13/cobra"
)

var taskManager *core.TaskManager
var taskScheduler *core.TaskScheduler

var version string = "1.0.0"

var rootCmd = &cobra.Command{
	Use:   "tooka",
	Short: "Tooka is your automation sidekick",
	Long:  `Tooka helps you automate tasks based on a cron-like schedule using shell commands and file watchers.`,
}

// Version command to show the version of Tooka
var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Show the version of Tooka",
	Long:  `Displays the current version of the Tooka CLI.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Printf("Tooka version %s\n", version)
	},
}

func Execute() error {
	taskScheduler = core.NewTaskScheduler()
	taskManager = core.NewTaskManager(taskScheduler)
	go taskScheduler.Start()
	return rootCmd.Execute()
}

func init() {
	rootCmd.AddCommand(runCmd)
	rootCmd.AddCommand(listCmd)
	rootCmd.AddCommand(addCmd)
	rootCmd.AddCommand(removeCmd)
	rootCmd.AddCommand(statusCmd)
	rootCmd.AddCommand(versionCmd)
}
