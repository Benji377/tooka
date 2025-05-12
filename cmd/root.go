package cmd

import (
	"github.com/Benji377/tooka/internal/core"
	"github.com/spf13/cobra"
)

var taskManager *core.TaskManager

var rootCmd = &cobra.Command{
	Use:     "tooka",
	Short:   "Tooka is your automation sidekick",
	Long:    `Tooka helps you automate tasks based on a cron-like schedule using shell commands and file watchers.`,
	Version: core.Version,
}

func Execute() error {
	taskManager = core.GetManager(core.TasksDir)
	return rootCmd.Execute()
}

func init() {
	rootCmd.AddCommand(runCmd)
	rootCmd.AddCommand(listCmd)
	rootCmd.AddCommand(addCmd)
	rootCmd.AddCommand(removeCmd)
	rootCmd.AddCommand(infoCmd)
	rootCmd.AddCommand(moduleCmd)
}
