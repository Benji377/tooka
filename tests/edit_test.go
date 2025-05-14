package tests

import (
	"strings"
	"testing"
)

func TestEditTask(t *testing.T) {
	taskFile := SetupTestEnv(t)

	AddTask(t, taskFile, "Old Title")

	taskID := "0"

	editOutput := RunCLI(t, taskFile, "edit", taskID,
		"--title", "New Title",
		"--description", "Updated description",
		"--priority", "2")

	if !strings.Contains(editOutput, "updated") {
		t.Fatalf("Edit failed: %s", editOutput)
	}

	infoOutput := RunCLI(t, taskFile, "info", taskID)
	if !strings.Contains(infoOutput, "New Title") || !strings.Contains(infoOutput, "Severe") {
		t.Fatalf("Edit changes not reflected:\n%s", infoOutput)
	}

	RemoveTask(t, taskFile, taskID)
}
