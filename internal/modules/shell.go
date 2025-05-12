package modules

import (
	"fmt"
	"os/exec"
	"time"
)

type ShellModule struct {
	Command    string
	MaxRetries int
	RetryDelay time.Duration
}

func NewShellModule(config map[string]any) (Module, error) {
	cmd, ok := config["command"].(string)
	maxRetries, okRetries := config["max_retries"].(int)
	retryDelay, okDelay := config["retry_delay"].(time.Duration)

	if !ok || cmd == "" {
		return nil, fmt.Errorf("missing or invalid 'command' in shell module config")
	}

	// If max_retries or retry_delay are not provided, set defaults
	if !okRetries {
		maxRetries = 3 // Default retries to 3
	}
	if !okDelay {
		retryDelay = 2 * time.Second // Default retry delay to 2 seconds
	}

	return &ShellModule{
		Command:    cmd,
		MaxRetries: maxRetries,
		RetryDelay: retryDelay,
	}, nil
}

func (m *ShellModule) Run() string {
	var lastErr error
	var output string

	// Retry logic
	for i := 0; i <= m.MaxRetries; i++ {
		// Run the shell command
		out, err := exec.Command("sh", "-c", m.Command).CombinedOutput()
		if err == nil {
			// If command runs successfully, return the output
			output = string(out)
			break
		}

		lastErr = err
		// Delay before the next retry
		time.Sleep(m.RetryDelay)
	}

	if lastErr != nil {
		return fmt.Sprintf("Error: Command failed after %d retries. Last error: %s", m.MaxRetries, lastErr.Error())
	}

	return output
}

func init() {
	RegisterModule(ModuleInfo{
		Name:        "shell",
		Description: "Runs a shell command with optional retries",
		ConfigHelp:  "Required: 'command' (string), Optional: 'max_retries' (int, default 3), 'retry_delay' (time.Duration, default 2s)",
		Constructor: NewShellModule,
	})
}
