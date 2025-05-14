package ui

import (
	"time"

	"github.com/Benji377/tooka/internal/core"
	tea "github.com/charmbracelet/bubbletea"
)

func updateAdding(m *model, msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd
	m.inputs[m.inputIndex], cmd = m.inputs[m.inputIndex].Update(msg)

	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			core.SaveTasks(m.taskManager.List())
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
				due, _ := time.Parse("2006-01-02", m.inputs[2].Value())
				priorityInput := m.inputs[3].Value()

				var priority core.Priority
				switch priorityInput {
				case "low":
					priority = core.Low
				case "medium":
					priority = core.Medium
				case "severe":
					priority = core.Severe
				default:
					priority = core.Low // Default to low if no valid input
				}

				newTask := core.Task{
					ID:          len(m.taskManager.List()) + 1,
					Title:       title,
					Description: desc,
					DueDate:     due,
					CreatedAt:   time.Now(),
					Completed:   false,
					Priority:    priority,
				}
				m.taskManager.Add(newTask)
				m.adding = false
			}
		}
	}

	return m, cmd
}

func updateEditing(m *model, msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd
	m.editingInputs[m.inputIndex], cmd = m.editingInputs[m.inputIndex].Update(msg)

	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			core.SaveTasks(m.taskManager.List())
			return m, tea.Quit
		case "esc":
			m.adding = false
		case "tab", "down":
			m.editingInputs, m.inputIndex = updateInputNavigation(m.editingInputs, m.inputIndex, true)
		case "shift+tab", "up":
			m.editingInputs, m.inputIndex = updateInputNavigation(m.editingInputs, m.inputIndex, false)
		case "enter":
			if m.inputIndex == len(m.editingInputs)-1 {
				tasks := m.taskManager.List()
				if m.cursor >= 0 && m.cursor < len(tasks) {
					t := &tasks[m.cursor]
					t.Title = m.editingInputs[0].Value()
					t.Description = m.editingInputs[1].Value()
					due, _ := time.Parse("2006-01-02", m.editingInputs[2].Value())
					t.DueDate = due
					priorityInput := m.editingInputs[3].Value()
					switch priorityInput {
					case "low":
						t.Priority = core.Low
					case "medium":
						t.Priority = core.Medium
					case "severe":
						t.Priority = core.Severe
					default:
						t.Priority = core.Low // Default to low if no valid input
					}
					// Update the task in the task manager
					m.taskManager.Edit(m.cursor, *t)
					core.SaveTasks(m.taskManager.List())
				}
				m.editing = false
			}
		}
	}

	return m, cmd
}
