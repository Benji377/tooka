package storage

import (
	"encoding/json"
	"os"
	"path/filepath"

	"github.com/Benji377/tooka/internal/task"
)

var taskFile string

func init() {
	home, err := os.UserHomeDir()
	if err != nil {
		// fallback to current directory if home directory cannot be determined
		taskFile = "tasks.json"
		return
	}
	taskFile = filepath.Join(home, ".tooka", "tasks.json")
	// Ensure the directory exists, create it if not
	if err := os.MkdirAll(filepath.Dir(taskFile), 0755); err != nil {
		panic("Failed to create task directory: " + err.Error())
	}
	// Ensure the file exists, create it if not
	if _, err := os.Stat(taskFile); err != nil {
		if os.IsNotExist(err) {
			file, err := os.Create(taskFile)
			if err != nil {
				panic("Failed to create task file: " + err.Error())
			}
			defer file.Close()
			file.Write([]byte("[]")) // Write empty JSON array
		}
	}

}

func SaveTasks(tasks []task.Task) error {
	data, err := json.MarshalIndent(tasks, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(taskFile, data, 0644)
}

func LoadTasks() ([]task.Task, error) {
	var tasks []task.Task
	data, err := os.ReadFile(taskFile)
	if err != nil {
		if os.IsNotExist(err) {
			return tasks, nil
		}
		return nil, err
	}

	if len(data) == 0 {
		// Treat empty file as no tasks
		return tasks, nil
	}

	err = json.Unmarshal(data, &tasks)
	return tasks, err
}
