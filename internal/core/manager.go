package core

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"slices"
	"sync"
)

// TaskManager manages all tasks in memory and persists them to disk.
type TaskManager struct {
	tasks         map[string]*Task
	backupPath    string
	loaded        bool
	mu            sync.Mutex
	TaskScheduler *TaskScheduler
}

var (
	instance *TaskManager
	once     sync.Once
)

// GetManager returns the singleton TaskManager.
// It also attempts to load tasks from the backup directory on first use.
func GetManager(backupDir string) *TaskManager {
	once.Do(func() {
		instance = &TaskManager{
			tasks:      make(map[string]*Task),
			backupPath: backupDir,
		}
		instance.TaskScheduler = NewTaskScheduler(instance)
		if err := instance.LoadFromBackup(); err != nil {
			fmt.Printf("Warning: failed to load tasks from backup: %v\n", err)
		}
		instance.TaskScheduler.Start()
	})
	return instance
}

// LoadFromBackup loads all task JSON files from the backup directory.
func (m *TaskManager) LoadFromBackup() error {
	m.mu.Lock()
	defer m.mu.Unlock()

	if m.loaded {
		return nil
	}

	files, err := os.ReadDir(m.backupPath)
	if err != nil {
		return fmt.Errorf("failed to read backup directory: %w", err)
	}

	for _, file := range files {
		if filepath.Ext(file.Name()) != ".json" {
			continue
		}

		fullPath := filepath.Join(m.backupPath, file.Name())
		task, err := LoadTaskFromFile(fullPath)
		if err != nil {
			fmt.Printf("Skipping invalid task file: %s (%v)\n", file.Name(), err)
			continue
		}
		m.tasks[task.Name] = task
	}

	m.loaded = true
	return nil
}

// AddTask adds a new task to memory and writes it to disk.
func (m *TaskManager) AddTask(task *Task) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.tasks[task.Name] = task

	data, err := json.MarshalIndent(task, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal task: %w", err)
	}

	if err := os.MkdirAll(m.backupPath, 0755); err != nil {
		return fmt.Errorf("failed to ensure backup directory: %w", err)
	}

	path := filepath.Join(m.backupPath, task.Name+".json")
	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("failed to write task file: %w", err)
	}

	return nil
}

// RemoveTask deletes a task from memory and its JSON file from disk.
func (m *TaskManager) RemoveTask(name string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	delete(m.tasks, name)

	path := filepath.Join(m.backupPath, name+".json")
	if err := os.Remove(path); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to delete task file: %w", err)
	}

	return nil
}

// GetTask retrieves a task by name.
func (m *TaskManager) GetTask(name string) (*Task, bool) {
	m.mu.Lock()
	defer m.mu.Unlock()

	task, ok := m.tasks[name]
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
	return tasks
}

// ToggleTask enables or disables a task and schedules or unschedules it.
func (m *TaskManager) ToggleTask(name string, enable bool) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	task, ok := m.tasks[name]
	if !ok {
		return fmt.Errorf("task not found: %s", name)
	}

	// Toggle task state
	task.Enabled = enable

	// If enabled, schedule it, else remove it
	if enable {
		// Schedule the task
		m.TaskScheduler.scheduleTask(task)
	} else {
		// Unschedule the task
		for _, job := range m.TaskScheduler.Scheduler.Jobs() {
			if slices.Contains(job.Tags(), task.Name) {
				m.TaskScheduler.Scheduler.RemoveByReference(job)
			}
		}
	}

	// Save the task state back to the file
	data, err := json.MarshalIndent(task, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal task: %w", err)
	}

	path := filepath.Join(m.backupPath, name+".json")
	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("failed to write task file: %w", err)
	}

	return nil
}

// Save a task to disk
func (m *TaskManager) SaveTask(task *Task) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	data, err := json.MarshalIndent(task, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal task: %w", err)
	}

	path := filepath.Join(m.backupPath, task.Name+".json")
	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("failed to write task file: %w", err)
	}

	return nil
}
