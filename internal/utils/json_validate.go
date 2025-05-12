package utils

import (
	"fmt"
	"os"

	"github.com/tidwall/gjson"
)

// ValidateTaskJSON uses gjson to ensure the structure of the task JSON file is valid
func ValidateTaskJSON(path string) error {
	data, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("cannot read file: %w", err)
	}

	content := string(data)

	requiredFields := []string{"name", "description", "modules"}
	for _, field := range requiredFields {
		if !gjson.Get(content, field).Exists() {
			return fmt.Errorf("missing required field: '%s'", field)
		}
	}

	// Validate modules array
	modules := gjson.Get(content, "modules")
	if !modules.IsArray() {
		return fmt.Errorf("'modules' must be an array")
	}

	for i, mod := range modules.Array() {
		if !mod.IsObject() {
			return fmt.Errorf("module at index %d must be an object", i)
		}
		// Must have exactly one key (e.g., "shell" or "file")
		if len(mod.Map()) != 1 {
			return fmt.Errorf("module at index %d must have exactly one key (e.g., 'shell' or 'file')", i)
		}
	}

	return nil
}
