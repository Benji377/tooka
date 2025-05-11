package tests

import (
	"os/exec"
	"testing"
	"strings"
)

func TestCLI_Version(t *testing.T) {
	// Simulate running the "tooka --version" command
	cmd := exec.Command("../tooka", "--version")

	// Capture the output of the command
	output, err := cmd.CombinedOutput()

	if err != nil {
		t.Fatalf("Failed to run the CLI command: %v", err)
	}

	// Check if the output contains the expected version string
	if !strings.Contains(string(output), "Tooka version") {
		t.Errorf("Expected version in output, but got: %s", string(output))
	}
}
