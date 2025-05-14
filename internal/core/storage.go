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
	shared.Log.Debug().Str("task_file_path", taskFile).Msg("Task file path initialized") // Debug log for task file path

	// Ensure the directory exists, create it if not
	if err := os.MkdirAll(filepath.Dir(taskFile), 0755); err != nil {
		shared.Log.Fatal().Err(err).Msg("Failed to create task directory") // Fatal log if directory creation fails
	}

	// Ensure the file exists, create it if not
	if _, err := os.Stat(taskFile); err != nil {
		if os.IsNotExist(err) {
			file, err := os.Create(taskFile)
			if err != nil {
				shared.Log.Fatal().Err(err).Msg("Failed to create task file") // Fatal log if file creation fails
			}
			defer func() {
				if cerr := file.Close(); cerr != nil {
					shared.Log.Fatal().Err(cerr).Msg("Failed to close task file") // Fatal log if file closing fails
				}
			}()
			if _, werr := file.Write([]byte("[]")); werr != nil {
				shared.Log.Fatal().Err(werr).Msg("Failed to write to task file") // Fatal log for write errors
			} // Write empty JSON array
			shared.Log.Debug().Msg("Created new task file with empty array") // Debug log for new file creation
		}
	}
}

func SaveTasks(tasks []Task) error {
	shared.Log.Debug().Msg("Saving tasks to storage") // Debug log for task saving
	data, err := json.MarshalIndent(tasks, "", "  ")
	if err != nil {
		shared.Log.Error().Err(err).Msg("Failed to marshal tasks for saving") // Log detailed error
		return err
	}
	err = os.WriteFile(taskFile, data, 0644)
	if err != nil {
		shared.Log.Error().Err(err).Msg("Failed to write tasks to file") // Log detailed error
	}
	return err
}

func LoadTasks() ([]Task, error) {
	shared.Log.Debug().Msg("Loading tasks from storage") // Debug log for task loading
	var tasks []Task
	data, err := os.ReadFile(taskFile)
	if err != nil {
		if os.IsNotExist(err) {
			shared.Log.Debug().Msg("No task file found, returning empty task list") // Debug log if file doesn't exist
			return tasks, nil
		}
		shared.Log.Error().Err(err).Msg("Failed to read task file") // Log detailed error
		return nil, err
	}

	if len(data) == 0 {
		shared.Log.Debug().Msg("Task file is empty, returning empty task list") // Debug log for empty file
		return tasks, nil
	}

	err = json.Unmarshal(data, &tasks)
	if err != nil {
		shared.Log.Error().Err(err).Msg("Failed to unmarshal task data") // Log detailed error
	}
	return tasks, err
}
