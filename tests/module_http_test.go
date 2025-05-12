package tests

import (
	"os/exec"
	"strings"
	"testing"
)

func TestHTTPModule(t *testing.T) {
	taskFile := "http_task.json"
	taskJSON := `{
		"name": "http-task",
		"description": "Test HTTP GET request",
		"modules": [
			{
				"name": "http",
				"config": {
					"url": "https://httpbin.org/get",
					"method": "GET",
					"timeout": "3s"
				}
			}
		]
	}`

	setup(t, taskFile, taskJSON)
	defer teardown(t, taskFile)

	cmdAdd := exec.Command("../tooka", "add", taskFile)
	runCommand(t, cmdAdd)

	cmdRun := exec.Command("../tooka", "run", "http-task")
	output, err := cmdRun.CombinedOutput()

	if err != nil {
		t.Fatalf("Failed to run task: %v", err)
	}
	if !strings.Contains(string(output), "httpbin.org") {
		t.Errorf("Expected HTTP response output, got: %s", string(output))
	}
}
