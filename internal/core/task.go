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
	Modules         []ModuleConfig   `json:"modules"`
	Output          string           `json:"output,omitempty"`
	CompiledModules []modules.Module `json:"-"`
}

type ModuleConfig struct {
	Name   string         `json:"name"`
	Config map[string]any `json:"config"`
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

	for _, modCfg := range task.Modules {
		modInstance, err := modules.LoadModule(map[string]map[string]any{
			modCfg.Name: modCfg.Config,
		})
		if err != nil {
			return nil, fmt.Errorf("error loading module: %w", err)
		}
		task.CompiledModules = append(task.CompiledModules, modInstance)
	}


	return &task, nil
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
