package tests

import (
	"strings"
	"testing"
)

func TestInfoCommand(t *testing.T) {
	taskFile := SetupTestEnv(t)

	AddTask(t, taskFile, "Task with info")
	taskID := "0"

	infoOutput := RunCLI(t, taskFile, "info", taskID)
	if !strings.Contains(infoOutput, "Task with info") {
		t.Fatalf("Info output missing title: %s", infoOutput)
	}
	if !strings.Contains(infoOutput, "Description") {
		t.Fatalf("Info output missing details: %s", infoOutput)
	}

	RemoveTask(t, taskFile, taskID)
}
