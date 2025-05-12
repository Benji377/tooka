package core

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
)

// TaskManager manages all tasks in memory and persists them to disk.
type TaskManager struct {
	tasks       map[string]*Task
	tasksFolder string
	loaded      bool
	mu          sync.Mutex
}

var (
	instance *TaskManager
	once     sync.Once
)

// GetManager returns the singleton TaskManager.
// It also attempts to load tasks from the backup directory on first use.
func GetManager(backupDir string) *TaskManager {
	once.Do(func() {
		// If the backup directory does not exist, create it.
		if _, err := os.Stat(backupDir); os.IsNotExist(err) {
			if err := os.MkdirAll(backupDir, 0755); err != nil {
				Log.Err(err).Msg("[MANAGER] failed to create backup directory: " + err.Error())
				fmt.Printf("Warning: failed to create backup directory: %v\n", err)
			} else {
				Log.Info().Msgf("[MANAGER] created backup directory: %s", backupDir)
			}
		} else {
			Log.Debug().Msgf("[MANAGER] backup directory exists: %s", backupDir)
		}
		// Initialize the TaskManager instance.

		instance = &TaskManager{
			tasks:       make(map[string]*Task),
			tasksFolder: backupDir,
		}
		if err := instance.LoadFromBackup(); err != nil {
			Log.Warn().Msgf("[MANAGER] failed to load tasks from backup: %v", err)
			fmt.Printf("Warning: failed to load tasks from backup: %v\n", err)
		} else {
			Log.Info().Msg("[MANAGER] loaded tasks from backup")
		}
	})
	return instance
}

// LoadFromBackup loads all task JSON files from the backup directory.
func (m *TaskManager) LoadFromBackup() error {
	m.mu.Lock()
	defer m.mu.Unlock()

	if m.loaded {
		Log.Debug().Msg("[MANAGER] tasks already loaded from backup")
		return nil
	}

	files, err := os.ReadDir(m.tasksFolder)
	if err != nil {
		Log.Error().Msgf("[MANAGER] failed to read backup directory: %v", err)
		return fmt.Errorf("failed to read backup directory: %w", err)
	}

	for _, file := range files {
		if filepath.Ext(file.Name()) != ".json" {
			continue
		}

		fullPath := filepath.Join(m.tasksFolder, file.Name())
		task, err := LoadTaskFromFile(fullPath)
		if err != nil {
			Log.Warn().Msgf("[MANAGER] skipping invalid task file: %s (%v)", file.Name(), err)
			fmt.Printf("Skipping invalid task file: %s (%v)\n", file.Name(), err)
			continue
		}
		m.tasks[task.Name] = task
		Log.Info().Msgf("[MANAGER] loaded task: %s", task.Name)
	}

	m.loaded = true
	Log.Info().Msg("[MANAGER] finished loading tasks from backup")
	return nil
}

// AddTask adds a new task to memory and writes it to disk.
func (m *TaskManager) AddTask(task *Task) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.tasks[task.Name] = task
	Log.Info().Msgf("[MANAGER] added task to memory: %s", task.Name)

	data, err := json.MarshalIndent(task, "", "  ")
	if err != nil {
		Log.Error().Msgf("[MANAGER] failed to marshal task: %v", err)
		return fmt.Errorf("failed to marshal task: %w", err)
	}

	if err := os.MkdirAll(m.tasksFolder, 0755); err != nil {
		Log.Error().Msgf("[MANAGER] failed to ensure backup directory: %v", err)
		return fmt.Errorf("failed to ensure backup directory: %w", err)
	}

	path := filepath.Join(m.tasksFolder, task.Name+".json")
	if err := os.WriteFile(path, data, 0644); err != nil {
		Log.Error().Msgf("[MANAGER] failed to write task file: %v", err)
		return fmt.Errorf("failed to write task file: %w", err)
	}

	Log.Info().Msgf("[MANAGER] saved task to disk: %s", task.Name)
	return nil
}

// RemoveTask deletes a task from memory and its JSON file from disk.
func (m *TaskManager) RemoveTask(name string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	if _, exists := m.tasks[name]; !exists {
		Log.Warn().Msgf("[MANAGER] tried to remove non-existent task: %s", name)
		return fmt.Errorf("task %q does not exist", name)
	}

	delete(m.tasks, name)
	Log.Info().Msgf("[MANAGER] removed task from memory: %s", name)

	path := filepath.Join(m.tasksFolder, name+".json")
	if err := os.Remove(path); err != nil && !os.IsNotExist(err) {
		Log.Error().Msgf("[MANAGER] failed to delete task file: %v", err)
		return fmt.Errorf("failed to delete task file: %w", err)
	}

	Log.Info().Msgf("[MANAGER] deleted task file from disk: %s", name)
	return nil
}

// GetTask retrieves a task by name.
func (m *TaskManager) GetTask(name string) (*Task, bool) {
	m.mu.Lock()
	defer m.mu.Unlock()

	task, ok := m.tasks[name]
	if ok {
		Log.Debug().Msgf("[MANAGER] retrieved task: %s", name)
	} else {
		Log.Debug().Msgf("[MANAGER] task not found: %s", name)
	}
	return task, ok
}

// ListTasks returns all currently loaded tasks.
func (m *TaskManager) ListTasks() []*Task {
	m.mu.Lock()
	defer m.mu.Unlock()

	tasks := make([]*Task, 0, len(m.tasks))
	for _, t := range m.tasks {
		tasks = append(tasks, t)
	}
	Log.Debug().Msgf("[MANAGER] listed all tasks (%d)", len(tasks))
	return tasks
}

// Save a task to disk
func (m *TaskManager) SaveTask(task *Task) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	data, err := json.MarshalIndent(task, "", "  ")
	if err != nil {
		Log.Err(err).Msgf("[MANAGER] failed to marshal task: %v", err)
		return fmt.Errorf("failed to marshal task: %w", err)
	}

	path := filepath.Join(m.tasksFolder, task.Name+".json")
	if err := os.WriteFile(path, data, 0644); err != nil {
		Log.Err(err).Msgf("[MANAGER] failed to write task file: %v", err)
		return fmt.Errorf("failed to write task file: %w", err)
	}

	Log.Info().Msgf("[MANAGER] saved task to disk: %s", task.Name)
	return nil
}
