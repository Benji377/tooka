package core

import (
	"errors"
	"time"
)

type TaskManager struct {
	tasks []Task
}

func NewTaskManager() (*TaskManager, error) {
	tasks, err := LoadTasks()
	if err != nil {
		return nil, err
	}
	return &TaskManager{tasks: tasks}, nil
}

func (m *TaskManager) Save() error {
	return SaveTasks(m.tasks)
}

func (m *TaskManager) Add(task Task) error {
	task.ID = m.getNextID()
	task.CreatedAt = time.Now()
	m.tasks = append(m.tasks, task)
	return m.Save()
}

func (m *TaskManager) Remove(id int) error {
	for i, task := range m.tasks {
		if task.ID == id {
			m.tasks = append(m.tasks[:i], m.tasks[i+1:]...)
			return m.Save()
		}
	}
	return errors.New("task not found")
}

func (m *TaskManager) Edit(id int, updated Task) error {
	for i, task := range m.tasks {
		if task.ID == id {
			updated.ID = id
			updated.CreatedAt = task.CreatedAt
			m.tasks[i] = updated
			return m.Save()
		}
	}
	return errors.New("task not found")
}

func (m *TaskManager) List() []Task {
	return m.tasks
}

func (m *TaskManager) Get(id int) (*Task, error) {
	for _, task := range m.tasks {
		if task.ID == id {
			return &task, nil
		}
	}
	return nil, errors.New("task not found")
}

func (m *TaskManager) ToggleComplete(id int) error {
	for i := range m.tasks {
		if m.tasks[i].ID == id {
			m.tasks[i].Completed = !m.tasks[i].Completed
			return m.Save()
		}
	}
	return errors.New("task not found")
}

func (m *TaskManager) getNextID() int {
	maxID := 0
	for _, task := range m.tasks {
		if task.ID > maxID {
			maxID = task.ID
		}
	}
	return maxID + 1
}
