package core

import (
	"fmt"
	"log"
	"os"

	"github.com/robfig/cron/v3"
)

// TaskScheduler holds the cron scheduler and task management
type TaskScheduler struct {
	cronScheduler *cron.Cron
	tasks         map[string]*Task
}

// NewTaskScheduler creates a new TaskScheduler instance
func NewTaskScheduler() *TaskScheduler {
	return &TaskScheduler{
		cronScheduler: cron.New(cron.WithLogger(cron.VerbosePrintfLogger(log.New(os.Stdout, "cron: ", log.LstdFlags)))),
		tasks:         make(map[string]*Task),
	}
}

// AddTask adds a task to the scheduler using its cron schedule
func (s *TaskScheduler) AddTask(task *Task) (cron.EntryID, error) {
	// Parse cron schedule
	_, err := cron.ParseStandard(task.Schedule)
	if err != nil {
		return 0, fmt.Errorf("invalid cron schedule: %w", err)
	}

	// Add task to scheduler
	id, err := s.cronScheduler.AddFunc(task.Schedule, func() {
		task.Run()
	})

	if err != nil {
		return 0, fmt.Errorf("failed to schedule task %s: %w", task.Name, err)
	}

	task.CronID = id

	// Store task for reference (you could add more sophisticated state tracking)
	s.tasks[task.Name] = task

	return id, nil
}

// Start starts the cron scheduler
func (s *TaskScheduler) Start() {
	s.cronScheduler.Start()
}

// Stop stops the cron scheduler
func (s *TaskScheduler) Stop() {
	s.cronScheduler.Stop()
}

// GetTask gets a task by name
func (s *TaskScheduler) GetTask(name string) (*Task, bool) {
	task, exists := s.tasks[name]
	return task, exists
}

// ListTasks lists all tasks currently scheduled
func (s *TaskScheduler) ListTasks() {
	fmt.Println("Scheduled Tasks:")
	for _, task := range s.tasks {
		fmt.Printf("Task Name: %s, Schedule: %s\n", task.Name, task.Schedule)
	}
}
