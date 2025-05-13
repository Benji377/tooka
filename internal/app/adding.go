package app

import (
	"time"

	"github.com/Benji377/tooka/internal/storage"
	"github.com/Benji377/tooka/internal/task"
	tea "github.com/charmbracelet/bubbletea"
)

func updateAdding(m model, msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd
	m.inputs[m.inputIndex], cmd = m.inputs[m.inputIndex].Update(msg)

	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			err := storage.SaveTasks(m.tasks)
			if err != nil {
				panic("Failed to save tasks: " + err.Error())
			}
			return m, tea.Quit
		case "esc":
			m.adding = false
		case "tab", "down":
			m.inputs, m.inputIndex = updateInputNavigation(m.inputs, m.inputIndex, true)
		case "shift+tab", "up":
			m.inputs, m.inputIndex = updateInputNavigation(m.inputs, m.inputIndex, false)
		case "enter":
			if m.inputIndex == len(m.inputs)-1 {
				title := m.inputs[0].Value()
				desc := m.inputs[1].Value()
				due, _ := time.Parse("2006-01-02 15:04", m.inputs[2].Value())
				priorityInput := m.inputs[3].Value()

				var priority task.Priority
				switch priorityInput {
				case "low":
					priority = task.Low
				case "medium":
					priority = task.Medium
				case "severe":
					priority = task.Severe
				default:
					priority = task.Low // Default to low if no valid input
				}

				newTask := task.Task{
					ID:          len(m.tasks) + 1,
					Title:       title,
					Description: desc,
					DueDate:     due,
					CreatedAt:   time.Now(),
					Completed:   false,
					Priority:    priority,
				}
				m.tasks = append(m.tasks, newTask)
				err := storage.SaveTasks(m.tasks)
				if err != nil {
					panic("Failed to save tasks: " + err.Error())
				}
				m.adding = false
			}
		}
	}

	return m, cmd
}

func updateEditing(m model, msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd
	m.editingInputs[m.inputIndex], cmd = m.editingInputs[m.inputIndex].Update(msg)

	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			err := storage.SaveTasks(m.tasks)
			if err != nil {
				panic("Failed to save tasks: " + err.Error())
			}
			return m, tea.Quit
		case "esc":
			m.adding = false
		case "tab", "down":
			m.editingInputs, m.inputIndex = updateInputNavigation(m.editingInputs, m.inputIndex, true)
		case "shift+tab", "up":
			m.editingInputs, m.inputIndex = updateInputNavigation(m.editingInputs, m.inputIndex, false)
		case "enter":
			if m.inputIndex == len(m.editingInputs)-1 {
				t := &m.tasks[m.cursor]
				t.Title = m.editingInputs[0].Value()
				t.Description = m.editingInputs[1].Value()
				due, _ := time.Parse("2006-01-02 15:04", m.editingInputs[2].Value())
				t.DueDate = due
				priorityInput := m.editingInputs[3].Value()
				switch priorityInput {
				case "low":
					t.Priority = task.Low
				case "medium":
					t.Priority = task.Medium
				case "severe":
					t.Priority = task.Severe
				default:
					t.Priority = task.Low // Default to low if no valid input
				}
				storage.SaveTasks(m.tasks)
				m.viewedTask = *t
				m.editing = false
				m.viewingTask = true
			}
		}
	}

	return m, cmd
}
