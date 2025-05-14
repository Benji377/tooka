package tests

import (
	"strings"
	"testing"
)


func TestRemoveTask(t *testing.T) {
	taskFile := SetupTestEnv(t)

	// Add, get ID, then remove
	AddTask(t, taskFile, "Temporary task")
	listOutput := RunCLI(t, taskFile, "list")
	taskID := GetTaskIDFromList(t, listOutput)

	RemoveTask(t, taskFile, taskID)

	// Verify it’s gone
	listOutputAfter := RunCLI(t, taskFile, "list")
	if strings.Contains(listOutputAfter, taskID) {
		t.Fatalf("Task still listed after removal: %s", listOutputAfter)
	}
}