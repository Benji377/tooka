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
	Log.Info().Msgf("[TASK] Loading task from file: %s", path)
	data, err := os.ReadFile(path)
	if err != nil {
		Log.Error().Msgf("[TASK] Failed to read task file: %s", path)
		return nil, fmt.Errorf("failed to read task file: %w", err)
	}

	var task Task
	if err := json.Unmarshal(data, &task); err != nil {
		Log.Error().Msgf("[TASK] Failed to unmarshal task JSON: %s", path)
		return nil, fmt.Errorf("invalid JSON: %w", err)
	}

	Log.Debug().Msgf("[TASK] Task loaded: %s", task.Name)
	for _, modCfg := range task.Modules {
		Log.Debug().Msgf("[TASK] Compiling module: %s", modCfg.Name)
		modInstance, err := modules.LoadModule(map[string]map[string]any{
			modCfg.Name: modCfg.Config,
		})
		if err != nil {
			Log.Error().Msgf("[TASK] Failed to load module: %s", modCfg.Name)
			return nil, fmt.Errorf("error loading module: %w", err)
		}
		task.CompiledModules = append(task.CompiledModules, modInstance)
	}
	Log.Info().Msgf("[TASK] Compiled %d modules for task: %s", len(task.CompiledModules), task.Name)

	return &task, nil
}

// Run executes the task's module chain and optionally writes output
func (t *Task) Run() {
	_ = t.RunLive("", false)
}

// RunLive executes the task and optionally overrides the output path
func (t *Task) RunLive(outputOverride string, quiet bool) error {
	Log.Info().Msgf("[TASK] Running task: %s", t.Name)
	var output string
	for _, mod := range t.CompiledModules {
		Log.Debug().Msgf("[TASK] Running module: %s", mod)
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
			Log.Error().Msgf("[TASK] Failed to write output to file: %s", finalOutputPath)
			return fmt.Errorf("error writing output to %s: %v", finalOutputPath, err)
		}
	} else if !quiet {
		fmt.Print(output)
	}
	return nil
}
