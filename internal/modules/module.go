package modules

import (
	"fmt"
)

// Module is the runtime behavior
type Module interface {
	Run() string
}

// ModuleInfo contains metadata and constructor
type ModuleInfo struct {
	Name        string
	Description string
	ConfigHelp  string
	Constructor func(config map[string]any) (Module, error)
}

// Global registry of all known modules
var moduleRegistry = map[string]ModuleInfo{}

// RegisterModule adds a module to the registry
func RegisterModule(info ModuleInfo) {
	if _, exists := moduleRegistry[info.Name]; exists {
		panic("module already registered: " + info.Name)
	}
	moduleRegistry[info.Name] = info
}

// GetRegisteredModules returns all registered module names
func GetRegisteredModules() []string {
	keys := make([]string, 0, len(moduleRegistry))
	for name := range moduleRegistry {
		keys = append(keys, name)
	}
	return keys
}

// GetModuleInfo returns info for a specific module
func GetModuleInfo(name string) (ModuleInfo, bool) {
	info, ok := moduleRegistry[name]
	return info, ok
}

// LoadModule uses registry instead of switch
func LoadModule(moduleMap map[string]map[string]any) (Module, error) {
	for name, config := range moduleMap {
		if info, ok := moduleRegistry[name]; ok {
			return info.Constructor(config)
		}
		return nil, fmt.Errorf("unknown module type: %s", name)
	}
	return nil, fmt.Errorf("no module type found")
}
