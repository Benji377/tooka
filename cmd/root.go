package cmd

import (
	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:     "tooka",
	Short:   "Tooka is a task management tool",
	Long:    "Tooka is a task management tool that helps you manage your tasks efficiently.",
	Version: "0.1.0",
}

func Execute() error {
	if err := rootCmd.Execute(); err != nil {
		return err
	}
	return nil
}

func init() {
	rootCmd.AddCommand(addCmd)
	rootCmd.AddCommand(removeCmd)
	rootCmd.AddCommand(listCmd)
	rootCmd.AddCommand(toggleCmd)
	rootCmd.AddCommand(editCmd)
	rootCmd.AddCommand(infoCmd)
}
