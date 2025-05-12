package modules

import (
	"fmt"
	"os"
	"os/exec"
	"sync"
	"time"
)

type FileModule struct {
	FilePath     string
	Command      string
	LastModified time.Time
	Mutex        sync.Mutex
}

func NewFileModule(config map[string]any) (Module, error) {
	path, ok1 := config["file"].(string)
	cmd, ok2 := config["on-change"].(string)

	if !ok1 || !ok2 || path == "" || cmd == "" {
		return nil, fmt.Errorf("both 'file' and 'on-change' must be provided")
	}

	info, err := os.Stat(path)
	if err != nil {
		return nil, fmt.Errorf("could not stat file: %w", err)
	}

	return &FileModule{
		FilePath:     path,
		Command:      cmd,
		LastModified: info.ModTime(),
	}, nil
}

func (m *FileModule) Run() string {
	m.Mutex.Lock()
	defer m.Mutex.Unlock()

	info, err := os.Stat(m.FilePath)
	if err != nil {
		return fmt.Sprintf("Error: could not stat file: %s", err.Error())
	}

	if info.ModTime() != m.LastModified {
		m.LastModified = info.ModTime()
		out, err := exec.Command("sh", "-c", m.Command).CombinedOutput()
		if err != nil {
			return fmt.Sprintf("Error: %s", err.Error())
		}
		return string(out)
	}

	return "No changes detected."
}

func init() {
	RegisterModule(ModuleInfo{
		Name:        "file",
		Description: "Executes a command when a file changes",
		ConfigHelp:  "Required: 'file' (string), 'on-change' (string)",
		Constructor: NewFileModule,
	})
}
