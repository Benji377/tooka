package core

import (
	"errors"
	"time"

	"github.com/Benji377/tooka/internal/shared"
)

type TaskManager struct {
	tasks  []Task
	nextID int
}

func NewTaskManager() (*TaskManager, error) {
	shared.Log.Debug().Msg("Initializing new TaskManager") // Debug log
	tasks, err := LoadTasks()
	if err != nil {
		shared.Log.Error().Err(err).Msg("Failed to load tasks from storage") // Log detailed error
		return nil, err
	}

	maxID := 0
	for _, t := range tasks {
		if t.ID >= maxID {
			maxID = t.ID + 1
		}
	}

	shared.Log.Debug().Int("nextID", maxID).Msg("Initialized task manager with next available ID") // Debug log

	return &TaskManager{
		tasks:  tasks,
		nextID: maxID,
	}, nil
}

func (m *TaskManager) Save() error {
	shared.Log.Debug().Msg("Saving tasks to storage") // Debug log
	err := SaveTasks(m.tasks)
	if err != nil {
		shared.Log.Error().Err(err).Msg("Failed to save tasks") // Log detailed error
	}
	return err
}

func (m *TaskManager) Add(task Task) error {
	task.ID = m.getNextID()
	task.CreatedAt = time.Now()
	m.tasks = append(m.tasks, task)

	shared.Log.Info().Msgf("Adding new task: %s", task.Title) // Info log for task added
	return m.Save()
}

func (m *TaskManager) Remove(id int) error {
	shared.Log.Debug().Int("task_id", id).Msg("Removing task by ID") // Debug log for task removal
	for i, task := range m.tasks {
		if task.ID == id {
			m.tasks = append(m.tasks[:i], m.tasks[i+1:]...)
			shared.Log.Info().Msgf("Task #%d removed", id) // Info log for task removal
			return m.Save()
		}
	}
	shared.Log.Warn().Int("task_id", id).Msg("Attempted to remove non-existing task") // Warn log for missing task
	return errors.New("task not found")
}

func (m *TaskManager) Edit(id int, updated Task) error {
	shared.Log.Debug().Int("task_id", id).Msg("Editing task") // Debug log for task edit
	for i, task := range m.tasks {
		if task.ID == id {
			updated.ID = id
			updated.CreatedAt = task.CreatedAt
			m.tasks[i] = updated
			shared.Log.Info().Msgf("Task #%d updated", id) // Info log for task update
			return m.Save()
		}
	}
	shared.Log.Warn().Int("task_id", id).Msg("Attempted to edit non-existing task") // Warn log for missing task
	return errors.New("task not found")
}

func (m *TaskManager) List() []Task {
	shared.Log.Debug().Msg("Listing all tasks") // Debug log for task listing
	return m.tasks
}

func (m *TaskManager) Get(id int) (*Task, error) {
	shared.Log.Debug().Int("task_id", id).Msg("Fetching task by ID") // Debug log for fetching task
	for _, task := range m.tasks {
		if task.ID == id {
			shared.Log.Debug().Int("task_id", id).Msg("Task found") // Debug log for found task
			return &task, nil
		}
	}
	shared.Log.Warn().Int("task_id", id).Msg("Task not found") // Warn log for missing task
	return nil, errors.New("task not found")
}

func (m *TaskManager) ToggleComplete(id int) error {
	shared.Log.Debug().Int("task_id", id).Msg("Toggling completion status") // Debug log for toggle
	for i := range m.tasks {
		if m.tasks[i].ID == id {
			m.tasks[i].Completed = !m.tasks[i].Completed
			shared.Log.Info().Bool("completed", m.tasks[i].Completed).Msgf("Toggled completion status for task #%d", id) // Info log for toggle status
			return m.Save()
		}
	}
	shared.Log.Warn().Int("task_id", id).Msg("Attempted to toggle non-existing task") // Warn log for missing task
	return errors.New("task not found")
}

func (m *TaskManager) getNextID() int {
	id := m.nextID
	m.nextID++
	shared.Log.Debug().Int("next_task_id", id).Msg("Generated next task ID") // Debug log for ID generation
	return id
}
