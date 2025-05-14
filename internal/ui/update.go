package ui

import (
	"fmt"
	"os"
	"os/exec"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/internal/shared"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
)

func (m *model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		if m.width < minWidth || m.height < minHeight {
			m.errMsg = fmt.Sprintf("Terminal too small. Resize to at least %dx%d", minWidth, minHeight)
		} else {
			m.errMsg = ""
		}
	case tea.KeyMsg:
		// Ignore global shortcuts when typing into inputs
		if m.adding && m.inputs[m.inputIndex].Focused() {
			return updateAdding(m, msg)
		}
		if m.editing && m.editingInputs[m.inputIndex].Focused() {
			return updateEditing(m, msg)
		}

		switch msg.String() {
		case "ctrl+c", "q":
			clearTerminal()
			err := core.SaveTasks(m.taskManager.List())
			if err != nil {
				shared.Log.Err(err).Msg("Failed to save tasks")
				os.Exit(1)
			}
			return m, tea.Quit
		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
			}
		case "down", "j":
			if m.cursor < len(m.taskManager.List())-1 {
				m.cursor++
			}
		case " ":
			tasks := m.taskManager.List()
			if len(tasks) > 0 {
				_ = m.taskManager.ToggleComplete(tasks[m.cursor].ID)
			}
		case "a":
			m.adding = true
			m.inputIndex = 0
			for i := range m.inputs {
				m.inputs[i].SetValue("")
				m.inputs[i].Blur()
			}
			m.inputs[0].Focus()
		case "e":
			t := m.taskManager.List()[m.cursor]
			m.editing = true
			m.inputIndex = 0
			m.editingInputs[0].SetValue(t.Title)
			m.editingInputs[1].SetValue(t.Description)
			m.editingInputs[2].SetValue(t.DueDate.Format("2006-01-02"))
			m.editingInputs[0].Focus()
		case "r":
			if !m.adding && len(m.taskManager.List()) > 0 {
				tasks := m.taskManager.List()
				_ = m.taskManager.Remove(tasks[m.cursor].ID)
				if m.cursor > 0 {
					m.cursor--
				}
				err := core.SaveTasks(m.taskManager.List())
				if err != nil {
					shared.Log.Err(err).Msg("Failed to save tasks")
					os.Exit(1)
				}
			}
		case "s":
			m.toggleSort()

		case "d":
			m.toggleSortDirection()
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

func (m *model) toggleSort() {
	switch m.sortBy {
	case SortByTitle:
		m.sortBy = SortByDueDate
	case SortByDueDate:
		m.sortBy = SortByPriority
	case SortByPriority:
		m.sortBy = SortByState
	case SortByState:
		m.sortBy = SortByTitle
	}
}

func (m *model) toggleSortDirection() {
	if m.sortDir == Asc {
		m.sortDir = Desc
	} else {
		m.sortDir = Asc
	}
}

func clearTerminal() {
	cmd := exec.Command("clear") // For Unix-like systems (Linux, macOS)
	if os.Getenv("OS") == "Windows_NT" {
		cmd = exec.Command("cmd", "/c", "cls") // For Windows
	}
	cmd.Stdout = os.Stdout
	err := cmd.Run()
	if err != nil {
		shared.Log.Err(err).Msg("Failed to clear terminal")
		os.Exit(1)
	}
}

func updateInputNavigation(inputs []textinput.Model, index int, forward bool) ([]textinput.Model, int) {
	inputs[index].Blur()
	if forward && index < len(inputs)-1 {
		index++
	} else if !forward && index > 0 {
		index--
	}
	inputs[index].Focus()
	return inputs, index
}
