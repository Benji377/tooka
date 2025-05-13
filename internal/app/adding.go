package app

import (
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/Benji377/tooka/internal/task"
	"github.com/Benji377/tooka/internal/storage"
)

func updateAdding(m model, msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd
	m.inputs[m.inputIndex], cmd = m.inputs[m.inputIndex].Update(msg)

	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "tab":
			m.inputs, m.inputIndex = updateInputNavigation(m.inputs, m.inputIndex, true)
		case "shift+tab":
			m.inputs, m.inputIndex = updateInputNavigation(m.inputs, m.inputIndex, false)
		case "enter":
			if m.inputIndex == len(m.inputs)-1 {
				title := m.inputs[0].Value()
				desc := m.inputs[1].Value()
				due, _ := time.Parse("2006-01-02 15:04", m.inputs[2].Value())

				newTask := task.Task{
					ID:          len(m.tasks) + 1,
					Title:       title,
					Description: desc,
					DueDate:     due,
					CreatedAt:   time.Now(),
					Completed:   false,
				}
				m.tasks = append(m.tasks, newTask)
				storage.SaveTasks(m.tasks)
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
		case "tab":
			m.editingInputs, m.inputIndex = updateInputNavigation(m.editingInputs, m.inputIndex, true)
		case "shift+tab":
			m.editingInputs, m.inputIndex = updateInputNavigation(m.editingInputs, m.inputIndex, false)
		case "enter":
			if m.inputIndex == len(m.editingInputs)-1 {
				t := &m.tasks[m.cursor]
				t.Title = m.editingInputs[0].Value()
				t.Description = m.editingInputs[1].Value()
				due, _ := time.Parse("2006-01-02 15:04", m.editingInputs[2].Value())
				t.DueDate = due
				storage.SaveTasks(m.tasks)
				m.viewedTask = *t
				m.editing = false
				m.viewingTask = true
			}
		}
	}

	return m, cmd
}
