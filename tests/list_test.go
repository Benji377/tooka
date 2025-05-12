package tests

import (
	"os/exec"
	"strings"
	"testing"
)

func TestCLI_List_AllTasksEmpty(t *testing.T) {
	// Simulate running the "tooka list" command
	cmd := exec.Command("../tooka", "list")

	// Capture the output of the command
	output, err := cmd.CombinedOutput()

	if err != nil {
		t.Fatalf("Failed to run the CLI command: %v", err)
	}

	// Check if the output contains the task name or any task
	if !strings.Contains(string(output), "No tasks found") {
		t.Errorf("Expected empty tasks output, but got: %s", string(output))
	}
}

func TestCLI_List_AllTasks(t *testing.T) {
	taskFile := "test_task.json"
	taskJSON := `{
		"name": "test-task",
		"description": "A test task to run",
		"modules": []
	}`

	// Set up the task file for this test
	setup(t, taskFile, taskJSON)
	defer teardown(t, taskFile)

	// First, add the task
	cmdAdd := exec.Command("../tooka", "add", taskFile)
	runCommand(t, cmdAdd)

	// Simulate running the "tooka list" command
	cmd := exec.Command("../tooka", "list")

	// Capture the output of the command
	output, err := cmd.CombinedOutput()

	if err != nil {
		t.Fatalf("Failed to run the CLI command: %v", err)
	}

	// Check if the output contains the task name or any task
	if !strings.Contains(string(output), "test-task") {
		t.Errorf("Expected name of tasks in output, but got: %s", string(output))
	}
}

func TestCLI_List_FilteredTasks(t *testing.T) {
	taskFile := "test_task.json"
	taskJSON := `{
		"name": "test-task",
		"description": "A test task to run",
		"modules": []
	}`

	// Set up the task file for this test
	setup(t, taskFile, taskJSON)
	defer teardown(t, taskFile)

	// First, add the task
	cmdAdd := exec.Command("../tooka", "add", taskFile)
	runCommand(t, cmdAdd)

	// Simulate running the "tooka list --name=test-task" command
	cmd := exec.Command("../tooka", "list", "--name", "test")

	// Capture the output of the command
	output, err := cmd.CombinedOutput()

	if err != nil {
		t.Fatalf("Failed to run the CLI command: %v", err)
	}

	// Check if the output contains the filtered task name
	if !strings.Contains(string(output), "test-task") {
		t.Errorf("Expected filtered task in output, but got: %s", string(output))
	}
}
