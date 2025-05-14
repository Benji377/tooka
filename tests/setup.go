package tests

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// SetupTestEnv creates a temporary JSON file and returns its path
func SetupTestEnv(t *testing.T) string {
	tmpDir := t.TempDir()
	taskFilePath := filepath.Join(tmpDir, "tasks.json")

	// Write empty JSON array to initialize
	if err := os.WriteFile(taskFilePath, []byte("[]"), 0644); err != nil {
		t.Fatalf("failed to write temp task file: %v", err)
	}

	return taskFilePath
}

// RunCLI runs the `tooka` CLI with args and a custom tasks file path
func RunCLI(t *testing.T, taskFile string, args ...string) string {
	cmd := exec.Command("../tooka", args...)
	cmd.Env = append(os.Environ(), "TASKS_PATH="+taskFile)

	output, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("command failed: %v\nOutput: %s", err, string(output))
	}
	return string(output)
}

// AddTask adds a task using the CLI
func AddTask(t *testing.T, taskFile string, title string) string {
	return RunCLI(t, taskFile, "add", 
		"--title",	title,
		"--description", "Test task",
		"--due", "2025-12-31",
		"--priority", "1")
}

// RemoveTask removes a task by ID using the CLI
func RemoveTask(t *testing.T, taskFile string, id string) {
	out := RunCLI(t, taskFile, "remove", id)
	if !strings.Contains(out, "removed") {
		t.Fatalf("task removal failed: %s", out)
	}
}
