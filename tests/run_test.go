package tests

import (
	"strings"
	"os/exec"
	"testing"
)

func TestCLI_Run_Task(t *testing.T) {
	taskFile := "test_task.json"
	taskJSON := `{
		"name": "test-task",
		"description": "A test task to run",
		"modules": []
	}`

	// Set up the task file for this test
	setup(t, taskFile, taskJSON)

	// First, add the task
	cmdAdd := exec.Command("../tooka", "add", taskFile)
	runCommand(t, cmdAdd)

	// Now, run the task
	cmdRun := exec.Command("../tooka", "run", "test-task")
	output := runCommand(t, cmdRun)

	// Clean up the task file after the test
	teardown(t, taskFile)

	// Check if the output contains the expected running message
	if !strings.Contains(output, "Success") {
		t.Errorf("Expected task running message, but got: %s", output)
	}
}


func TestCLI_Run_Quiet(t *testing.T) {
	taskFile := "test_task.json"
	taskJSON := `{
		"name": "test-task",
		"description": "A test task to run",
		"modules": []
	}`

	// Set up the task file for this test
	setup(t, taskFile, taskJSON)

	cmdAdd := exec.Command("../tooka", "add", taskFile)
	runCommand(t, cmdAdd)

	// Now, run the task
	cmdRun := exec.Command("../tooka", "run", "test-task", "--quiet")
	output := runCommand(t, cmdRun)

	// Clean up the task file after the test
	teardown(t, taskFile)

	// Ensure no output is printed due to quiet flag
	if len(output) > 0 {
		t.Errorf("Expected no output in quiet mode, but got: %s", string(output))
	}
}
