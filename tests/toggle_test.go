package tests

import (
	"strings"
	"testing"
)

func TestToggleComplete(t *testing.T) {
	taskFile := SetupTestEnv(t)

	AddTask(t, taskFile, "Finish toggling")
	listOutput := RunCLI(t, taskFile, "list")
	taskID := GetTaskIDFromList(t, listOutput)

	toggleOutput := RunCLI(t, taskFile, "toggle", taskID)
	if !strings.Contains(toggleOutput, "Toggled") {
		t.Fatalf("Toggle failed: %s", toggleOutput)
	}

	infoOutput := RunCLI(t, taskFile, "info", taskID)
	if !strings.Contains(infoOutput, "Completed: true") {
		t.Fatalf("Toggle didn't mark task as completed: %s", infoOutput)
	}

	RemoveTask(t, taskFile, taskID)
}