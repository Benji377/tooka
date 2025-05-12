package tests

import (
	"os/exec"
	"strings"
	"testing"
)

func TestCLI_Remove_Task(t *testing.T) {
	taskFile := "test_task_to_remove.json"
	taskJSON := `{
		"name": "test-task-to-remove",
		"description": "A test task to remove",
		"modules": []
	}`

	// Set up the task file for this test
	setup(t, taskFile, taskJSON)

	// First, add the task
	cmdAdd := exec.Command("../tooka", "add", taskFile)
	runCommand(t, cmdAdd)

	// Now, run the "remove" command
	cmdRemove := exec.Command("../tooka", "remove", "test-task-to-remove")
	output := runCommand(t, cmdRemove)

	// Check if the task was removed successfully
	if !strings.Contains(output, "removed") {
		t.Errorf("Expected task removal confirmation, but got: %s", output)
	}
}

func TestCLI_Remove_NonExistentTask(t *testing.T) {
	// Attempt to remove a non-existent task
	cmd := exec.Command("../tooka", "remove", "non-existent-task")
	output := runCommand(t, cmd)

	// Check if the error message is as expected
	if !strings.Contains(output, "not exist") {
		t.Errorf("Expected task not found message, but got: %s", output)
	}
}
