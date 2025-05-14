package shared

import (
	"os"
	"path/filepath"
)

var Version string = "1.0.0"

func GetTasksFilePath() string {
	home, err := os.UserHomeDir()
	if err == nil && home != "" {
		return filepath.Join(home, ".tooka", "tasks.json")
	}
	// fallback to current directory if home directory cannot be determined
	return "tasks.json"
}

func GetLogsDir() string {
	home, err := os.UserHomeDir()
	if err == nil && home != "" {
		return filepath.Join(home, ".tooka", "logs")
	}
	// fallback to current directory if home directory cannot be determined
	return "logs"
}
