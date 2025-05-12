package tests

import (
	"os/exec"
	"strings"
	"testing"
)

func TestCLI_Run_ShellModule(t *testing.T) {
	taskFile := "shell_task.json"
	taskJSON := `{
		"name": "shell-task",
		"description": "A task that runs a shell command",
		"modules": [
			{
				"name": "shell",
				"config": {
					"command": "echo 'hello from shell'"
				}
			}
		]
	}`

	setup(t, taskFile, taskJSON)
	defer teardown(t, taskFile)

	runCommand(t, exec.Command("../tooka", "add", taskFile))

	cmd := exec.Command("../tooka", "run", "shell-task")
	output, err := cmd.CombinedOutput()

	if err != nil {
		t.Fatalf("Run failed: %v", err)
	}
	if !strings.Contains(string(output), "hello from shell") {
		t.Errorf("Expected shell output, got: %s", output)
	}
}
