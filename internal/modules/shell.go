package modules

import (
	"fmt"
	"os/exec"
)

type ShellModule struct {
	Command string
}

func NewShellModule(config map[string]any) (*ShellModule, error) {
	cmd, ok := config["command"].(string)
	if !ok || cmd == "" {
		return nil, fmt.Errorf("missing or invalid 'command' in shell module config")
	}
	return &ShellModule{Command: cmd}, nil
}

func (m *ShellModule) Run() string {
	out, err := exec.Command("sh", "-c", m.Command).CombinedOutput()
	if err != nil {
		return fmt.Sprintf("Error: %s", err.Error())
	}
	return string(out)
}
