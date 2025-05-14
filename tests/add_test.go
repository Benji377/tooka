package tests

import (
	"strings"
	"testing"
)

func TestAddTask(t *testing.T) {
	taskFile := SetupTestEnv(t)

	addOutput := AddTask(t, taskFile, "Read a book")
	if !strings.Contains(addOutput, "added") {
		t.Fatalf("Add failed: %s", addOutput)
	}

	listOutput := RunCLI(t, taskFile, "list")
	taskID := GetTaskIDFromList(t, listOutput)
	RemoveTask(t, taskFile, taskID)
}
