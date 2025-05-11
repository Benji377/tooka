package core

import (
	"log"
	"time"

	"github.com/go-co-op/gocron"
)

// TaskScheduler holds the scheduler and manages task scheduling.
type TaskScheduler struct {
	Scheduler   *gocron.Scheduler
	taskManager *TaskManager
	Jobs        map[string]*gocron.Job // Map to store tasks by name to their corresponding job
}

// NewTaskScheduler creates a new TaskScheduler instance.
func NewTaskScheduler(taskManager *TaskManager) *TaskScheduler {
	scheduler := gocron.NewScheduler(time.UTC)

	// Create and return the scheduler
	return &TaskScheduler{
		Scheduler:   scheduler,
		taskManager: taskManager,
		Jobs:        make(map[string]*gocron.Job), // Initialize the jobs map
	}
}

// Start starts the task scheduler.
func (ts *TaskScheduler) Start() {
	// Periodically check for new tasks or task state changes
	go ts.checkTasks()

	// Start the scheduler
	ts.Scheduler.StartAsync()
}

// Stop stops the task scheduler.
func (ts *TaskScheduler) Stop() {
	ts.Scheduler.Stop()
}

// checkTasks checks the task manager's tasks and schedules them based on their state.
func (ts *TaskScheduler) checkTasks() {
	for {
		// Iterate through all tasks
		tasks := ts.taskManager.ListTasks()
		for _, task := range tasks {
			if task.Enabled {
				// If the task is enabled and not scheduled, add it to the scheduler
				if _, exists := ts.Jobs[task.Name]; !exists {
					ts.scheduleTask(task)
				}
			} else {
				// If the task is disabled, remove it from the scheduler if it exists
				if job, exists := ts.Jobs[task.Name]; exists {
					ts.Scheduler.RemoveByReference(job)
					delete(ts.Jobs, task.Name)
				}
			}
		}

		// Wait for a bit before checking again (can adjust the interval as needed)
		time.Sleep(30 * time.Second)
	}
}

// scheduleTask schedules a task based on its cron schedule.
func (ts *TaskScheduler) scheduleTask(task *Task) {
	job, err := ts.Scheduler.Cron(task.Schedule).Do(func() {
		// Call the Run method on the task when it's triggered
		task.Run()
	})

	if err != nil {
		log.Printf("Error scheduling task %s: %v\n", task.Name, err)
		return
	}

	// Store the job in the jobs map
	ts.Jobs[task.Name] = job

	log.Printf("Task '%s' scheduled with cron expression: %s\n", task.Name, task.Schedule)
}
