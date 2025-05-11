package core

import (
	"encoding/json"
	"fmt"
	"os"
)

// TaskManager is responsible for managing tasks (adding, removing, listing)
type TaskManager struct {
	scheduler *TaskScheduler
}

// NewTaskManager creates a new TaskManager instance
func NewTaskManager(scheduler *TaskScheduler) *TaskManager {
	return &TaskManager{scheduler: scheduler}
}

// SaveTask saves a task to a JSON file
func (tm *TaskManager) SaveTask(task *Task, filePath string) error {
	// Convert the task to JSON format
	taskData, err := json.MarshalIndent(task, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal task to JSON: %w", err)
	}

	// Write the JSON task data to the specified file
	err = os.WriteFile(filePath, taskData, 0644)
	if err != nil {
		return fmt.Errorf("failed to save task to file: %w", err)
	}

	return nil
}

// LoadTask loads a task from a JSON file
func (tm *TaskManager) LoadTask(filePath string) (*Task, error) {
	// Read the task data from the file
	taskData, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read task file: %w", err)
	}

	// Unmarshal the JSON data into a Task object
	var task Task
	err = json.Unmarshal(taskData, &task)
	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal task data: %w", err)
	}

	return &task, nil
}

// RemoveTask removes a task from the task scheduler and from the tasks map
func (tm *TaskManager) RemoveTask(name string) error {
	// Check if the task exists
	task, exists := tm.scheduler.GetTask(name)
	if !exists {
		return fmt.Errorf("task with name '%s' not found", name)
	}

	// Stop the task from the scheduler
	tm.scheduler.cronScheduler.Remove(task.CronID)

	// Delete the task from the map
	delete(tm.scheduler.tasks, name)

	return nil
}

// ListTasks lists all the tasks currently managed by the task scheduler
func (tm *TaskManager) ListTasks() {
	tm.scheduler.ListTasks()
}
