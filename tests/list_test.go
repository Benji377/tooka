package tests

import (
	"strings"
	"testing"
)


func TestListTasks(t *testing.T) {
	taskFile := SetupTestEnv(t)

	// Add task
	AddTask(t, taskFile, "Write test coverage")
	listOutput := RunCLI(t, taskFile, "list")

	if !strings.Contains(listOutput, "Write test coverage") {
		t.Fatalf("List does not show added task:\n%s", listOutput)
	}

	// Cleanup
	RemoveTask(t, taskFile, "0")
}