package cmd

import (
	"github.com/Benji377/tooka/internal/core"
	"github.com/spf13/cobra"
	"os"
	"path/filepath"
)

var taskManager *core.TaskManager
var tasksDir = filepath.Join(os.Getenv("HOME"), ".tooka", "tasks")
var version string = "1.0.0"

var rootCmd = &cobra.Command{
	Use:     "tooka",
	Short:   "Tooka is your automation sidekick",
	Long:    `Tooka helps you automate tasks based on a cron-like schedule using shell commands and file watchers.`,
	Version: version,
}

func Execute() error {
	taskManager = core.GetManager(tasksDir)
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
