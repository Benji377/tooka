package tests

import (
	"os/exec"
	"strings"
	"testing"
)

func TestCLI_Run_CombinedModules(t *testing.T) {
	taskFile := "combined_task.json"
	taskJSON := `{
		"name": "combined-task",
		"description": "Shell and HTTP combo",
		"modules": [
			{
				"name": "shell",
				"config": {
					"command": "echo 'from shell'"
				}
			},
			{
				"name": "http",
				"config": {
					"url": "https://httpbin.org/get",
					"method": "GET",
					"return": "status"
				}
			}
		]
	}`

	setup(t, taskFile, taskJSON)
	defer teardown(t, taskFile)

	runCommand(t, exec.Command("../tooka", "add", taskFile))

	cmd := exec.Command("../tooka", "run", "combined-task")
	output, err := cmd.CombinedOutput()

	if err != nil {
		t.Fatalf("Run failed: %v", err)
	}
	if !strings.Contains(string(output), "from shell") || !strings.Contains(string(output), "200") {
		t.Errorf("Expected both shell and HTTP outputs, got: %s", output)
	}
}
