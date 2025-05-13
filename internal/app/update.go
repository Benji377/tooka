package app

import (
	"time"

	"github.com/Benji377/tooka/internal/storage"
	"github.com/Benji377/tooka/internal/task"
	tea "github.com/charmbracelet/bubbletea"
)

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			storage.SaveTasks(m.tasks)
			return m, tea.Quit

		case "a":
			m.adding = true
			m.inputIndex = 0
			for i := range m.inputs {
				m.inputs[i].SetValue("")
				m.inputs[i].Blur()
			}
			m.inputs[0].Focus()

		case "enter":
			if m.adding {
				if m.inputIndex < len(m.inputs)-1 {
					m.inputs[m.inputIndex].Blur()
					m.inputIndex++
					m.inputs[m.inputIndex].Focus()
				} else {
					title := m.inputs[0].Value()
					desc := m.inputs[1].Value()
					due, _ := time.Parse("2006-01-02", m.inputs[2].Value())

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
			} else if m.editing {
				t := &m.tasks[m.cursor]
				t.Title = m.editingInputs[0].Value()
				t.Description = m.editingInputs[1].Value()
				due, _ := time.Parse("2006-01-02", m.editingInputs[2].Value())
				t.DueDate = due
				storage.SaveTasks(m.tasks)
				m.viewingTask = true
				m.editing = false
			} else if m.viewingTask {
				m.viewingTask = false
			} else if len(m.tasks) > 0 {
				m.viewingTask = true
				m.viewedTask = m.tasks[m.cursor]
			}

		case "esc":
			if m.adding || m.editing {
				m.adding = false
				m.editing = false
			} else {
				m.viewingTask = false
			}

		case "e":
			if m.viewingTask {
				t := m.viewedTask
				m.editing = true
				m.viewingTask = false
				m.inputIndex = 0
				m.editingInputs[0].SetValue(t.Title)
				m.editingInputs[1].SetValue(t.Description)
				m.editingInputs[2].SetValue(t.DueDate.Format("2006-01-02"))
				m.editingInputs[0].Focus()
			}

		case " ":
			if m.viewingTask {
				t := &m.tasks[m.cursor]
				t.Completed = !t.Completed
				storage.SaveTasks(m.tasks)
				m.viewedTask = *t
			}

		case "up", "k":
			if !m.adding && !m.viewingTask && m.cursor > 0 {
				m.cursor--
			}

		case "down", "j":
			if !m.adding && !m.viewingTask && m.cursor < len(m.tasks)-1 {
				m.cursor++
			}

		case "d":
			if !m.adding && !m.viewingTask && len(m.tasks) > 0 {
				m.tasks = append(m.tasks[:m.cursor], m.tasks[m.cursor+1:]...)
				if m.cursor > 0 {
					m.cursor--
				}
				storage.SaveTasks(m.tasks)
			}
		}
	}

	if m.adding {
		return updateAdding(m, msg)
	}

	if m.editing {
		return updateEditing(m, msg)
	}

	return m, nil
}
