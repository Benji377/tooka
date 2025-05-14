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
	RemoveTask(t, taskFile, "0")
}
