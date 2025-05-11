package modules

import (
	"fmt"
)

type Module interface {
	Run() string
}

// Factory function to load a module from JSON-style input
func LoadModule(moduleMap map[string]map[string]any) (Module, error) {
	for name, config := range moduleMap {
		switch name {
		case "shell":
			return NewShellModule(config)
		case "file":
			return NewFileModule(config)
		default:
			return nil, fmt.Errorf("unknown module type: %s", name)
		}
	}
	return nil, fmt.Errorf("no module type found")
}
