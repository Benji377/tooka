package tests

import (
	"os"
	"os/exec"
	"testing"
	"strings"
)

func TestCLI_Add_ValidTask(t *testing.T) {
	taskFile := "test_task.json"
	taskJSON := `{
		"name": "test-task",
		"description": "A test task",
		"modules": []
	}`

	// Set up the task file for this test
	setup(t, taskFile, taskJSON)
	defer teardown(t, taskFile)

	// Run the "add" command
	cmd := exec.Command("../tooka", "add", taskFile)
	output := runCommand(t, cmd)

	// Check if the output contains the expected message
	if !strings.Contains(output, "loaded successfully") {
		t.Errorf("Expected task to be loaded, but got: %s", output)
	}
}


func TestCLI_Add_InvalidTask(t *testing.T) {
	// Create a temporary invalid task file
	taskFile := "invalid_task.json"
	taskJSON := `{
		"name": "invalid-task",
		"description": "An invalid test task",
		"modules": ["invalid-module"]
	}`
	if err := os.WriteFile(taskFile, []byte(taskJSON), 0644); err != nil {
		t.Fatalf("Failed to write task file: %v", err)
	}
	defer func() {
		if err := os.Remove(taskFile); err != nil {
			t.Errorf("Failed to remove task file: %v", err)
		}
	}()

	// Simulate running the "tooka add invalid_task.json" command
	cmd := exec.Command("../tooka", "add", taskFile)

	// Capture the output of the command
	output, err := cmd.CombinedOutput()

	if err != nil {
		t.Fatalf("Failed to run the CLI command: %v", err)
	}

	// Check if the output contains the expected error message
	if !strings.Contains(string(output), "Error") {
		t.Errorf("Expected error loading module, but got: %s", string(output))
	}
}
