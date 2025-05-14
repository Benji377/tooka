package core

import (
	"encoding/json"
	"os"
	"path/filepath"

	"github.com/Benji377/tooka/internal/shared"
)

var taskFile string

func init() {
	taskFile = shared.GetTasksFilePath()
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
			defer func() {
				if cerr := file.Close(); cerr != nil {
					panic("Failed to close task file: " + cerr.Error())
				}
			}()
			if _, werr := file.Write([]byte("[]")); werr != nil {
				panic("Failed to write to task file: " + werr.Error())
			} // Write empty JSON array
		}
	}

}

func SaveTasks(tasks []Task) error {
	data, err := json.MarshalIndent(tasks, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(taskFile, data, 0644)
}

func LoadTasks() ([]Task, error) {
	var tasks []Task
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
