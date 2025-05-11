package core

import (
	"encoding/json"
	"fmt"
	"os"
	"time"

	"github.com/Benji377/tooka/internal/modules"
	"github.com/robfig/cron/v3"
)

// Task defines the structure of a Tooka task loaded from JSON
type Task struct {
	Name     string           `json:"name"`
	Schedule string           `json:"schedule"`
	Modules  []map[string]any `json:"modules"`
	Output   string           `json:"output,omitempty"`

	compiledModules []modules.Module // Holds loaded module instances
	CronID          cron.EntryID
}

// LoadTaskFromFile reads a JSON file and parses it into a Task object
func LoadTaskFromFile(path string) (*Task, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read task file: %w", err)
	}

	var task Task
	if err := json.Unmarshal(data, &task); err != nil {
		return nil, fmt.Errorf("invalid JSON: %w", err)
	}

	// Load module implementations
	for _, rawMod := range task.Modules {
		modInstance, err := modules.LoadModule(mapToStringMap(rawMod))
		if err != nil {
			return nil, fmt.Errorf("error loading module: %w", err)
		}
		task.compiledModules = append(task.compiledModules, modInstance)
	}

	return &task, nil
}

func mapToStringMap(raw map[string]interface{}) map[string]map[string]interface{} {
	converted := make(map[string]map[string]interface{})
	for key, value := range raw {
		if innerMap, ok := value.(map[string]interface{}); ok {
			converted[key] = innerMap
		}
	}
	return converted
}

// Run executes the task's module chain and optionally writes output
func (t *Task) Run() {
	var output string
	for _, mod := range t.compiledModules {
		result := mod.Run()
		output += fmt.Sprintf("[%s] %s\n", time.Now().Format(time.RFC3339), result)
	}

	// Log output if set
	if t.Output != "" {
		err := os.WriteFile(t.Output, []byte(output), 0644)
		if err != nil {
			fmt.Printf("Error writing output to %s: %v\n", t.Output, err)
		}
	} else {
		fmt.Print(output)
	}
}
