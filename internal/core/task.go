package core

import (
	"encoding/json"
	"fmt"
	"os"
	"time"

	"github.com/Benji377/tooka/internal/modules"
)

// Task defines the structure of a Tooka task loaded from JSON
type Task struct {
	Name            string           `json:"name"`
	Description     string           `json:"description"`
	Modules         []map[string]any `json:"modules"`
	Output          string           `json:"output,omitempty"`
	CompiledModules []modules.Module // Holds loaded module instances
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
		task.CompiledModules = append(task.CompiledModules, modInstance)
	}

	return &task, nil
}

func mapToStringMap(raw map[string]any) map[string]map[string]any {
	converted := make(map[string]map[string]any)
	for key, value := range raw {
		if innerMap, ok := value.(map[string]any); ok {
			converted[key] = innerMap
		}
	}
	return converted
}

// Run executes the task's module chain and optionally writes output
func (t *Task) Run() {
	_ = t.RunLive("", false)
}

// RunLive executes the task and optionally overrides the output path
func (t *Task) RunLive(outputOverride string, quiet bool) error {
	var output string
	for _, mod := range t.CompiledModules {
		result := mod.Run()
		output += fmt.Sprintf("[%s] %s\n", time.Now().Format(time.RFC3339), result)
	}

	finalOutputPath := t.Output
	if outputOverride != "" {
		finalOutputPath = outputOverride
	}

	if finalOutputPath != "" {
		err := os.WriteFile(finalOutputPath, []byte(output), 0644)
		if err != nil {
			return fmt.Errorf("error writing output to %s: %v", finalOutputPath, err)
		}
	} else if !quiet {
		fmt.Print(output)
	}
	return nil
}
