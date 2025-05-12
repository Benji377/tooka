package tests

import (
	"os"
	"os/exec"
	"strings"
	"testing"
)

// Helper function to create a temporary task file
func setup(t *testing.T, taskFileName string, taskJSON string) {
	// Write the JSON task file for the test
	err := os.WriteFile(taskFileName, []byte(taskJSON), 0644)
	if err != nil {
		t.Fatalf("Failed to create task file: %v", err)
	}

	// Automatically clean up the task file after the test
	t.Cleanup(func() {
		err := os.Remove(taskFileName)
		if err != nil {
			t.Errorf("Failed to clean up task file: %v", err)
		}
	})
}

// Helper function to run a command and check its output
func runCommand(t *testing.T, cmd *exec.Cmd) string {
	output, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("Failed to run command: %v", err)
	}
	return string(output)
}

func teardown(t *testing.T, taskFileName string) {
	// Removes the task file after the test

	// Truncate .json from the task file name and transform _ to -
	taskFileName = strings.ReplaceAll(taskFileName, "_", "-")
	taskFileName = strings.TrimSuffix(taskFileName, ".json")

	cmdRemove := exec.Command("../tooka", "remove", taskFileName)
	output := runCommand(t, cmdRemove)
	if !strings.Contains(output, "removed") {
		t.Errorf("Expected task to be removed, but got: %s", output)
	}
}
